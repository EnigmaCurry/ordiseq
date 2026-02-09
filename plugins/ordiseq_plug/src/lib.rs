use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, widgets, EguiState};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, RwLock};

mod names;
mod protocol;
mod ws_client;

use protocol::{AppMessage, MidiClip, PluginMessage};
use ws_client::{STATUS_CONNECTED, STATUS_CONNECTING, STATUS_DISCONNECTED};

const DEFAULT_PORT: u16 = 9850;
const DEFAULT_TEMPO_BPM: f64 = 120.0;

/// Persisted plugin state (survives DAW save/load).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PluginPersistState {
    pub name: String,
    pub port: u16,
    pub clip: Option<MidiClip>,
    pub clip_version: u64,
}

impl Default for PluginPersistState {
    fn default() -> Self {
        Self {
            name: names::generate_name(),
            port: DEFAULT_PORT,
            clip: None,
            clip_version: 0,
        }
    }
}

#[derive(Params)]
struct OrdiseqPlugParams {
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    #[persist = "plugin-state"]
    plugin_state: Arc<RwLock<PluginPersistState>>,

    #[id = "velocity_scale"]
    velocity_scale: FloatParam,

    #[id = "gate_scale"]
    gate_scale: FloatParam,
}

impl Default for OrdiseqPlugParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(400, 280),
            plugin_state: Arc::new(RwLock::new(PluginPersistState::default())),
            velocity_scale: FloatParam::new(
                "Velocity",
                0.8,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),
            gate_scale: FloatParam::new(
                "Gate",
                1.0,
                FloatRange::Linear { min: 0.1, max: 1.0 },
            )
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),
        }
    }
}

struct ActiveNote {
    channel: u8,
    note: u8,
    off_sample: u64,
}

struct OrdiseqPlug {
    params: Arc<OrdiseqPlugParams>,
    sample_rate: f32,
    /// Absolute sample position within the clip (resets on transport start, wraps on loop).
    samples_elapsed: u64,
    was_playing: bool,
    active_notes: Vec<ActiveNote>,
    /// Cached clip copied from persisted state for RT-safe access.
    cached_clip: Option<MidiClip>,
    clip_version: u64,

    // WebSocket
    ws_outbox: Option<crossbeam_channel::Sender<PluginMessage>>,
    ws_inbox: Option<crossbeam_channel::Receiver<AppMessage>>,
    ws_stop_flag: Arc<AtomicBool>,
    ws_thread: Option<std::thread::JoinHandle<()>>,
    connection_status: Arc<AtomicU8>,

    /// Set by audio thread when persist state changes (clip/rename from WS).
    /// GUI checks this and touches a parameter so the host knows to re-save.
    needs_host_notify: Arc<AtomicBool>,

    // Throttling transport reports
    last_reported_bpm: f64,
    last_reported_playing: bool,
}

impl Default for OrdiseqPlug {
    fn default() -> Self {
        Self {
            params: Arc::new(OrdiseqPlugParams::default()),
            sample_rate: 44100.0,
            samples_elapsed: 0,
            was_playing: false,
            active_notes: Vec::new(),
            cached_clip: None,
            clip_version: 0,
            ws_outbox: None,
            ws_inbox: None,
            ws_stop_flag: Arc::new(AtomicBool::new(false)),
            ws_thread: None,
            connection_status: Arc::new(AtomicU8::new(STATUS_DISCONNECTED)),
            needs_host_notify: Arc::new(AtomicBool::new(false)),
            last_reported_bpm: 0.0,
            last_reported_playing: false,
        }
    }
}

impl OrdiseqPlug {
    fn stop_all_notes(&mut self, context: &mut impl ProcessContext<Self>) {
        for note in self.active_notes.drain(..) {
            context.send_event(NoteEvent::NoteOff {
                timing: 0,
                voice_id: None,
                channel: note.channel,
                note: note.note,
                velocity: 0.0,
            });
        }
    }

    fn stop_ws_thread(&mut self) {
        self.ws_stop_flag.store(true, Ordering::Relaxed);
        if let Some(handle) = self.ws_thread.take() {
            let _ = handle.join();
        }
        self.ws_outbox = None;
        self.ws_inbox = None;
        self.connection_status.store(STATUS_DISCONNECTED, Ordering::Relaxed);
    }

    fn start_ws_thread(&mut self) {
        self.ws_stop_flag = Arc::new(AtomicBool::new(false));

        let (outbox_tx, outbox_rx) = crossbeam_channel::bounded::<PluginMessage>(32);
        let (inbox_tx, inbox_rx) = crossbeam_channel::bounded::<AppMessage>(32);

        let handle = ws_client::spawn_ws_thread(
            outbox_rx,
            inbox_tx,
            self.connection_status.clone(),
            self.ws_stop_flag.clone(),
            self.params.plugin_state.clone(),
        );

        self.ws_outbox = Some(outbox_tx);
        self.ws_inbox = Some(inbox_rx);
        self.ws_thread = Some(handle);
    }

    /// Drain incoming WebSocket messages (non-blocking).
    fn poll_ws_inbox(&mut self) {
        let inbox = match &self.ws_inbox {
            Some(rx) => rx,
            None => return,
        };

        while let Ok(msg) = inbox.try_recv() {
            match msg {
                AppMessage::Clip { clip, clip_id } => {
                    if let Ok(mut state) = self.params.plugin_state.write() {
                        state.clip_version += 1;
                        state.clip = Some(clip);
                    }
                    self.needs_host_notify.store(true, Ordering::Relaxed);
                    // Send ack
                    if let Some(tx) = &self.ws_outbox {
                        let _ = tx.try_send(PluginMessage::ClipAck { clip_id });
                    }
                }
                AppMessage::Rename { name } => {
                    if let Ok(mut state) = self.params.plugin_state.write() {
                        state.name = name;
                    }
                    self.needs_host_notify.store(true, Ordering::Relaxed);
                }
                AppMessage::Ping => {
                    // No action needed
                }
            }
        }
    }

    /// Sync cached clip from persisted state if version changed (non-blocking read lock).
    fn sync_cached_clip(&mut self) {
        if let Ok(state) = self.params.plugin_state.try_read() {
            if state.clip_version != self.clip_version {
                self.cached_clip = state.clip.clone();
                self.clip_version = state.clip_version;
            }
        }
    }

    /// Report transport state to app if changed.
    fn report_transport(&mut self, bpm: f64, playing: bool) {
        if (bpm - self.last_reported_bpm).abs() > 0.01 || playing != self.last_reported_playing {
            self.last_reported_bpm = bpm;
            self.last_reported_playing = playing;
            if let Some(tx) = &self.ws_outbox {
                let _ = tx.try_send(PluginMessage::Transport {
                    bpm: bpm as f32,
                    playing,
                });
            }
        }
    }
}

impl Plugin for OrdiseqPlug {
    const NAME: &'static str = "Ordiseq";
    const VENDOR: &'static str = "EnigmaCurry";
    const URL: &'static str = "https://github.com/EnigmaCurry/ordiseq";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[];
    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::MidiCCs;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;
        self.samples_elapsed = 0;
        self.was_playing = false;
        self.active_notes.clear();

        // Sync clip from persisted state
        if let Ok(state) = self.params.plugin_state.read() {
            self.cached_clip = state.clip.clone();
            self.clip_version = state.clip_version;
            nih_log!("Ordiseq: initialize, clip={}, version={}",
                state.clip.as_ref().map_or("None".to_string(), |c| format!("\"{}\" ({} notes)", c.name, c.notes.len())),
                state.clip_version);
        }

        // Start WebSocket thread
        self.start_ws_thread();

        true
    }

    fn deactivate(&mut self) {
        self.stop_ws_thread();
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();
        let connection_status = self.connection_status.clone();
        let needs_host_notify = self.needs_host_notify.clone();

        create_egui_editor(
            self.params.editor_state.clone(),
            (),
            |_, _| {},
            move |egui_ctx, setter, _state| {
                // Notify host that persist state changed so the DAW re-saves
                if needs_host_notify.swap(false, Ordering::Relaxed) {
                    let v = params.velocity_scale.value();
                    setter.begin_set_parameter(&params.velocity_scale);
                    setter.set_parameter(&params.velocity_scale, v);
                    setter.end_set_parameter(&params.velocity_scale);
                }

                egui::CentralPanel::default().show(egui_ctx, |ui| {
                    ui.heading("Ordiseq");
                    ui.add_space(8.0);

                    // Name field
                    {
                        let mut state = params.plugin_state.write().unwrap();
                        ui.horizontal(|ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut state.name);
                        });
                    }

                    // Port field
                    {
                        let mut state = params.plugin_state.write().unwrap();
                        ui.horizontal(|ui| {
                            ui.label("Port:");
                            let mut port_str = state.port.to_string();
                            let response = ui.add(
                                egui::TextEdit::singleline(&mut port_str)
                                    .desired_width(60.0),
                            );
                            if response.changed() {
                                if let Ok(p) = port_str.parse::<u16>() {
                                    if p > 0 {
                                        state.port = p;
                                    }
                                }
                            }
                        });
                    }

                    // Connection status
                    let status = connection_status.load(Ordering::Relaxed);
                    ui.horizontal(|ui| {
                        ui.label("Status:");
                        let (color, text) = match status {
                            STATUS_CONNECTED => (egui::Color32::from_rgb(80, 200, 80), "Connected"),
                            STATUS_CONNECTING => (egui::Color32::from_rgb(220, 180, 50), "Connecting..."),
                            _ => (egui::Color32::from_rgb(200, 80, 80), "Disconnected"),
                        };
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(10.0, 10.0),
                            egui::Sense::hover(),
                        );
                        ui.painter().circle_filled(rect.center(), 5.0, color);
                        ui.label(text);
                    });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(4.0);

                    // Clip info
                    {
                        let state = params.plugin_state.read().unwrap();
                        if let Some(clip) = &state.clip {
                            ui.label(format!(
                                "Clip: \"{}\" ({} beats, {} notes)",
                                clip.name,
                                clip.length_beats,
                                clip.notes.len()
                            ));
                        } else {
                            ui.label("No clip loaded");
                        }
                    }

                    ui.add_space(4.0);

                    // Parameter sliders
                    ui.add(widgets::ParamSlider::for_param(
                        &params.velocity_scale,
                        setter,
                    ));
                    ui.add(widgets::ParamSlider::for_param(
                        &params.gate_scale,
                        setter,
                    ));
                });
            },
        )
    }

    fn process(
        &mut self,
        _buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let transport = context.transport();

        // Poll WebSocket inbox for new clips/renames
        self.poll_ws_inbox();
        self.sync_cached_clip();

        let bpm = transport.tempo.unwrap_or(DEFAULT_TEMPO_BPM);
        let playing = transport.playing;

        // Report transport to app
        self.report_transport(bpm, playing);

        // If not playing, stop notes and reset
        if !playing {
            if self.was_playing {
                self.stop_all_notes(context);
                self.samples_elapsed = 0;
                self.was_playing = false;
            }
            return ProcessStatus::Normal;
        }

        // Transport just started
        if !self.was_playing {
            self.samples_elapsed = 0;
            self.was_playing = true;
        }

        let clip = match &self.cached_clip {
            Some(c) if !c.notes.is_empty() && c.length_beats > 0.0 => c,
            _ => return ProcessStatus::Normal,
        };

        let velocity_scale = self.params.velocity_scale.value();
        let gate_scale = self.params.gate_scale.value();
        let sample_rate = self.sample_rate as f64;
        let beats_per_second = bpm / 60.0;
        let samples_per_beat = sample_rate / beats_per_second;
        let clip_length_samples = (clip.length_beats as f64 * samples_per_beat) as u64;

        if clip_length_samples == 0 {
            return ProcessStatus::Normal;
        }

        let buffer_len = _buffer.samples() as u64;
        let mut processed: u64 = 0;

        while processed < buffer_len {
            let pos_in_clip = self.samples_elapsed % clip_length_samples;
            let pos_beats = pos_in_clip as f64 / samples_per_beat;

            let samples_to_next = buffer_len - processed;
            let samples_to_clip_end = clip_length_samples - pos_in_clip;
            let chunk = samples_to_next.min(samples_to_clip_end);

            let chunk_start_beats = pos_beats;
            let chunk_end_beats = (pos_in_clip + chunk) as f64 / samples_per_beat;

            // Check for note-ons in this chunk
            for clip_note in &clip.notes {
                let note_start = clip_note.start_beats as f64;
                if note_start >= chunk_start_beats && note_start < chunk_end_beats {
                    let sample_offset_in_chunk =
                        ((note_start - chunk_start_beats) * samples_per_beat) as u32;
                    let timing = (processed as u32) + sample_offset_in_chunk;
                    let vel = (clip_note.velocity * velocity_scale).clamp(0.0, 1.0);

                    context.send_event(NoteEvent::NoteOn {
                        timing,
                        voice_id: None,
                        channel: clip_note.channel,
                        note: clip_note.note,
                        velocity: vel,
                    });

                    let duration_samples =
                        (clip_note.duration_beats as f64 * gate_scale as f64 * samples_per_beat) as u64;
                    self.active_notes.push(ActiveNote {
                        channel: clip_note.channel,
                        note: clip_note.note,
                        off_sample: self.samples_elapsed + sample_offset_in_chunk as u64 + duration_samples,
                    });
                }
            }

            // Check for note-offs in this chunk
            let abs_chunk_end = self.samples_elapsed + chunk;
            let mut i = 0;
            while i < self.active_notes.len() {
                if self.active_notes[i].off_sample <= abs_chunk_end {
                    let note = &self.active_notes[i];
                    let off_timing = if note.off_sample > self.samples_elapsed {
                        (note.off_sample - self.samples_elapsed + processed) as u32
                    } else {
                        processed as u32
                    };
                    let off_timing = off_timing.min((buffer_len - 1) as u32);

                    context.send_event(NoteEvent::NoteOff {
                        timing: off_timing,
                        voice_id: None,
                        channel: note.channel,
                        note: note.note,
                        velocity: 0.0,
                    });
                    self.active_notes.swap_remove(i);
                } else {
                    i += 1;
                }
            }

            processed += chunk;
            self.samples_elapsed += chunk;

            // Loop wrap: kill any remaining active notes
            if self.samples_elapsed % clip_length_samples == 0 && !self.active_notes.is_empty() {
                let timing = processed.min(buffer_len - 1) as u32;
                for note in self.active_notes.drain(..) {
                    context.send_event(NoteEvent::NoteOff {
                        timing,
                        voice_id: None,
                        channel: note.channel,
                        note: note.note,
                        velocity: 0.0,
                    });
                }
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for OrdiseqPlug {
    const CLAP_ID: &'static str = "com.enigmacurry.ordiseq";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("WebSocket-connected MIDI clip player");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::Instrument];
}

impl Vst3Plugin for OrdiseqPlug {
    const VST3_CLASS_ID: [u8; 16] = *b"0rd1s3qPlug1234\0";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument];
}

nih_export_clap!(OrdiseqPlug);
nih_export_vst3!(OrdiseqPlug);
