use crate::db::Db;

use once_cell::sync::Lazy;
use serde::Serialize;
use ssh2::Session;
use std::{
    collections::HashMap,
    io::Read,
    net::{TcpStream, ToSocketAddrs},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use tauri::Manager;
use tauri::{AppHandle, Emitter};

/* =========================
CONFIG
========================= */

const SSH_TIMEOUT_SECS: u64 = 10;
const MAX_OUTPUT_BYTES: usize = 1024 * 1024; // 1MB

/* =========================
TASK REGISTRY
========================= */

static SSH_TASKS: Lazy<Mutex<HashMap<String, Arc<SshTask>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

struct SshTask {
    cancel: Arc<AtomicBool>,
}

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
    chunk: String,
}

#[derive(Serialize, Clone)]
struct SshDoneEvent {
    task_id: String,
    exit_code: i32,
}

/* =========================
COMMANDS
========================= */

#[tauri::command]
pub fn ssh_exec_start(
    task_id: String,
    host_id: i64,
    command: String,
    app: AppHandle,
) -> Result<(), String> {
    let cancel_flag = Arc::new(AtomicBool::new(false));

    SSH_TASKS.lock().unwrap().insert(
        task_id.clone(),
        Arc::new(SshTask {
            cancel: cancel_flag.clone(),
        }),
    );

    // let db = db.inner().clone();

    std::thread::spawn(move || {
        let _ = ssh_exec_worker(task_id, host_id, command, cancel_flag, app);
    });

    Ok(())
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
    command: String,
    cancel: Arc<AtomicBool>,
    app: AppHandle,
) -> Result<(), String> {
    emit_progress(&app, &task_id, "running");

    /* ===== LOAD HOST ===== */
    let db = app.state::<Db>();
    let conn = db.conn.lock().map_err(|_| "DB lock failed")?;

    let (host, port, username, password): (String, i64, String, Option<String>) = conn
        .query_row(
            "SELECT host, port, username, password FROM hosts WHERE id = ?",
            [host_id],
            |r: &rusqlite::Row| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .map_err(|e: rusqlite::Error| e.to_string())?;

    let password = password.ok_or("Password auth required")?;

    /* ===== TCP CONNECT ===== */
    let addr = (host.as_str(), port as u16)
        .to_socket_addrs()
        .map_err(|_| "Invalid address")?
        .next()
        .ok_or("Unable to resolve address")?;

    let tcp = TcpStream::connect_timeout(&addr, Duration::from_secs(SSH_TIMEOUT_SECS))
        .map_err(|_| "SSH connection timeout")?;

    /* ===== SSH SESSION ===== */
    let mut sess = Session::new().map_err(|_| "SSH session failed")?;
    sess.set_tcp_stream(tcp);
    sess.handshake().map_err(|_| "Handshake failed")?;
    sess.userauth_password(&username, &password)
        .map_err(|_| "Auth failed")?;

    /* ===== EXEC ===== */
    let mut channel = sess.channel_session().map_err(|_| "Channel failed")?;
    channel.exec(&command).map_err(|_| "Exec failed")?;

    let mut buf = [0u8; 1024];
    let mut total = 0usize;

    while !cancel.load(Ordering::Relaxed) {
        let n = match channel.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };

        total += n;
        if total > MAX_OUTPUT_BYTES {
            emit_progress(&app, &task_id, "error");
            cleanup_task(&task_id);
            return Err("Output too large".into());
        }

        emit_event(
            &app,
            "ssh:stdout",
            SshStdoutEvent {
                task_id: task_id.clone(),
                chunk: String::from_utf8_lossy(&buf[..n]).to_string(),
            },
        );
    }

    if cancel.load(Ordering::Relaxed) {
        emit_progress(&app, &task_id, "cancelled");
        cleanup_task(&task_id);
        return Ok(());
    }

    channel.wait_close().ok();
    let exit_code = channel.exit_status().unwrap_or(-1);

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

fn emit_done(app: &AppHandle, task_id: &str, exit_code: i32) {
    let _ = app.emit(
        "ssh:done",
        SshDoneEvent {
            task_id: task_id.into(),
            exit_code,
        },
    );
}

fn emit_event<T: Serialize + Clone>(app: &AppHandle, event: &str, payload: T) {
    let _ = app.emit(event, payload);
}

fn cleanup_task(task_id: &str) {
    SSH_TASKS.lock().unwrap().remove(task_id);
}
