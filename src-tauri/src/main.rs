mod commands;
mod db;
// mod ssh_stream;
mod ssh_stream_xterm;

use commands::*;
use db::init_db;
// use ssh_stream::*;
use ssh_stream_xterm::*;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let db = init_db(&app.handle());
            app.manage(db);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // GROUP
            list_groups_by_parent,
            create_group,
            update_group,
            rename_group,
            delete_group,
            // HOST
            list_hosts_by_group,
            create_host,
            update_host,
            delete_host,
            // SSH (STREAMING)
            ssh_exec_start,
            // ssh_exec_input,
            // ssh_exec_resize,
            ssh_exec_cancel,
            ssh_exec_input
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
