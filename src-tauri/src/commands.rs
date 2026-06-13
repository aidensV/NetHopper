use crate::db::Db;
use serde::Serialize;
/* =========================
MODELS
========================= */
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
    pub password_id: Option<i64>,
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
            "SELECT id, name, host, port, username, auth_type, group_id,password_id
             FROM hosts WHERE group_id = ?",
            vec![id],
        ),
        None => (
            "SELECT id, name, host, port, username, auth_type, group_id,password_id
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
                password_id: r.get(7)?,
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
    password_id: i64,
    auth_type: String,
    group_id: Option<i64>,
    db: tauri::State<Db>,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|_| "DB lock failed")?;

    conn.execute(
        "INSERT INTO hosts (name, host, port, username, password_id, auth_type, group_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![name, host, port, username, password_id, auth_type, group_id],
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

#[tauri::command]
pub fn update_host(
    id: i64,
    name: String,
    host: String,
    port: i64,
    username: String,
    password_id: i64,
    auth_type: String,
    group_id: Option<i64>,
    db: tauri::State<Db>,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|_| "DB lock failed")?;

    let rows = conn.execute(
        "UPDATE hosts
         SET name = ?, host = ?, port = ?, username = ?, password_id = ?, auth_type = ?, group_id = ?
         WHERE id = ?",
        rusqlite::params![name, host, port, username, password_id, auth_type, group_id, id],
    )
    .map_err(|e| e.to_string())?;

    println!("update_host: {} row(s) affected for id={}", rows, id);

    if rows == 0 {
        return Err(format!("Host with id={} not found", id));
    }

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
