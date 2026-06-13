mod commands;
mod db;
// mod ssh_stream;
mod crypto;
mod password_manager;
mod ssh_stream_xterm;
mod tunnel;

use commands::*;
use db::init_db;
use password_manager::*;
use ssh_stream_xterm::*;
use tauri::Manager;
use tunnel::*;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let db = init_db(&app.handle());
            app.manage(db);
            Ok(())
        })
        .manage(TunnelState::new())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            // GROUP
            list_groups_by_parent,
            create_group,
            update_group,
            rename_group,
            delete_group,
            // PASSWORD
            list_passwords,
            create_password,
            update_password,
            delete_password,
            copy_password,
            // HOST
            list_hosts_by_group,
            create_host,
            update_host,
            delete_host,
            // SSH (STREAMING)
            ssh_exec_start,
            ssh_exec_cancel,
            ssh_exec_input,
            // TUNNEL
            list_tunnels,
            create_tunnel,
            update_tunnel,
            delete_tunnel,
            start_tunnel,
            stop_tunnel,
            list_active_tunnels,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
