use rusqlite::Connection;
use std::sync::Mutex;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};


pub struct Db {
    pub conn: Mutex<Connection>,
}

pub fn init_db(app: &AppHandle) -> Db {
    let app_dir = app
        .path()
        .app_data_dir()
        .expect("cannot get app data dir");

    // pastikan folder ada
    fs::create_dir_all(&app_dir)
        .expect("failed to create app data dir");

    let mut db_path = PathBuf::from(app_dir);
    db_path.push("nethopper.db");

    let conn = Connection::open(&db_path)
        .expect("Failed to open database");

    conn.execute_batch(
        include_str!("../schema.sql")
    ).expect("Failed to initialize schema");

    Db {
        conn: Mutex::new(conn),
    }
}
