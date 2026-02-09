#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod protocol;
mod ws_commands;
mod ws_server;

use tauri::Manager;
use tauri_plugin_store::StoreExt;

use commands::{clip_to_midi_file, generate_midi, get_playback_status, play_midi, stop_midi};
use ws_commands::{get_clients, rename_client, send_clip_to_client};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_drag::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            // Restore always-on-top before the window is shown
            if let Ok(store) = app.store("settings.json") {
                if let Some(val) = store.get("alwaysOnTop") {
                    if val.as_bool().unwrap_or(false) {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.set_always_on_top(true);
                        }
                    }
                }
            }

            let server = ws_server::WsServer::start(9850);
            app.manage(server);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            generate_midi,
            play_midi,
            stop_midi,
            get_playback_status,
            clip_to_midi_file,
            get_clients,
            send_clip_to_client,
            rename_client,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
