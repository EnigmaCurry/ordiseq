use crate::protocol::MidiClip;
use crate::ws_server::{ClientInfo, WsServer};
use tauri::State;

#[tauri::command]
pub fn get_clients(server: State<WsServer>) -> Vec<ClientInfo> {
    server.get_clients()
}

#[tauri::command]
pub fn send_clip_to_client(
    server: State<WsServer>,
    client_id: u64,
    clip: MidiClip,
) -> Result<(), String> {
    let clip_id = format!("clip-{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis());
    server.send_clip(client_id, clip_id, clip)
}

#[tauri::command]
pub fn rename_client(
    server: State<WsServer>,
    client_id: u64,
    name: String,
) -> Result<(), String> {
    server.rename_client(client_id, &name)
}
