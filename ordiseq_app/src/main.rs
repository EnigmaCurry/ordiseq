#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;

use commands::{generate_midi, get_playback_status, play_midi, stop_midi};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_drag::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            generate_midi,
            play_midi,
            stop_midi,
            get_playback_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
