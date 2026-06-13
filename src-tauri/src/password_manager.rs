use crate::crypto;
use crate::db::Db;
use serde::Serialize;
use tauri_plugin_clipboard_manager::ClipboardExt;

#[derive(Serialize)]
pub struct Password {
    pub id: i64,
    pub name: String,
    pub created_at: String,
}

#[tauri::command]
pub fn list_passwords(db: tauri::State<Db>) -> Result<Vec<Password>, String> {
    let conn = db.conn.lock().map_err(|_| "DB lock failed")?;

    let mut stmt = conn
        .prepare("SELECT id, name, created_at FROM passwords")
        .map_err(|e| e.to_string())?;

    let passwords = stmt
        .query_map([], |row| {
            Ok(Password {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let passwords: Result<Vec<_>, _> = passwords.collect();
    passwords.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_password(name: String, password: String, db: tauri::State<Db>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|_| "DB lock failed")?;
    let encrypted = crypto::encrypt(&password)?;
    conn.execute(
        "INSERT INTO passwords (name, password) VALUES (?, ?)",
        rusqlite::params![name, encrypted],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn update_password(
    id: i64,
    name: String,
    password: Option<String>,
    db: tauri::State<Db>,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|_| "DB lock failed")?;

    match password {
        Some(pw) if !pw.trim().is_empty() => {
            // User isi password baru → encrypt dan update semua
            let encrypted = crypto::encrypt(&pw)?;
            conn.execute(
                "UPDATE passwords SET name = ?, password = ? WHERE id = ?",
                rusqlite::params![name, encrypted, id],
            )
            .map_err(|e| e.to_string())?;
        }
        _ => {
            // User kosongkan password → hanya update name
            conn.execute(
                "UPDATE passwords SET name = ? WHERE id = ?",
                rusqlite::params![name, id],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
pub fn delete_password(id: i64, db: tauri::State<Db>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|_| "DB lock failed")?;
    conn.execute("DELETE FROM passwords WHERE id = ?", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn copy_password(id: i64, db: tauri::State<Db>, app: tauri::AppHandle) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|_| "DB lock failed")?;

    let encrypted: String = conn
        .query_row(
            "SELECT password FROM passwords WHERE id = ?",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let plaintext = crypto::decrypt(&encrypted)?;

    // Tulis langsung ke clipboard dari Rust
    app.clipboard()
        .write_text(plaintext)
        .map_err(|e| e.to_string())?;

    Ok(())
}
