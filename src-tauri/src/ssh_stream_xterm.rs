use once_cell::sync::Lazy;
use serde::Serialize;
use ssh2::Session;
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
    println!("[SSH Worker] Progress event emitted");

    /* ===== LOAD HOST ===== */
    println!("[SSH Worker] Loading host data from DB...");
    let db = app.state::<Db>();
    let conn = db.conn.lock().map_err(|_| "DB lock failed")?;
    println!("[SSH Worker] DB lock acquired");

    let (host, port, username, password): (String, i64, String, Option<String>) = conn
        .query_row(
            "SELECT host, port, username, password FROM hosts WHERE id = ?",
            [host_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .map_err(|e| {
            println!("[SSH Worker] DB query failed: {:?}", e);
            e.to_string()
        })?;

    println!("[SSH Worker] Host data loaded: {}:{}", host, port);
    drop(conn);

    let password = password.ok_or_else(|| {
        println!("[SSH Worker] No password found");
        "Password auth required".to_string()
    })?;

    /* ===== TCP ===== */
    println!("[SSH Worker] Connecting to {}:{}...", host, port);
    let addr = (host.as_str(), port as u16)
        .to_socket_addrs()
        .map_err(|_| "Invalid address")?
        .next()
        .ok_or("Resolve failed")?;

    let tcp =
        TcpStream::connect_timeout(&addr, Duration::from_secs(SSH_TIMEOUT_SECS)).map_err(|e| {
            println!("[SSH Worker] TCP connect failed: {:?}", e);
            "SSH connect timeout".to_string()
        })?;

    println!("[SSH Worker] TCP connected");

    /* ===== SSH ===== */
    println!("[SSH Worker] Starting SSH handshake...");
    let mut sess = Session::new().map_err(|e| {
        println!("[SSH Worker] Session creation failed: {:?}", e);
        "SSH session failed".to_string()
    })?;
    sess.set_tcp_stream(tcp);

    // Use longer timeout for all setup operations (handshake, auth, channel, pty, shell)
    sess.set_timeout(10000); // 10 seconds

    sess.handshake().map_err(|e| {
        println!("[SSH Worker] Handshake failed: {:?}", e);
        e.to_string()
    })?;
    println!("[SSH Worker] SSH handshake completed");

    sess.userauth_password(&username, &password).map_err(|e| {
        println!("[SSH Worker] Auth failed: {:?}", e);
        e.to_string()
    })?;
    println!("[SSH Worker] Authentication successful");

    /* ===== CHANNEL ===== */
    println!("[SSH Worker] Opening SSH channel...");
    let mut channel = sess.channel_session().map_err(|e| {
        println!("[SSH Worker] Channel creation failed: {:?}", e);
        e.to_string()
    })?;
    println!("[SSH Worker] Channel created");

    channel
        .request_pty("xterm-256color", None, None)
        .map_err(|e| {
            println!("[SSH Worker] PTY request failed: {:?}", e);
            e.to_string()
        })?;
    println!("[SSH Worker] PTY requested");

    channel.shell().map_err(|e| {
        println!("[SSH Worker] Shell request failed: {:?}", e);
        e.to_string()
    })?;
    println!("[SSH Worker] Shell started");

    // Set reasonable timeout for I/O operations
    // 200ms is enough to detect real errors but not cause unnecessary delays
    sess.set_timeout(100);

    println!("[SSH Worker] Starting I/O threads for task: {}", task_id);

    // Wait a bit for shell to initialize
    std::thread::sleep(Duration::from_millis(300));

    // Create channel for output thread to signal errors
    let (output_err_tx, output_err_rx) = std::sync::mpsc::channel::<String>();
    let (input_err_tx, input_err_rx) = std::sync::mpsc::channel::<String>();

    // Wrap channel in Arc<Mutex> for sharing between threads
    let channel = Arc::new(Mutex::new(channel));
    let channel_reader = channel.clone();
    let channel_writer = channel.clone();

    let task_id_reader = task_id.clone();
    let task_id_writer = task_id.clone();
    let app_reader = app.clone();
    let cancel_reader = task.cancel.clone();
    let cancel_writer = task.cancel.clone();

    // Output reader thread
    let reader_handle = std::thread::spawn(move || {
        let mut buf = [0u8; 8192];
        let mut total = 0usize;

        loop {
            if cancel_reader.load(Ordering::Relaxed) {
                println!("[Reader] Cancel flag detected, exiting");
                break;
            }

            let mut ch = match channel_reader.lock() {
                Ok(ch) => ch,
                Err(e) => {
                    println!("[Reader] Lock failed: {:?}", e);
                    let _ = output_err_tx.send("Lock failed".into());
                    break;
                }
            };

            match ch.read(&mut buf) {
                Ok(0) => {
                    println!("[Reader] EOF, connection closed");
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
                    // Timeout is NORMAL when there's no data - don't exit!
                    drop(ch); // Release lock before sleep
                    std::thread::sleep(Duration::from_millis(3)); // Very short sleep
                }
                Err(e) => {
                    println!("[Reader] Read error (fatal): {:?}", e);
                    break;
                }
            }
        }
        println!("[Reader] Thread exiting");
    });

    // Input writer thread
    let writer_handle = std::thread::spawn(move || {
        loop {
            if cancel_writer.load(Ordering::Relaxed) {
                println!("[Writer] Cancel flag detected, exiting");
                break;
            }

            match stdin_rx.recv_timeout(Duration::from_millis(50)) {
                Ok(input) => {
                    let mut ch = match channel_writer.lock() {
                        Ok(ch) => ch,
                        Err(e) => {
                            println!("[Writer] Lock failed: {:?}", e);
                            let _ = input_err_tx.send("Lock failed".into());
                            break;
                        }
                    };

                    if let Err(e) = ch.write_all(&input) {
                        println!("[Writer] Write failed: {:?}", e);
                        let _ = input_err_tx.send(format!("{:?}", e));
                        break;
                    }

                    if let Err(e) = ch.flush() {
                        println!("[Writer] Flush failed: {:?}", e);
                        // Don't break on flush error, just log
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // No input, continue - THIS IS NORMAL
                    continue;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    println!("[Writer] Input channel disconnected (frontend closed)");
                    break;
                }
            }
        }
        println!("[Writer] Thread exiting");
    });

    // Wait for either thread to finish
    reader_handle.join().ok();
    writer_handle.join().ok();

    // Check for errors
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

    // Get exit code
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
