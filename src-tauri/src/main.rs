// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state;

use state::AppState;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::ports::scan_web_services,
            commands::engine::check_engine_status,
            commands::engine::download_engine,
            commands::tunnel::start_tunnel,
            commands::tunnel::stop_tunnel,
            commands::tunnel::stop_all_tunnels
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } | tauri::WindowEvent::Destroyed = event {
                let state = window.state::<AppState>();
                tauri::async_runtime::block_on(async {
                    state.kill_all_processes().await;
                });
            }
        })
        .run(tauri::generate_context!())
        .expect("运行 LocalShare 桌面应用时发生错误");
}
