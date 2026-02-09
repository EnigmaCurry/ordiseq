use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
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

struct ClientState {
    info: ClientInfo,
    sender: std::sync::mpsc::Sender<String>,
}

pub struct WsServer {
    clients: Arc<RwLock<HashMap<ClientId, ClientState>>>,
    _next_id: Arc<AtomicU64>,
}

impl WsServer {
    pub fn start(port: u16) -> Self {
        let clients: Arc<RwLock<HashMap<ClientId, ClientState>>> =
            Arc::new(RwLock::new(HashMap::new()));
        let next_id = Arc::new(AtomicU64::new(1));

        let server = Self {
            clients: clients.clone(),
            _next_id: next_id.clone(),
        };

        std::thread::spawn(move || {
            Self::accept_loop(port, clients, next_id);
        });

        server
    }

    fn accept_loop(
        port: u16,
        clients: Arc<RwLock<HashMap<ClientId, ClientState>>>,
        next_id: Arc<AtomicU64>,
    ) {
        let addr = format!("127.0.0.1:{port}");
        let listener = match TcpListener::bind(&addr) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("WsServer: failed to bind to {addr}: {e}");
                return;
            }
        };
        eprintln!("WsServer: listening on {addr}");

        for stream in listener.incoming() {
            let stream = match stream {
                Ok(s) => s,
                Err(_) => continue,
            };

            let clients = clients.clone();
            let client_id = next_id.fetch_add(1, Ordering::Relaxed);

            std::thread::spawn(move || {
                Self::handle_client(stream, client_id, clients);
            });
        }
    }

    fn handle_client(
        stream: std::net::TcpStream,
        client_id: ClientId,
        clients: Arc<RwLock<HashMap<ClientId, ClientState>>>,
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
            // Drain outbound messages
            while let Ok(msg) = out_rx.try_recv() {
                if ws.send(Message::Text(msg)).is_err() {
                    Self::remove_client(&clients, client_id);
                    return;
                }
            }

            // Read inbound
            match ws.read() {
                Ok(Message::Text(text)) => {
                    if let Ok(plugin_msg) = serde_json::from_str::<PluginMessage>(&text) {
                        let mut map = clients.write().unwrap();
                        match plugin_msg {
                            PluginMessage::Register { name, version } => {
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
                                }
                            }
                            PluginMessage::Transport { bpm, playing, program } => {
                                if let Some(client) = map.get_mut(&client_id) {
                                    client.info.bpm = bpm;
                                    client.info.playing = playing;
                                    client.info.program = program;
                                }
                            }
                            PluginMessage::ClipAck { clip_id } => {
                                eprintln!("WsServer: client {client_id} acked clip \"{clip_id}\"");
                            }
                        }
                    }
                }
                Ok(Message::Ping(data)) => {
                    let _ = ws.send(Message::Pong(data));
                }
                Ok(Message::Close(_)) => {
                    eprintln!("WsServer: client {client_id} closed connection");
                    Self::remove_client(&clients, client_id);
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
                    Self::remove_client(&clients, client_id);
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
    ) {
        let mut map = clients.write().unwrap();
        map.remove(&client_id);
    }

    // -- Public API for Tauri commands --

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
}
