use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender};
use tungstenite::{client::IntoClientRequest, Message, WebSocket};

use crate::protocol::{AppMessage, PluginMessage};
use crate::PluginPersistState;

pub const STATUS_DISCONNECTED: u8 = 0;
pub const STATUS_CONNECTING: u8 = 1;
pub const STATUS_CONNECTED: u8 = 2;

pub fn spawn_ws_thread(
    outbox_rx: Receiver<PluginMessage>,
    inbox_tx: Sender<AppMessage>,
    connection_status: Arc<AtomicU8>,
    stop_flag: Arc<AtomicBool>,
    plugin_state: Arc<RwLock<PluginPersistState>>,
    sync_enabled: Arc<AtomicBool>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        ws_thread_loop(outbox_rx, inbox_tx, connection_status, stop_flag, plugin_state, sync_enabled);
    })
}

fn ws_thread_loop(
    outbox_rx: Receiver<PluginMessage>,
    inbox_tx: Sender<AppMessage>,
    connection_status: Arc<AtomicU8>,
    stop_flag: Arc<AtomicBool>,
    plugin_state: Arc<RwLock<PluginPersistState>>,
    sync_enabled: Arc<AtomicBool>,
) {
    loop {
        if stop_flag.load(Ordering::Relaxed) {
            connection_status.store(STATUS_DISCONNECTED, Ordering::Relaxed);
            return;
        }

        // Reset sync on reconnect — app must re-request
        sync_enabled.store(false, Ordering::Relaxed);

        let (port, name) = {
            let state = plugin_state.read().unwrap();
            (state.port, state.name.clone())
        };
        let mut last_port = port;

        connection_status.store(STATUS_CONNECTING, Ordering::Relaxed);

        let url = format!("ws://127.0.0.1:{port}");
        let stream = match TcpStream::connect_timeout(
            &format!("127.0.0.1:{port}").parse().unwrap(),
            Duration::from_secs(2),
        ) {
            Ok(s) => s,
            Err(_) => {
                connection_status.store(STATUS_DISCONNECTED, Ordering::Relaxed);
                sleep_with_stop_check(&stop_flag, Duration::from_secs(3));
                continue;
            }
        };

        if let Err(_) = stream.set_read_timeout(Some(Duration::from_millis(50))) {
            connection_status.store(STATUS_DISCONNECTED, Ordering::Relaxed);
            sleep_with_stop_check(&stop_flag, Duration::from_secs(3));
            continue;
        }

        let request = match url.as_str().into_client_request() {
            Ok(r) => r,
            Err(_) => {
                connection_status.store(STATUS_DISCONNECTED, Ordering::Relaxed);
                sleep_with_stop_check(&stop_flag, Duration::from_secs(3));
                continue;
            }
        };
        let mut ws = match tungstenite::client(request, stream) {
            Ok((ws, _)) => ws,
            Err(_) => {
                connection_status.store(STATUS_DISCONNECTED, Ordering::Relaxed);
                sleep_with_stop_check(&stop_flag, Duration::from_secs(3));
                continue;
            }
        };

        connection_status.store(STATUS_CONNECTED, Ordering::Relaxed);

        // Send register message
        let register = PluginMessage::Register {
            name,
            version: env!("CARGO_PKG_VERSION").to_string(),
        };
        if send_message(&mut ws, &register).is_err() {
            connection_status.store(STATUS_DISCONNECTED, Ordering::Relaxed);
            sleep_with_stop_check(&stop_flag, Duration::from_secs(3));
            continue;
        }

        // Main read/write loop
        if !client_loop(&mut ws, &outbox_rx, &inbox_tx, &connection_status, &stop_flag, &plugin_state, &sync_enabled, &mut last_port) {
            let _ = ws.close(None);
            connection_status.store(STATUS_DISCONNECTED, Ordering::Relaxed);
            sleep_with_stop_check(&stop_flag, Duration::from_secs(3));
        }
    }
}

/// Returns false if we should reconnect, never returns true (loops until disconnect or stop).
fn client_loop(
    ws: &mut WebSocket<TcpStream>,
    outbox_rx: &Receiver<PluginMessage>,
    inbox_tx: &Sender<AppMessage>,
    connection_status: &Arc<AtomicU8>,
    stop_flag: &Arc<AtomicBool>,
    plugin_state: &Arc<RwLock<PluginPersistState>>,
    sync_enabled: &Arc<AtomicBool>,
    last_port: &mut u16,
) -> bool {
    loop {
        if stop_flag.load(Ordering::Relaxed) {
            connection_status.store(STATUS_DISCONNECTED, Ordering::Relaxed);
            return true;
        }

        // Check if port changed — if so, reconnect
        {
            let state = plugin_state.read().unwrap();
            if state.port != *last_port {
                *last_port = state.port;
                return false;
            }
        }

        // Drain outbox and send to server
        while let Ok(msg) = outbox_rx.try_recv() {
            if send_message(ws, &msg).is_err() {
                return false;
            }
        }

        // Try to read from server (50ms timeout set on socket)
        match ws.read() {
            Ok(Message::Text(text)) => {
                if let Ok(app_msg) = serde_json::from_str::<AppMessage>(&text) {
                    // Handle rename locally on WS thread to update persist state
                    if let AppMessage::Rename { ref name } = app_msg {
                        if let Ok(mut state) = plugin_state.write() {
                            state.name = name.clone();
                        }
                    }
                    // Handle sync commands directly on WS thread (no round-trip to audio)
                    match &app_msg {
                        AppMessage::StartSync => {
                            sync_enabled.store(true, Ordering::Relaxed);
                        }
                        AppMessage::StopSync => {
                            sync_enabled.store(false, Ordering::Relaxed);
                        }
                        _ => {}
                    }
                    let _ = inbox_tx.try_send(app_msg);
                }
            }
            Ok(Message::Ping(data)) => {
                let _ = ws.send(Message::Pong(data));
            }
            Ok(Message::Close(_)) => return false,
            Err(tungstenite::Error::Io(ref e))
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                // Normal timeout, continue loop
            }
            Err(_) => return false,
            _ => {}
        }
    }
}

fn send_message(
    ws: &mut WebSocket<TcpStream>,
    msg: &PluginMessage,
) -> Result<(), ()> {
    let json = serde_json::to_string(msg).map_err(|_| ())?;
    ws.send(Message::Text(json)).map_err(|_| ())
}

fn sleep_with_stop_check(stop_flag: &Arc<AtomicBool>, duration: Duration) {
    let steps = 10;
    let step_dur = duration / steps;
    for _ in 0..steps {
        if stop_flag.load(Ordering::Relaxed) {
            return;
        }
        std::thread::sleep(step_dur);
    }
}
