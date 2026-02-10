use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use tungstenite::Message;

use crate::protocol::{AppMessage, MidiClip, PluginMessage};

pub type ClientId = u64;

#[derive(Clone, Debug, serde::Serialize)]
pub struct ClientInfo {
    pub id: ClientId,
    pub name: String,
    pub version: String,
    pub bpm: f32,
    pub playing: bool,
    pub connected: bool,
    pub program: u8,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct SyncStateInfo {
    pub source_client_id: Option<ClientId>,
    pub beat_position: f64,
    pub bpm: f64,
    pub playing: bool,
    pub received_at_ms: u64,
}

struct SyncState {
    beat_position: f64,
    bpm: f64,
    playing: bool,
    received_at_ms: u64,
}

impl Default for SyncState {
    fn default() -> Self {
        Self {
            beat_position: 0.0,
            bpm: 120.0,
            playing: false,
            received_at_ms: 0,
        }
    }
}

struct ClientState {
    info: ClientInfo,
    sender: std::sync::mpsc::Sender<String>,
}

pub struct WsServer {
    clients: Arc<RwLock<HashMap<ClientId, ClientState>>>,
    next_id: Arc<AtomicU64>,
    sync_source: Arc<RwLock<Option<ClientId>>>,
    sync_state: Arc<RwLock<SyncState>>,
    port: RwLock<u16>,
    stop_flag: Arc<AtomicBool>,
    accept_handle: Mutex<Option<std::thread::JoinHandle<()>>>,
}

impl WsServer {
    pub fn start(port: u16) -> Self {
        let clients: Arc<RwLock<HashMap<ClientId, ClientState>>> =
            Arc::new(RwLock::new(HashMap::new()));
        let next_id = Arc::new(AtomicU64::new(1));
        let sync_source = Arc::new(RwLock::new(None));
        let sync_state = Arc::new(RwLock::new(SyncState::default()));
        let stop_flag = Arc::new(AtomicBool::new(false));

        let handle = {
            let clients = clients.clone();
            let next_id = next_id.clone();
            let sync_source = sync_source.clone();
            let sync_state = sync_state.clone();
            let stop_flag = stop_flag.clone();
            std::thread::spawn(move || {
                Self::accept_loop(port, clients, next_id, sync_source, sync_state, stop_flag);
            })
        };

        Self {
            clients,
            next_id,
            sync_source,
            sync_state,
            port: RwLock::new(port),
            stop_flag,
            accept_handle: Mutex::new(Some(handle)),
        }
    }

    fn accept_loop(
        port: u16,
        clients: Arc<RwLock<HashMap<ClientId, ClientState>>>,
        next_id: Arc<AtomicU64>,
        sync_source: Arc<RwLock<Option<ClientId>>>,
        sync_state: Arc<RwLock<SyncState>>,
        stop_flag: Arc<AtomicBool>,
    ) {
        let addr = format!("127.0.0.1:{port}");
        let listener = match TcpListener::bind(&addr) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("WsServer: failed to bind to {addr}: {e}");
                return;
            }
        };
        listener.set_nonblocking(true).ok();
        eprintln!("WsServer: listening on {addr}");

        loop {
            if stop_flag.load(Ordering::Relaxed) {
                eprintln!("WsServer: accept loop stopping on port {port}");
                break;
            }

            match listener.accept() {
                Ok((stream, _)) => {
                    stream.set_nonblocking(false).ok();
                    let clients = clients.clone();
                    let sync_source = sync_source.clone();
                    let sync_state = sync_state.clone();
                    let stop_flag = stop_flag.clone();
                    let client_id = next_id.fetch_add(1, Ordering::Relaxed);

                    std::thread::spawn(move || {
                        Self::handle_client(stream, client_id, clients, sync_source, sync_state, stop_flag);
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(_) => {
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        }
    }

    fn handle_client(
        stream: std::net::TcpStream,
        client_id: ClientId,
        clients: Arc<RwLock<HashMap<ClientId, ClientState>>>,
        sync_source: Arc<RwLock<Option<ClientId>>>,
        sync_state: Arc<RwLock<SyncState>>,
        stop_flag: Arc<AtomicBool>,
    ) {
        let mut ws = match tungstenite::accept(stream) {
            Ok(ws) => ws,
            Err(_) => return,
        };

        // Set read timeout for non-blocking outbound check
        let _ = ws.get_ref().set_read_timeout(Some(Duration::from_millis(50)));

        // Create outbound channel for this client
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();

        // Register client with placeholder info
        {
            let mut map = clients.write().unwrap();
            map.insert(
                client_id,
                ClientState {
                    info: ClientInfo {
                        id: client_id,
                        name: format!("Plugin #{client_id}"),
                        version: String::new(),
                        bpm: 0.0,
                        playing: false,
                        connected: true,
                        program: 0,
                    },
                    sender: out_tx,
                },
            );
        }

        eprintln!("WsServer: client {client_id} connected");

        // Main loop
        loop {
            if stop_flag.load(Ordering::Relaxed) {
                eprintln!("WsServer: client {client_id} handler stopping (server restart)");
                let _ = ws.close(None);
                Self::remove_client(&clients, client_id, &sync_source, &sync_state);
                return;
            }

            // Drain outbound messages
            while let Ok(msg) = out_rx.try_recv() {
                if ws.send(Message::Text(msg)).is_err() {
                    Self::remove_client(&clients, client_id, &sync_source, &sync_state);
                    return;
                }
            }

            // Read inbound
            match ws.read() {
                Ok(Message::Text(text)) => {
                    if let Ok(plugin_msg) = serde_json::from_str::<PluginMessage>(&text) {
                        match plugin_msg {
                            PluginMessage::Register { name, version } => {
                                let mut map = clients.write().unwrap();
                                let unique_name = Self::deduplicate_name(&name, client_id, &map);
                                if let Some(client) = map.get_mut(&client_id) {
                                    client.info.version = version;
                                    if unique_name != name {
                                        let rename_msg = AppMessage::Rename { name: unique_name.clone() };
                                        if let Ok(json) = serde_json::to_string(&rename_msg) {
                                            let _ = client.sender.send(json);
                                        }
                                    }
                                    client.info.name = unique_name;
                                    eprintln!("WsServer: client {client_id} registered as \"{}\"", client.info.name);

                                    // Auto-assign as sync source if none exists
                                    let mut source = sync_source.write().unwrap();
                                    if source.is_none() {
                                        *source = Some(client_id);
                                        let start_msg = AppMessage::StartSync;
                                        if let Ok(json) = serde_json::to_string(&start_msg) {
                                            let _ = client.sender.send(json);
                                        }
                                        eprintln!("WsServer: auto-assigned client {client_id} as sync source");
                                    }
                                }
                            }
                            PluginMessage::Transport { bpm, playing, program } => {
                                let mut map = clients.write().unwrap();
                                if let Some(client) = map.get_mut(&client_id) {
                                    client.info.bpm = bpm;
                                    client.info.playing = playing;
                                    client.info.program = program;
                                }
                                // If the sync source reports transport stopped, update sync state
                                // so the frontend stops animating even if the final TimeSync is lost.
                                if !playing {
                                    let is_source = sync_source.read().unwrap()
                                        .map_or(false, |id| id == client_id);
                                    if is_source {
                                        let mut state = sync_state.write().unwrap();
                                        if state.playing {
                                            state.playing = false;
                                            state.received_at_ms = std::time::SystemTime::now()
                                                .duration_since(std::time::UNIX_EPOCH)
                                                .unwrap_or_default()
                                                .as_millis() as u64;
                                        }
                                    }
                                }
                            }
                            PluginMessage::ClipAck { clip_id } => {
                                eprintln!("WsServer: client {client_id} acked clip \"{clip_id}\"");
                            }
                            PluginMessage::TimeSync { beat_position, bpm, playing } => {
                                let is_source = sync_source.read().unwrap().map_or(false, |id| id == client_id);
                                if is_source {
                                    let now_ms = std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_millis() as u64;
                                    let mut state = sync_state.write().unwrap();
                                    state.beat_position = beat_position;
                                    state.bpm = bpm;
                                    state.playing = playing;
                                    state.received_at_ms = now_ms;
                                }
                            }
                        }
                    }
                }
                Ok(Message::Ping(data)) => {
                    let _ = ws.send(Message::Pong(data));
                }
                Ok(Message::Close(_)) => {
                    eprintln!("WsServer: client {client_id} closed connection");
                    Self::remove_client(&clients, client_id, &sync_source, &sync_state);
                    return;
                }
                Err(tungstenite::Error::Io(ref e))
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    // Normal timeout
                }
                Err(_) => {
                    eprintln!("WsServer: client {client_id} disconnected");
                    Self::remove_client(&clients, client_id, &sync_source, &sync_state);
                    return;
                }
                _ => {}
            }
        }
    }

    /// Given a desired name and the current clients map (already write-locked),
    /// return a unique name. If "Blessed Raspberry" is taken, returns
    /// "Blessed Raspberry 2", then "Blessed Raspberry 3", etc.
    fn deduplicate_name(
        desired: &str,
        exclude_id: ClientId,
        map: &HashMap<ClientId, ClientState>,
    ) -> String {
        let existing: std::collections::HashSet<&str> = map
            .iter()
            .filter(|(id, _)| **id != exclude_id)
            .map(|(_, cs)| cs.info.name.as_str())
            .collect();

        // Strip any existing numeric suffix to get the base name
        // e.g. "X Y 2" -> "X Y", "X Y" -> "X Y"
        let base = match desired.rsplit_once(' ') {
            Some((prefix, suffix)) if suffix.chars().all(|c| c.is_ascii_digit()) => prefix,
            _ => desired,
        };

        if !existing.contains(desired) {
            return desired.to_string();
        }

        let mut suffix = 2u32;
        loop {
            let candidate = format!("{base} {suffix}");
            if !existing.contains(candidate.as_str()) {
                return candidate;
            }
            suffix += 1;
        }
    }

    fn remove_client(
        clients: &Arc<RwLock<HashMap<ClientId, ClientState>>>,
        client_id: ClientId,
        sync_source: &Arc<RwLock<Option<ClientId>>>,
        sync_state: &Arc<RwLock<SyncState>>,
    ) {
        let mut map = clients.write().unwrap();
        map.remove(&client_id);

        // Auto-failover if this was the sync source
        let mut source = sync_source.write().unwrap();
        if *source == Some(client_id) {
            *source = None;
            *sync_state.write().unwrap() = SyncState::default();

            // Promote another client (prefer one that's playing)
            let new_source = map.iter()
                .find(|(_, c)| c.info.playing)
                .or_else(|| map.iter().next());
            if let Some((&new_id, new_client)) = new_source {
                *source = Some(new_id);
                let msg = AppMessage::StartSync;
                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = new_client.sender.send(json);
                }
                eprintln!("WsServer: sync source failover to client {new_id} ({})", new_client.info.name);
            }
        }
    }

    // -- Public API for Tauri commands --

    pub fn get_port(&self) -> u16 {
        *self.port.read().unwrap()
    }

    /// Restart the server on a new port. Disconnects all existing clients.
    pub fn restart(&self, new_port: u16) {
        // Signal all threads to stop
        self.stop_flag.store(true, Ordering::Relaxed);

        // Wait for accept loop to finish
        if let Some(handle) = self.accept_handle.lock().unwrap().take() {
            let _ = handle.join();
        }

        // At this point, stop_flag is true so client handlers will also exit.
        // Give them a moment to close their WebSockets.
        std::thread::sleep(Duration::from_millis(200));

        // Clear all state
        self.clients.write().unwrap().clear();
        *self.sync_source.write().unwrap() = None;
        *self.sync_state.write().unwrap() = SyncState::default();
        *self.port.write().unwrap() = new_port;

        // Reset stop flag and start new accept loop
        self.stop_flag.store(false, Ordering::Relaxed);

        let handle = {
            let clients = self.clients.clone();
            let next_id = self.next_id.clone();
            let sync_source = self.sync_source.clone();
            let sync_state = self.sync_state.clone();
            let stop_flag = self.stop_flag.clone();
            std::thread::spawn(move || {
                Self::accept_loop(new_port, clients, next_id, sync_source, sync_state, stop_flag);
            })
        };

        *self.accept_handle.lock().unwrap() = Some(handle);
        eprintln!("WsServer: restarted on port {new_port}");
    }

    pub fn get_clients(&self) -> Vec<ClientInfo> {
        let map = self.clients.read().unwrap();
        map.values().map(|c| c.info.clone()).collect()
    }

    pub fn send_clip(
        &self,
        client_id: ClientId,
        clip_id: String,
        clip: MidiClip,
        program: u8,
    ) -> Result<(), String> {
        let map = self.clients.read().unwrap();
        let client = map
            .get(&client_id)
            .ok_or_else(|| format!("Client {client_id} not found"))?;
        let msg = AppMessage::Clip { clip_id, clip, program };
        let json = serde_json::to_string(&msg).map_err(|e| e.to_string())?;
        client
            .sender
            .send(json)
            .map_err(|e| format!("Failed to send: {e}"))
    }

    pub fn send_play_mode(
        &self,
        client_id: ClientId,
        program: u8,
        note_trigger: bool,
    ) -> Result<(), String> {
        let map = self.clients.read().unwrap();
        let client = map
            .get(&client_id)
            .ok_or_else(|| format!("Client {client_id} not found"))?;
        let msg = AppMessage::PlayMode { program, note_trigger };
        let json = serde_json::to_string(&msg).map_err(|e| e.to_string())?;
        client
            .sender
            .send(json)
            .map_err(|e| format!("Failed to send: {e}"))
    }

    pub fn rename_client(
        &self,
        client_id: ClientId,
        name: &str,
    ) -> Result<(), String> {
        let mut map = self.clients.write().unwrap();
        let unique_name = Self::deduplicate_name(name, client_id, &map);
        let client = map
            .get_mut(&client_id)
            .ok_or_else(|| format!("Client {client_id} not found"))?;
        client.info.name = unique_name.clone();
        let msg = AppMessage::Rename {
            name: unique_name,
        };
        let json = serde_json::to_string(&msg).map_err(|e| e.to_string())?;
        client
            .sender
            .send(json)
            .map_err(|e| format!("Failed to send: {e}"))
    }

    pub fn set_sync_source(&self, client_id: ClientId) -> Result<(), String> {
        let map = self.clients.read().unwrap();

        // Stop current sync source
        {
            let mut source = self.sync_source.write().unwrap();
            if let Some(old_id) = *source {
                if let Some(old_client) = map.get(&old_id) {
                    let msg = AppMessage::StopSync;
                    if let Ok(json) = serde_json::to_string(&msg) {
                        let _ = old_client.sender.send(json);
                    }
                }
            }
            *source = Some(client_id);
        }

        // Start new sync source
        let client = map.get(&client_id)
            .ok_or_else(|| format!("Client {client_id} not found"))?;
        let msg = AppMessage::StartSync;
        let json = serde_json::to_string(&msg).map_err(|e| e.to_string())?;
        client.sender.send(json).map_err(|e| format!("Failed to send: {e}"))
    }

    pub fn clear_sync_source(&self) {
        let map = self.clients.read().unwrap();
        let mut source = self.sync_source.write().unwrap();
        if let Some(old_id) = source.take() {
            if let Some(old_client) = map.get(&old_id) {
                let msg = AppMessage::StopSync;
                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = old_client.sender.send(json);
                }
            }
        }
        *self.sync_state.write().unwrap() = SyncState::default();
    }

    pub fn get_sync_state(&self) -> SyncStateInfo {
        let source = self.sync_source.read().unwrap();
        let state = self.sync_state.read().unwrap();
        SyncStateInfo {
            source_client_id: *source,
            beat_position: state.beat_position,
            bpm: state.bpm,
            playing: state.playing,
            received_at_ms: state.received_at_ms,
        }
    }
}
