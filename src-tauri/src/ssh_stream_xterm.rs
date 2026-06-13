use crate::crypto;
use once_cell::sync::Lazy;
use serde::Serialize;
use ssh2::{KeyboardInteractivePrompt, Prompt, Session};
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager};

use crate::db::Db;

/* =========================
   CONFIG
========================= */

const SSH_TIMEOUT_SECS: u64 = 10;
const MAX_OUTPUT_BYTES: usize = 10 * 1024 * 1024; // 10MB

/* =========================
   GLOBAL TASK REGISTRY
========================= */

struct SshTask {
    cancel: Arc<AtomicBool>,
    stdin_tx: Mutex<Option<std::sync::mpsc::Sender<Vec<u8>>>>,
}

static SSH_TASKS: Lazy<Mutex<HashMap<String, Arc<SshTask>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/* =========================
   EVENTS
========================= */

#[derive(Serialize, Clone)]
struct SshProgressEvent {
    task_id: String,
    status: String,
}

#[derive(Serialize, Clone)]
struct SshStdoutEvent {
    task_id: String,
    data: String,
}

#[derive(Serialize, Clone)]
struct SshDoneEvent {
    task_id: String,
    exit_code: i32,
}

/* =========================
   TAURI COMMANDS
========================= */

#[tauri::command]
pub fn ssh_exec_start(task_id: String, host_id: i64, app: AppHandle) -> Result<(), String> {
    let (stdin_tx, stdin_rx) = std::sync::mpsc::channel::<Vec<u8>>();

    let task = Arc::new(SshTask {
        cancel: Arc::new(AtomicBool::new(false)),
        stdin_tx: Mutex::new(Some(stdin_tx)),
    });

    SSH_TASKS
        .lock()
        .unwrap()
        .insert(task_id.clone(), task.clone());

    std::thread::spawn(move || {
        let _ = ssh_exec_worker(task_id, host_id, task, app, stdin_rx);
    });

    Ok(())
}

#[tauri::command]
pub fn ssh_exec_input(task_id: String, data: String) -> Result<(), String> {
    let tasks = SSH_TASKS.lock().unwrap();

    if let Some(task) = tasks.get(&task_id) {
        let stdin_guard = task.stdin_tx.lock().unwrap();

        if let Some(tx) = stdin_guard.as_ref() {
            match tx.send(data.into_bytes()) {
                Ok(_) => Ok(()),
                Err(e) => {
                    println!("[ssh_exec_input] ✗ Channel send failed: {:?}", e);
                    Err(format!("Channel send failed: {:?}", e))
                }
            }
        } else {
            Err("Stdin sender not available".into())
        }
    } else {
        Err("Task not found".into())
    }
}

#[tauri::command]
pub fn ssh_exec_cancel(task_id: String) {
    if let Some(task) = SSH_TASKS.lock().unwrap().get(&task_id) {
        task.cancel.store(true, Ordering::Relaxed);
    }
}

/* =========================
   WORKER
========================= */
// ─── Keyboard-interactive handler ────────────────────────────────────────────
// Dipakai saat auth_type = "none": server akan kirim prompt (misal "Password:")
// kita teruskan ke xterm lewat emit_stdout, lalu tunggu user ketik dan tekan Enter.

struct PtyPasswordHandler {
    app: AppHandle,
    task_id: String,
    // stdin_rx dibungkus Arc<Mutex> supaya bisa dipakai di sini
    // setelah auth selesai, Arc yang sama diteruskan ke writer thread
    stdin_rx: Arc<Mutex<std::sync::mpsc::Receiver<Vec<u8>>>>,
}

impl KeyboardInteractivePrompt for PtyPasswordHandler {
    fn prompt(
        &mut self,
        _username: &str,
        _instructions: &str,
        prompts: &[Prompt<'_>],
    ) -> Vec<String> {
        prompts
            .iter()
            .map(|p| {
                // Tampilkan prompt server di terminal (misal "Password: ")
                emit_stdout(&self.app, &self.task_id, p.text.to_string());

                // Tunggu input dari user (max 60 detik)
                let rx = self.stdin_rx.lock().unwrap();
                let deadline = std::time::Instant::now() + Duration::from_secs(60);

                loop {
                    let remaining = deadline.saturating_duration_since(std::time::Instant::now());
                    if remaining.is_zero() {
                        println!("[Auth] Timeout waiting for keyboard-interactive input");
                        return String::new();
                    }

                    match rx.recv_timeout(remaining.min(Duration::from_millis(100))) {
                        Ok(bytes) => {
                            // Trim newline yang dikirim saat user tekan Enter
                            let s = String::from_utf8_lossy(&bytes);
                            return s.trim_end_matches('\n').trim_end_matches('\r').to_string();
                        }
                        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                            return String::new();
                        }
                    }
                }
            })
            .collect()
    }
}

fn ssh_exec_worker(
    task_id: String,
    host_id: i64,
    task: Arc<SshTask>,
    app: AppHandle,
    stdin_rx: std::sync::mpsc::Receiver<Vec<u8>>,
) -> Result<(), String> {
    println!(
        "[SSH Worker] ========== STARTED for task: {} ==========",
        task_id
    );

    emit_progress(&app, &task_id, "running");

    // ── Load host dari DB ──────────────────────────────────────────────────
    let db = app.state::<Db>();
    let (host, port, username, password_id, auth_type): (String, i64, String, Option<i64>, String) = {
        let conn = db.conn.lock().map_err(|_| "DB lock failed")?;
        conn.query_row(
            "SELECT host, port, username, password_id, auth_type FROM hosts WHERE id = ?",
            [host_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .map_err(|e| e.to_string())?
    };

    println!(
        "[SSH Worker] Host: {}:{}, auth_type: {}",
        host, port, auth_type
    );

    let db_password: Option<String> = if auth_type == "password" {
        let conn = db.conn.lock().map_err(|_| "DB lock failed")?;
        let pid = password_id.ok_or("password_id is null")?;
        let pw: String = conn
            .query_row("SELECT password FROM passwords WHERE id = ?", [pid], |r| {
                r.get(0)
            })
            .map_err(|e| e.to_string())?;
        Some(pw)
    } else {
        None
    };

    // ── TCP connect ────────────────────────────────────────────────────────
    println!("[SSH Worker] Connecting to {}:{}...", host, port);
    emit_stdout(
        &app,
        &task_id,
        format!("Connecting to {}:{}...\r\n", host, port),
    );

    let addr = match (host.as_str(), port as u16).to_socket_addrs() {
        Ok(mut addrs) => match addrs.next() {
            Some(addr) => addr,
            None => {
                let msg = "Resolve failed\r\n".to_string();

                emit_stdout(&app, &task_id, msg.clone());
                emit_progress(&app, &task_id, "error");
                cleanup_task(&task_id);

                return Err(msg.trim().to_string());
            }
        },
        Err(e) => {
            let msg = format!("Invalid address: {}\r\n", e);

            emit_stdout(&app, &task_id, msg.clone());
            emit_progress(&app, &task_id, "error");
            cleanup_task(&task_id);

            return Err(msg.trim().to_string());
        }
    };

    let tcp = match TcpStream::connect_timeout(&addr, Duration::from_secs(SSH_TIMEOUT_SECS)) {
        Ok(tcp) => tcp,
        Err(e) => {
            let msg = format!("SSH connect failed: {}\r\n", e);

            emit_stdout(&app, &task_id, msg.clone());
            emit_progress(&app, &task_id, "error");
            cleanup_task(&task_id);

            return Err(msg.trim().to_string());
        }
    };

    // ── SSH handshake ──────────────────────────────────────────────────────
    let mut sess = Session::new().map_err(|_| "SSH session failed".to_string())?;
    sess.set_tcp_stream(tcp);
    sess.set_timeout(10_000); // 10s untuk handshake + auth
    sess.handshake().map_err(|e| e.to_string())?;
    println!("[SSH Worker] Handshake OK");

    // ── Bungkus stdin_rx sebelum auth ──────────────────────────────────────
    // Arc<Mutex> diperlukan supaya PtyPasswordHandler bisa pakai stdin_rx,
    // lalu setelah auth selesai Arc yang sama diteruskan ke writer thread.
    let stdin_rx_shared = Arc::new(Mutex::new(stdin_rx));

    // ── Autentikasi ────────────────────────────────────────────────────────
    match auth_type.as_str() {
        "password" => {
            let pw = db_password.as_deref().ok_or("Password not found")?;
            let pw_plain_text = crypto::decrypt(&pw).unwrap_or_default();

            match sess.userauth_password(&username, &pw_plain_text) {
                Ok(_) => {
                    println!("[SSH Worker] Password auth OK");
                }
                Err(e) => {
                    let msg = format!("Auth failed: {}\r\n", e);

                    println!("[SSH Worker] {}", msg.trim_end());
                    emit_stdout(&app, &task_id, msg);
                    emit_progress(&app, &task_id, "error");
                    cleanup_task(&task_id);

                    return Err(format!("Auth failed: {}", e));
                }
            }
        }
        _ => {
            println!("[SSH Worker] auth_type=none, trying available methods...");

            // Cek method apa yang server support
            let methods = sess.auth_methods(&username).unwrap_or_default();
            println!("[SSH Worker] Server auth methods: {}", methods);

            if methods.contains("keyboard-interactive") {
                let mut handler = PtyPasswordHandler {
                    app: app.clone(),
                    task_id: task_id.clone(),
                    stdin_rx: stdin_rx_shared.clone(),
                };
                match sess.userauth_keyboard_interactive(&username, &mut handler) {
                    Ok(_) => println!("[SSH Worker] keyboard-interactive OK"),
                    Err(e) => println!("[SSH Worker] keyboard-interactive failed: {:?}", e),
                }
            }

            // Kalau masih belum auth, coba password method tapi prompt via terminal
            if !sess.authenticated() && methods.contains("password") {
                println!("[SSH Worker] Trying password method via PTY prompt...");

                let mut last_error = String::new();

                for attempt in 1..=3 {
                    emit_stdout(&app, &task_id, "Password: ".to_string());

                    let password_input = {
                        let rx = stdin_rx_shared.lock().unwrap();
                        let mut buffer = String::new();
                        let deadline = std::time::Instant::now() + Duration::from_secs(60);

                        loop {
                            let remaining =
                                deadline.saturating_duration_since(std::time::Instant::now());

                            if remaining.is_zero() {
                                emit_stdout(
                                    &app,
                                    &task_id,
                                    "\r\nPassword input timeout\r\n".to_string(),
                                );
                                break;
                            }

                            match rx.recv_timeout(remaining.min(Duration::from_millis(100))) {
                                Ok(bytes) => {
                                    let s = String::from_utf8_lossy(&bytes);
                                    let mut done = false;

                                    for ch in s.chars() {
                                        match ch {
                                            '\r' | '\n' => {
                                                emit_stdout(&app, &task_id, "\r\n".to_string());
                                                done = true;
                                                break;
                                            }
                                            '\x08' | '\x7f' => {
                                                buffer.pop();
                                            }
                                            c if !c.is_control() => {
                                                buffer.push(c);
                                            }
                                            _ => {}
                                        }
                                    }

                                    if done {
                                        break;
                                    }
                                }
                                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                                Err(_) => break,
                            }
                        }

                        buffer
                    };

                    match sess.userauth_password(&username, &password_input) {
                        Ok(_) => {
                            println!("[SSH Worker] Password auth via prompt OK");
                            break;
                        }
                        Err(e) => {
                            last_error = e.message().to_string();

                            if attempt < 3 {
                                emit_stdout(
                                    &app,
                                    &task_id,
                                    format!(
                                        "Authentication failed. Try again. ({}/3)\r\n",
                                        attempt
                                    ),
                                );
                            } else {
                                emit_stdout(
                                    &app,
                                    &task_id,
                                    "Authentication failed 3 times. Session closed.\r\n"
                                        .to_string(),
                                );
                            }
                        }
                    }
                }

                if !sess.authenticated() {
                    emit_progress(&app, &task_id, "error");
                    cleanup_task(&task_id);
                    return Err(format!("Auth failed after 3 attempts: {}", last_error));
                }
            }

            // if !sess.authenticated() {
            //     println!("[SSH Worker] All auth methods failed, continuing anyway...");
            // }
        }
    }

    // ── Buka channel + PTY + shell ─────────────────────────────────────────
    let mut channel = match sess.channel_session() {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("Failed to open channel session: {}\r\n", e);

            emit_stdout(&app, &task_id, msg.clone());
            emit_progress(&app, &task_id, "error");
            cleanup_task(&task_id);

            return Err(msg.trim().to_string());
        }
    };

    match channel.request_pty("xterm-256color", None, None) {
        Ok(_) => {
            println!("[SSH Worker] PTY allocated");
        }
        Err(e) => {
            let msg = format!("Failed to request PTY: {}\r\n", e);

            emit_stdout(&app, &task_id, msg.clone());
            emit_progress(&app, &task_id, "error");
            cleanup_task(&task_id);

            return Err(msg.trim().to_string());
        }
    }

    match channel.shell() {
        Ok(_) => {
            println!("[SSH Worker] Shell started");
        }
        Err(e) => {
            let msg = format!("Failed to start shell: {}\r\n", e);

            emit_stdout(&app, &task_id, msg.clone());
            emit_progress(&app, &task_id, "error");
            cleanup_task(&task_id);

            return Err(msg.trim().to_string());
        }
    };
    println!("[SSH Worker] Shell started");

    // I/O timeout lebih pendek setelah shell up
    sess.set_timeout(100);

    std::thread::sleep(Duration::from_millis(300));

    // ── Error channels antar thread ────────────────────────────────────────
    let (output_err_tx, output_err_rx) = std::sync::mpsc::channel::<String>();
    let (input_err_tx, input_err_rx) = std::sync::mpsc::channel::<String>();

    let channel = Arc::new(Mutex::new(channel));
    let channel_reader = channel.clone();
    let channel_writer = channel.clone();

    let task_id_reader = task_id.clone();
    let app_reader = app.clone();
    let cancel_reader = task.cancel.clone();
    let cancel_writer = task.cancel.clone();

    // ── Reader thread ──────────────────────────────────────────────────────
    let reader_handle = std::thread::spawn(move || {
        let mut buf = [0u8; 8192];
        let mut total = 0usize;

        loop {
            if cancel_reader.load(Ordering::Relaxed) {
                break;
            }

            let mut ch = match channel_reader.lock() {
                Ok(ch) => ch,
                Err(_) => {
                    let _ = output_err_tx.send("Lock failed".into());
                    break;
                }
            };

            match ch.read(&mut buf) {
                Ok(0) => {
                    println!("[Reader] EOF");
                    break;
                }
                Ok(n) => {
                    total += n;
                    if total > MAX_OUTPUT_BYTES {
                        let _ = output_err_tx.send("Output limit exceeded".into());
                        break;
                    }
                    emit_stdout(
                        &app_reader,
                        &task_id_reader,
                        String::from_utf8_lossy(&buf[..n]).to_string(),
                    );
                }
                Err(e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    drop(ch);
                    std::thread::sleep(Duration::from_millis(3));
                }
                Err(e) => {
                    println!("[Reader] Fatal read error: {:?}", e);
                    break;
                }
            }
        }
    });

    // ── Writer thread ──────────────────────────────────────────────────────
    // Pakai Arc<Mutex<Receiver>> yang sama dengan yang dipakai handler auth
    let writer_handle = std::thread::spawn(move || {
        loop {
            if cancel_writer.load(Ordering::Relaxed) {
                break;
            }

            let input = {
                // Lock sebentar hanya untuk recv, lalu lepas
                match stdin_rx_shared.lock() {
                    Ok(rx) => rx.recv_timeout(Duration::from_millis(50)),
                    Err(_) => {
                        let _ = input_err_tx.send("stdin_rx lock failed".into());
                        break;
                    }
                }
            };

            match input {
                Ok(data) => {
                    let mut ch = match channel_writer.lock() {
                        Ok(ch) => ch,
                        Err(_) => {
                            let _ = input_err_tx.send("Channel lock failed".into());
                            break;
                        }
                    };
                    if let Err(e) = ch.write_all(&data) {
                        println!("[Writer] Write failed: {:?}", e);
                        let _ = input_err_tx.send(format!("{:?}", e));
                        break;
                    }
                    let _ = ch.flush();
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    println!("[Writer] stdin disconnected");
                    break;
                }
            }
        }
    });

    // ── Tunggu selesai ─────────────────────────────────────────────────────
    reader_handle.join().ok();
    writer_handle.join().ok();

    if let Ok(err) = output_err_rx.try_recv() {
        emit_progress(&app, &task_id, "error");
        cleanup_task(&task_id);
        return Err(err);
    }
    if let Ok(err) = input_err_rx.try_recv() {
        emit_progress(&app, &task_id, "error");
        cleanup_task(&task_id);
        return Err(err);
    }

    let exit_code = {
        let mut ch = channel.lock().unwrap();
        ch.wait_close().ok();
        ch.exit_status().unwrap_or(-1)
    };

    emit_done(&app, &task_id, exit_code);
    cleanup_task(&task_id);

    Ok(())
}

/* =========================
   HELPERS
========================= */

fn emit_progress(app: &AppHandle, task_id: &str, status: &str) {
    let _ = app.emit(
        "ssh:progress",
        SshProgressEvent {
            task_id: task_id.into(),
            status: status.into(),
        },
    );
}

fn emit_stdout(app: &AppHandle, task_id: &str, data: String) {
    let _ = app.emit(
        "ssh:stdout",
        SshStdoutEvent {
            task_id: task_id.into(),
            data,
        },
    );
}

fn emit_done(app: &AppHandle, task_id: &str, exit_code: i32) {
    let _ = app.emit(
        "ssh:done",
        SshDoneEvent {
            task_id: task_id.into(),
            exit_code,
        },
    );
}

fn cleanup_task(task_id: &str) {
    SSH_TASKS.lock().unwrap().remove(task_id);
}
