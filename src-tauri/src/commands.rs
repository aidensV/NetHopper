use crate::db::Db;
use serde::Serialize;
/* =========================
MODELS
========================= */
const SSH_TIMEOUT_SECS: u64 = 10;
const MAX_OUTPUT_BYTES: usize = 64 * 1024;

#[derive(Serialize)]
pub struct Group {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
}

#[derive(Serialize)]
pub struct Host {
    pub id: i64,
    pub name: String,
    pub host: String,
    pub port: i64,
    pub username: String,
    pub auth_type: String,
    pub group_id: Option<i64>,
}

#[derive(serde::Serialize)]
pub struct SshExecResult {
    pub success: bool,
    pub exit_code: i32,
    pub stdout: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SshErrorKind {
    Network,
    Auth,
    Command,
    Timeout,
    Internal,
}

#[derive(serde::Serialize)]
pub struct SshError {
    pub kind: SshErrorKind,
    pub message: String,
}

#[derive(serde::Serialize)]
#[serde(untagged)]
pub enum SshResponse {
    Ok { ok: bool, data: SshExecResult },
    Err { ok: bool, error: SshError },
}

/* =========================
LIST GROUPS
========================= */

#[tauri::command]
pub fn list_groups_by_parent(
    parent_id: Option<i64>,
    db: tauri::State<Db>,
) -> Result<Vec<Group>, String> {
    let conn = db.conn.lock().map_err(|_| "DB lock failed")?;

    let (sql, params): (&str, Vec<i64>) = match parent_id {
        Some(id) => (
            "SELECT id, name, parent_id FROM groups WHERE parent_id = ?",
            vec![id],
        ),
        None => (
            "SELECT id, name, parent_id FROM groups WHERE parent_id IS NULL",
            vec![],
        ),
    };

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params_from_iter(params), |r| {
            Ok(Group {
                id: r.get(0)?,
                name: r.get(1)?,
                parent_id: r.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;

    Ok(rows.filter_map(Result::ok).collect())
}

/* =========================
LIST HOSTS
========================= */

#[tauri::command]
pub fn list_hosts_by_group(
    group_id: Option<i64>,
    db: tauri::State<Db>,
) -> Result<Vec<Host>, String> {
    let conn = db.conn.lock().map_err(|_| "DB lock failed")?;

    let (sql, params): (&str, Vec<i64>) = match group_id {
        Some(id) => (
            "SELECT id, name, host, port, username, auth_type, group_id
             FROM hosts WHERE group_id = ?",
            vec![id],
        ),
        None => (
            "SELECT id, name, host, port, username, auth_type, group_id
             FROM hosts WHERE group_id IS NULL",
            vec![],
        ),
    };

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params_from_iter(params), |r| {
            Ok(Host {
                id: r.get(0)?,
                name: r.get(1)?,
                host: r.get(2)?,
                port: r.get(3)?,
                username: r.get(4)?,
                auth_type: r.get(5)?,
                group_id: r.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;

    Ok(rows.filter_map(Result::ok).collect())
}

#[tauri::command]
pub fn create_group(
    name: String,
    parent_id: Option<i64>,
    db: tauri::State<Db>,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|_| "DB lock failed")?;

    conn.execute(
        "INSERT INTO groups (name, parent_id) VALUES (?, ?)",
        rusqlite::params![name, parent_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
#[tauri::command]
pub fn update_group(
    id: i64,
    name: String,
    parent_id: Option<i64>,
    db: tauri::State<Db>,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|_| "DB lock failed")?;

    conn.execute(
        "UPDATE groups SET name = ?, parent_id = ? WHERE id = ?",
        rusqlite::params![name, parent_id, id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn delete_group(id: i64, db: tauri::State<Db>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|_| "DB lock failed")?;

    // cek sub group
    let sub_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM groups WHERE parent_id = ?",
            rusqlite::params![id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    if sub_count > 0 {
        return Err("Group masih memiliki sub group".into());
    }

    // cek host
    let host_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM hosts WHERE group_id = ?",
            rusqlite::params![id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    if host_count > 0 {
        return Err("Group masih memiliki host".into());
    }

    conn.execute("DELETE FROM groups WHERE id = ?", rusqlite::params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn create_host(
    name: String,
    host: String,
    port: i64,
    username: String,
    password: String,
    auth_type: String,
    group_id: Option<i64>,
    db: tauri::State<Db>,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|_| "DB lock failed")?;

    conn.execute(
        "INSERT INTO hosts (name, host, port, username, password, auth_type, group_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![name, host, port, username, password, auth_type, group_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn delete_host(id: i64, db: tauri::State<Db>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|_| "DB lock failed")?;
    conn.execute("DELETE FROM hosts WHERE id = ?", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// #[tauri::command]
// pub fn update_host(
//     id: i64,
//     name: String,
//     host: String,
//     port: i64,
//     username: String,
//     auth_type: String,
//     db: tauri::State<Db>,
// ) -> Result<(), String> {
//     let conn = db.conn.lock().map_err(|_| "DB lock failed")?;

//     conn.executes(
//         "UPDATE hosts
//          SET name = ?, host = ?, port = ?, username = ?, auth_type = ?
//          WHERE id = ?",
//         (name, host, port, username, auth_type, id),
//     )
//     .map_err(|e| e.to_string())?;

//     Ok(())
// }

#[tauri::command]
pub fn update_host(
    id: i64,
    name: String,
    host: String,
    port: i64,
    username: String,
    password: String,
    auth_type: String,
    group_id: Option<i64>,
    db: tauri::State<Db>,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|_| "DB lock failed")?;

    conn.execute(
        "UPDATE hosts
         SET name = ?, host = ?, port = ?, username = ?, password = ?, auth_type = ?, group_id = ?
         WHERE id = ?",
        rusqlite::params![name, host, port, username, password, auth_type, group_id, id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn rename_group(id: i64, name: String, db: tauri::State<Db>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|_| "DB lock failed")?;

    conn.execute(
        "UPDATE groups SET name = ? WHERE id = ?",
        rusqlite::params![name, id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

fn ssh_err(kind: SshErrorKind, msg: impl Into<String>) -> SshResponse {
    SshResponse::Err {
        ok: false,
        error: SshError {
            kind,
            message: msg.into(),
        },
    }
}

#[tauri::command]
pub fn ssh_exec(
    host_id: i64,
    command: String,
    db: tauri::State<Db>,
) -> Result<SshResponse, String> {
    use ssh2::Session;
    use std::io::Read;
    use std::net::{TcpStream, ToSocketAddrs};
    use std::time::Duration;

    let conn = db.conn.lock().map_err(|_| "DB lock failed")?;

    /* =====================
       LOAD HOST
    ===================== */

    let mut stmt = conn
        .prepare(
            "SELECT host, port, username, password
         FROM hosts WHERE id = ?",
        )
        .map_err(|e| e.to_string())?;

    let (host, port, username, password): (String, i64, String, Option<String>) = stmt
        .query_row([host_id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })
        .map_err(|e| e.to_string())?;

    let password = match password {
        Some(p) => p,
        None => return Ok(ssh_err(SshErrorKind::Auth, "Password auth required")),
    };

    /* =====================
       TCP CONNECT (TIMEOUT)
    ===================== */

    let addr = match (host.as_str(), port as u16)
        .to_socket_addrs()
        .ok()
        .and_then(|mut a| a.next())
    {
        Some(a) => a,
        None => return Ok(ssh_err(SshErrorKind::Network, "Invalid host address")),
    };

    let tcp = match TcpStream::connect_timeout(&addr, Duration::from_secs(SSH_TIMEOUT_SECS)) {
        Ok(t) => t,
        Err(_) => return Ok(ssh_err(SshErrorKind::Timeout, "SSH connection timeout")),
    };

    tcp.set_read_timeout(Some(Duration::from_secs(SSH_TIMEOUT_SECS)))
        .ok();
    tcp.set_write_timeout(Some(Duration::from_secs(SSH_TIMEOUT_SECS)))
        .ok();

    /* =====================
       SSH SESSION
    ===================== */

    let mut sess = match Session::new() {
        Ok(s) => s,
        Err(_) => {
            return Ok(ssh_err(
                SshErrorKind::Internal,
                "Failed to create SSH session",
            ))
        }
    };

    sess.set_tcp_stream(tcp);

    if let Err(e) = sess.handshake() {
        return Ok(ssh_err(
            SshErrorKind::Network,
            format!("Handshake failed: {}", e),
        ));
    }

    if let Err(_) = sess.userauth_password(&username, &password) {
        return Ok(ssh_err(SshErrorKind::Auth, "SSH authentication failed"));
    }

    if !sess.authenticated() {
        return Ok(ssh_err(SshErrorKind::Auth, "SSH authentication failed"));
    }

    /* =====================
       EXEC COMMAND
    ===================== */

    let mut channel = match sess.channel_session() {
        Ok(c) => c,
        Err(e) => return Ok(ssh_err(SshErrorKind::Internal, e.to_string())),
    };

    if let Err(e) = channel.exec(&command) {
        return Ok(ssh_err(SshErrorKind::Command, e.to_string()));
    }

    let mut output = Vec::new();
    let mut buffer = [0u8; 1024];

    loop {
        match channel.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                output.extend_from_slice(&buffer[..n]);
                if output.len() > MAX_OUTPUT_BYTES {
                    channel.close().ok();
                    return Ok(ssh_err(
                        SshErrorKind::Command,
                        "Command output exceeded limit",
                    ));
                }
            }
            Err(e) => {
                return Ok(ssh_err(SshErrorKind::Command, e.to_string()));
            }
        }
    }

    channel.wait_close().ok();

    let exit_code = channel.exit_status().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output).to_string();

    Ok(SshResponse::Ok {
        ok: true,
        data: SshExecResult {
            success: exit_code == 0,
            exit_code,
            stdout,
        },
    })
}
