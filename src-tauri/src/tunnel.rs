use crate::db::Db;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use ssh2::Session;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tunnel {
    pub id: i64,
    pub name: String,
    #[serde(rename = "type")]
    pub tunnel_type: String,
    pub host_id: i64,
    pub local_port: i64,
    pub remote_host: Option<String>,
    pub remote_port: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTunnelPayload {
    pub name: String,
    #[serde(rename = "type")]
    pub tunnel_type: String,
    pub host_id: i64,
    pub local_port: i64,
    pub remote_host: Option<String>,
    pub remote_port: Option<i64>,
}

// ── Active tunnel state ───────────────────────────────────────────────────────

pub struct TunnelHandle {
    pub stop_tx: std::sync::mpsc::Sender<()>,
}

pub struct TunnelState(pub Mutex<HashMap<i64, TunnelHandle>>);

impl TunnelState {
    pub fn new() -> Self {
        TunnelState(Mutex::new(HashMap::new()))
    }
}

// ── Session Pool ──────────────────────────────────────────────────────────────
// Setiap koneksi dapat session SSH sendiri dari pool.
// Setelah selesai, session dikembalikan ke pool untuk dipakai lagi.

struct SessionPool {
    sessions: Mutex<Vec<Session>>,
    ssh_host: String,
    ssh_port: i64,
    ssh_user: String,
    ssh_password: String,
}

impl SessionPool {
    fn new(host: String, port: i64, user: String, password: String) -> Self {
        SessionPool {
            sessions: Mutex::new(Vec::new()),
            ssh_host: host,
            ssh_port: port,
            ssh_user: user,
            ssh_password: password,
        }
    }
    fn is_session_alive(sess: &Session) -> bool {
        // Cek apakah TCP stream underlying masih oke
        // ssh2 expose method `authenticated()` yang baca internal state tanpa network call
        sess.authenticated()
    }

    // Ambil session dari pool, atau buat baru kalau pool kosong
    fn acquire(&self) -> Result<Session, String> {
        {
            let mut pool = self.sessions.lock().unwrap();
            while let Some(sess) = pool.pop() {
                if Self::is_session_alive(&sess) {
                    return Ok(sess);
                }
                eprintln!("[Pool] Session mati, dibuang");
            }
        }

        // Retry dengan backoff
        let mut last_err = String::new();
        for attempt in 0..3 {
            thread::sleep(std::time::Duration::from_millis(150 * (attempt + 1) as u64));
            match create_ssh_session(
                &self.ssh_host,
                self.ssh_port,
                &self.ssh_user,
                &self.ssh_password,
            ) {
                Ok(s) => return Ok(s),
                Err(e) => {
                    last_err = e;
                }
            }
        }
        Err(last_err)
    }

    // Kembalikan session ke pool setelah selesai dipakai
    fn release(&self, mut sess: Session, healthy: bool) {
        if !healthy || !Self::is_session_alive(&sess) {
            return; // drop saja, jangan kembalikan ke pool
        }
        sess.set_blocking(true);
        let mut pool = self.sessions.lock().unwrap();
        if pool.len() < 6 {
            pool.push(sess);
        }
    }
}

// ── CRUD Commands ─────────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_tunnels(db: tauri::State<Db>) -> Result<Vec<Tunnel>, String> {
    let conn = db.conn.lock().map_err(|_| "DB lock failed")?;

    let mut stmt = conn
        .prepare(
            "SELECT id, name, type, host_id, local_port, remote_host, remote_port, created_at
             FROM tunnels ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let tunnels = stmt
        .query_map([], |row| {
            Ok(Tunnel {
                id: row.get(0)?,
                name: row.get(1)?,
                tunnel_type: row.get(2)?,
                host_id: row.get(3)?,
                local_port: row.get(4)?,
                remote_host: row.get(5)?,
                remote_port: row.get(6)?,
                created_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(tunnels)
}

#[tauri::command]
pub fn create_tunnel(payload: CreateTunnelPayload, db: tauri::State<Db>) -> Result<(), String> {
    if payload.tunnel_type == "local" {
        if payload.remote_host.is_none() || payload.remote_port.is_none() {
            return Err(
                "Local port forwarding membutuhkan remote_host dan remote_port".to_string(),
            );
        }
    }

    let conn = db.conn.lock().map_err(|_| "DB lock failed")?;

    conn.execute(
        "INSERT INTO tunnels (name, type, host_id, local_port, remote_host, remote_port)
         VALUES (?, ?, ?, ?, ?, ?)",
        params![
            payload.name,
            payload.tunnel_type,
            payload.host_id,
            payload.local_port,
            payload.remote_host,
            payload.remote_port,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn update_tunnel(
    id: i64,
    payload: CreateTunnelPayload,
    db: tauri::State<Db>,
    tunnel_state: tauri::State<TunnelState>,
) -> Result<(), String> {
    let state = tunnel_state.0.lock().map_err(|_| "State lock failed")?;
    if state.contains_key(&id) {
        return Err("Stop tunnel dulu sebelum edit".to_string());
    }
    drop(state);

    if payload.tunnel_type == "local" {
        if payload.remote_host.is_none() || payload.remote_port.is_none() {
            return Err(
                "Local port forwarding membutuhkan remote_host dan remote_port".to_string(),
            );
        }
    }

    let conn = db.conn.lock().map_err(|_| "DB lock failed")?;

    let rows = conn
        .execute(
            "UPDATE tunnels SET name=?, type=?, host_id=?, local_port=?, remote_host=?, remote_port=?
             WHERE id=?",
            params![
                payload.name,
                payload.tunnel_type,
                payload.host_id,
                payload.local_port,
                payload.remote_host,
                payload.remote_port,
                id,
            ],
        )
        .map_err(|e| e.to_string())?;

    if rows == 0 {
        return Err(format!("Tunnel id={} tidak ditemukan", id));
    }

    Ok(())
}

#[tauri::command]
pub fn delete_tunnel(
    id: i64,
    db: tauri::State<Db>,
    tunnel_state: tauri::State<TunnelState>,
) -> Result<(), String> {
    stop_tunnel_internal(id, &tunnel_state)?;

    let conn = db.conn.lock().map_err(|_| "DB lock failed")?;
    conn.execute("DELETE FROM tunnels WHERE id=?", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

// ── Start / Stop ──────────────────────────────────────────────────────────────

#[tauri::command]
pub fn start_tunnel(
    id: i64,
    db: tauri::State<Db>,
    tunnel_state: tauri::State<TunnelState>,
) -> Result<(), String> {
    {
        let state = tunnel_state.0.lock().map_err(|_| "State lock failed")?;
        if state.contains_key(&id) {
            return Err("Tunnel sudah running".to_string());
        }
    }

    let (tunnel, ssh_host, ssh_port, ssh_user, ssh_password) = {
        let conn = db.conn.lock().map_err(|_| "DB lock failed")?;

        let tunnel = conn
            .query_row(
                "SELECT id, name, type, host_id, local_port, remote_host, remote_port, created_at
                 FROM tunnels WHERE id=?",
                params![id],
                |row| {
                    Ok(Tunnel {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        tunnel_type: row.get(2)?,
                        host_id: row.get(3)?,
                        local_port: row.get(4)?,
                        remote_host: row.get(5)?,
                        remote_port: row.get(6)?,
                        created_at: row.get(7)?,
                    })
                },
            )
            .map_err(|e| e.to_string())?;

        let (ssh_host, ssh_port, ssh_user, encrypted_pw): (String, i64, String, String) = conn
            .query_row(
                "SELECT h.host, h.port, h.username, p.password
                 FROM hosts h
                 LEFT JOIN passwords p ON h.password_id = p.id
                 WHERE h.id=?",
                params![tunnel.host_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .map_err(|e| e.to_string())?;

        let password = crate::crypto::decrypt(&encrypted_pw)?;
        (tunnel, ssh_host, ssh_port, ssh_user, password)
    };

    let (stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();
    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<(), String>>();

    let tunnel_clone = tunnel.clone();

    thread::spawn(move || {
        if let Err(e) = run_tunnel(
            tunnel_clone,
            ssh_host,
            ssh_port,
            ssh_user,
            ssh_password,
            stop_rx,
            ready_tx,
        ) {
            eprintln!("[Tunnel {}] Error: {}", id, e);
        }
    });

    match ready_rx.recv() {
        Ok(Ok(())) => {
            let mut state = tunnel_state.0.lock().map_err(|_| "State lock failed")?;
            state.insert(id, TunnelHandle { stop_tx });
            Ok(())
        }
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Tunnel gagal start".to_string()),
    }
}

#[tauri::command]
pub fn stop_tunnel(id: i64, tunnel_state: tauri::State<TunnelState>) -> Result<(), String> {
    stop_tunnel_internal(id, &tunnel_state)
}

fn stop_tunnel_internal(id: i64, tunnel_state: &TunnelState) -> Result<(), String> {
    let mut state = tunnel_state.0.lock().map_err(|_| "State lock failed")?;
    if let Some(handle) = state.remove(&id) {
        let _ = handle.stop_tx.send(());
    }
    Ok(())
}

#[tauri::command]
pub fn list_active_tunnels(tunnel_state: tauri::State<TunnelState>) -> Result<Vec<i64>, String> {
    let state = tunnel_state.0.lock().map_err(|_| "State lock failed")?;
    Ok(state.keys().cloned().collect())
}

// ── SSH Session Helper ────────────────────────────────────────────────────────

fn create_ssh_session(
    ssh_host: &str,
    ssh_port: i64,
    ssh_user: &str,
    ssh_password: &str,
) -> Result<Session, String> {
    let tcp =
        TcpStream::connect(format!("{}:{}", ssh_host, ssh_port)).map_err(|e| e.to_string())?;

    let mut sess = Session::new().map_err(|e| e.to_string())?;
    sess.set_tcp_stream(tcp);
    sess.handshake().map_err(|e| e.to_string())?;
    sess.userauth_password(ssh_user, ssh_password)
        .map_err(|e| e.to_string())?;

    if !sess.authenticated() {
        return Err("SSH authentication failed".to_string());
    }

    Ok(sess)
}

// ── Core tunnel logic ─────────────────────────────────────────────────────────

fn run_tunnel(
    tunnel: Tunnel,
    ssh_host: String,
    ssh_port: i64,
    ssh_user: String,
    ssh_password: String,
    stop_rx: std::sync::mpsc::Receiver<()>,
    ready_tx: std::sync::mpsc::Sender<Result<(), String>>,
) -> Result<(), String> {
    let result = run_tunnel_inner(
        tunnel,
        ssh_host,
        ssh_port,
        ssh_user,
        ssh_password,
        stop_rx,
        ready_tx.clone(),
    );

    if let Err(e) = &result {
        let _ = ready_tx.send(Err(e.clone()));
    }

    result
}

fn run_tunnel_inner(
    tunnel: Tunnel,
    ssh_host: String,
    ssh_port: i64,
    ssh_user: String,
    ssh_password: String,
    stop_rx: std::sync::mpsc::Receiver<()>,
    ready_tx: std::sync::mpsc::Sender<Result<(), String>>,
) -> Result<(), String> {
    // Buat session pool dengan beberapa session siap pakai
    let pool = Arc::new(SessionPool::new(
        ssh_host.clone(),
        ssh_port,
        ssh_user.clone(),
        ssh_password.clone(),
    ));

    // Pre-fill pool dengan 4 session
    {
        let mut p = pool.sessions.lock().unwrap();
        for i in 0..3 {
            if i > 0 {
                thread::sleep(std::time::Duration::from_millis(300));
            }
            match create_ssh_session(&ssh_host, ssh_port, &ssh_user, &ssh_password) {
                Ok(s) => p.push(s),
                Err(e) => eprintln!("[Pool] Init session {} gagal: {}", i, e),
            }
        }
        if p.is_empty() {
            return Err("Tidak bisa buat SSH session".to_string());
        }
    }

    // Background refill thread
    {
        let pool_ref = Arc::clone(&pool);
        let ssh_host = ssh_host.clone();
        let ssh_user = ssh_user.clone();
        let ssh_password = ssh_password.clone();

        thread::spawn(move || {
            loop {
                thread::sleep(std::time::Duration::from_secs(2));

                let current_size = pool_ref.sessions.lock().unwrap().len();
                let target = 3;

                if current_size < target {
                    let needed = target - current_size;
                    for _ in 0..needed {
                        match create_ssh_session(&ssh_host, ssh_port, &ssh_user, &ssh_password) {
                            Ok(s) => {
                                let mut p = pool_ref.sessions.lock().unwrap();
                                if p.len() < 6 {
                                    p.push(s);
                                }
                            }
                            Err(_) => break, // SSH server mungkin busy, coba lagi nanti
                        }
                        thread::sleep(std::time::Duration::from_millis(300));
                    }
                }
            }
        });
    }

    let listener =
        TcpListener::bind(format!("127.0.0.1:{}", tunnel.local_port)).map_err(|e| e.to_string())?;
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    let _ = ready_tx.send(Ok(()));

    println!("[Tunnel] Listening on 127.0.0.1:{}", tunnel.local_port);

    loop {
        if stop_rx.try_recv().is_ok() {
            println!("[Tunnel] Stop signal received");
            break;
        }

        match listener.accept() {
            Ok((client, _)) => {
                client.set_nonblocking(false).map_err(|e| e.to_string())?;

                let tunnel = tunnel.clone();
                let pool = Arc::clone(&pool);

                thread::spawn(move || {
                    let sess = match pool.acquire() {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("[Pool] Acquire failed: {}", e);
                            return;
                        }
                    };

                    match handle_client(client, sess, &tunnel, &pool) {
                        Ok(_) => {}
                        Err(e) => {
                            // Error ini = session mungkin masih oke, sudah di-release di dalam handle_client
                            let silent = e.contains("Broken pipe")
                || e.contains("EOF")
                || e.contains("os error 10054")  // connection reset by peer
                || e.contains("os error 104");
                            if !silent {
                                eprintln!("[Tunnel] Client error: {}", e);
                            }
                        }
                    }
                });
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(e) => {
                eprintln!("[Tunnel] Accept error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

// ── Per-client handler ────────────────────────────────────────────────────────

fn handle_client(
    client: TcpStream,
    sess: Session,
    tunnel: &Tunnel,
    pool: &Arc<SessionPool>,
) -> Result<(), String> {
    match tunnel.tunnel_type.as_str() {
        "local" => {
            let remote_host = tunnel.remote_host.as_deref().unwrap_or("").to_string();
            let remote_port = tunnel.remote_port.unwrap_or(0) as u16;

            let channel = match sess.channel_direct_tcpip(&remote_host, remote_port, None) {
                Ok(ch) => ch,
                Err(e) => {
                    pool.release(sess, true);
                    return Err(e.to_string());
                }
            };

            // Set non-blocking SETELAH channel dibuka, SEBELUM forward
            sess.set_blocking(false);
            let result = forward_traffic(client, channel);
            let healthy = !result
                .as_ref()
                .err()
                .map(|e| e.contains("10053") || e.contains("10054") || e.contains("aborted"))
                .unwrap_or(false);
            pool.release(sess, healthy);
            result
        }
        "socks5" => handle_socks5(client, sess, pool),
        _ => {
            pool.release(sess, false);
            Err("Unknown tunnel type".to_string())
        }
    }
}

// ── SOCKS5 ────────────────────────────────────────────────────────────────────

fn handle_socks5(
    mut client: TcpStream,
    sess: Session,
    pool: &Arc<SessionPool>,
) -> Result<(), String> {
    client.set_nonblocking(false).map_err(|e| e.to_string())?;

    let mut buf = [0u8; 256];

    // 1. Baca greeting dari client
    client
        .read_exact(&mut buf[..2])
        .map_err(|e| e.to_string())?;
    let nmethods = buf[1] as usize;
    if nmethods > 0 {
        client
            .read_exact(&mut buf[..nmethods])
            .map_err(|e| e.to_string())?;
    }

    // 2. Pilih no-auth
    client.write_all(&[0x05, 0x00]).map_err(|e| e.to_string())?;

    // 3. Baca request
    client
        .read_exact(&mut buf[..4])
        .map_err(|e| e.to_string())?;
    // buf[1] = cmd (0x01 = CONNECT)
    let atyp = buf[3];

    let target_host = match atyp {
        0x01 => {
            // IPv4
            client
                .read_exact(&mut buf[..4])
                .map_err(|e| e.to_string())?;
            format!("{}.{}.{}.{}", buf[0], buf[1], buf[2], buf[3])
        }
        0x03 => {
            // Domain
            client
                .read_exact(&mut buf[..1])
                .map_err(|e| e.to_string())?;
            let len = buf[0] as usize;
            client
                .read_exact(&mut buf[..len])
                .map_err(|e| e.to_string())?;
            String::from_utf8_lossy(&buf[..len]).to_string()
        }
        0x04 => {
            // IPv6 — baca habis dulu baru tolak
            client
                .read_exact(&mut buf[..16])
                .map_err(|e| e.to_string())?;
            client
                .read_exact(&mut buf[..2])
                .map_err(|e| e.to_string())?;
            let _ = client.write_all(&[0x05, 0x08, 0x00, 0x01, 0, 0, 0, 0, 0, 0]);
            pool.release(sess, false);
            return Err("IPv6 not supported".to_string());
        }
        _ => {
            let _ = client.write_all(&[0x05, 0x08, 0x00, 0x01, 0, 0, 0, 0, 0, 0]);
            pool.release(sess, false);
            return Err(format!("Unsupported atyp: {}", atyp));
        }
    };

    let mut port_buf = [0u8; 2];
    client
        .read_exact(&mut port_buf)
        .map_err(|e| e.to_string())?;
    let target_port = u16::from_be_bytes(port_buf);

    println!("[SOCKS5] -> {}:{}", target_host, target_port);

    // 4. Buka SSH channel ke target
    let channel = match sess.channel_direct_tcpip(&target_host, target_port, None) {
        Ok(ch) => ch,
        Err(e) => {
            // Kirim failure reply ke browser
            let _ = client.write_all(&[0x05, 0x04, 0x00, 0x01, 0, 0, 0, 0, 0, 0]);
            pool.release(sess, true);
            return Err(format!(
                "channel_direct_tcpip {}:{} => {}",
                target_host, target_port, e
            ));
        }
    };

    // 5. Balas sukses ke browser
    client
        .write_all(&[0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
        .map_err(|e| e.to_string())?;

    // 6. Set non-blocking SETELAH channel dibuka
    sess.set_blocking(false);

    // 7. Forward traffic
    let result = forward_traffic(client, channel);
    let healthy = !result
        .as_ref()
        .err()
        .map(|e| e.contains("10053") || e.contains("10054") || e.contains("aborted"))
        .unwrap_or(false);
    pool.release(sess, healthy);
    result
}

// ── Traffic forwarding ────────────────────────────────────────────────────────

fn is_would_block_error(e: &str) -> bool {
    e.contains("Would block") || e.contains("Session(-37)")
}

fn forward_traffic(mut client: TcpStream, mut channel: ssh2::Channel) -> Result<(), String> {
    // Client non-blocking, channel sudah non-blocking via sess.set_blocking(false)
    client.set_nonblocking(true).map_err(|e| e.to_string())?;

    let mut client_buf = [0u8; 16384];
    let mut channel_buf = [0u8; 16384];
    let mut upload_done = false;

    loop {
        let mut did_work = false;

        // ── Client → SSH channel ──────────────────────────────────
        if !upload_done {
            match client.read(&mut client_buf) {
                Ok(0) => {
                    upload_done = true;
                    let _ = channel.send_eof();
                }
                Ok(n) => {
                    if channel.write_all(&client_buf[..n]).is_err() {
                        break;
                    }
                    let _ = channel.flush();
                    did_work = true;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(_) => {
                    let _ = channel.send_eof();
                    break;
                }
            }
        }

        // ── SSH channel → Client ──────────────────────────────────
        if channel.eof() {
            break;
        }

        match channel.read(&mut channel_buf) {
            Ok(0) => {}
            Ok(n) => {
                // Client non-blocking: retry kalau WouldBlock
                let mut written = 0;
                while written < n {
                    match client.write(&channel_buf[written..n]) {
                        Ok(w) => {
                            written += w;
                            did_work = true;
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(std::time::Duration::from_millis(1));
                        }
                        Err(_) => return Ok(()),
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => {
                let msg = e.to_string();
                if !is_would_block_error(&msg) {
                    break;
                }
            }
        }

        if !did_work {
            thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    let _ = channel.close();
    let _ = channel.wait_close();

    Ok(())
}
