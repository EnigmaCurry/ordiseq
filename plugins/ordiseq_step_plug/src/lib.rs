use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, widgets, EguiState};
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, RwLock,
};

/// A 4-track, 16-step sequencer with interactive GUI
struct OrdiseqStep {
    params: Arc<OrdiseqStepParams>,

    // Timing (same as Euclid/Topograph)
    current_step: usize,
    samples_elapsed: u32,
    sample_rate: f32,

    // Note tracking (4 independent tracks)
    active_notes: [Option<u8>; 4],

    // Step pattern cache (synced from params)
    pattern: [[bool; 16]; 4],

    // GUI communication
    current_step_display: Arc<AtomicUsize>,
}

#[derive(Params)]
struct OrdiseqStepParams {
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    #[persist = "pattern-state"]
    pattern_state: Arc<RwLock<PatternState>>,

    #[id = "current_pattern"]
    current_pattern: IntParam,

    #[id = "velocity"]
    velocity: FloatParam,

    #[id = "gate_length"]
    gate_length: FloatParam,
}

#[derive(Serialize, Deserialize)]
struct PatternState {
    patterns: [[[bool; 16]; 4]; 16],  // 16 patterns, each with 4 tracks × 16 steps
}

impl Default for PatternState {
    fn default() -> Self {
        Self {
            patterns: [[[false; 16]; 4]; 16],  // All 16 patterns start empty
        }
    }
}

impl Default for OrdiseqStepParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(700, 350),
            pattern_state: Arc::new(RwLock::new(PatternState::default())),
            current_pattern: IntParam::new(
                "Pattern",
                0,
                IntRange::Linear { min: 0, max: 15 },
            )
            .with_value_to_string(Arc::new(|value| format!("{}", value + 1)))
            .with_string_to_value(Arc::new(|string| {
                string.parse::<i32>().ok().map(|v| v - 1)
            })),
            velocity: FloatParam::new(
                "Velocity",
                0.8,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),
            gate_length: FloatParam::new(
                "Gate Length",
                0.5,
                FloatRange::Linear { min: 0.1, max: 1.0 },
            )
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),
        }
    }
}

impl Default for OrdiseqStep {
    fn default() -> Self {
        Self {
            params: Arc::new(OrdiseqStepParams::default()),
            current_step: 0,
            samples_elapsed: 0,
            active_notes: [None; 4],
            sample_rate: 44100.0,
            pattern: [[false; 16]; 4],
            current_step_display: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl OrdiseqStep {
    /// Default tempo in BPM if DAW doesn't provide one
    const DEFAULT_TEMPO_BPM: f32 = 120.0;
    /// Subdivision: 16th notes (4 per beat)
    const STEPS_PER_BEAT: f32 = 4.0;
    /// Pattern length (16 steps)
    const PATTERN_LENGTH: usize = 16;
    /// MIDI note assignments (chromatic half-steps starting at C1)
    const TRACK_NOTES: [u8; 4] = [
        36, // C1  - Track 1
        37, // C#1 - Track 2
        38, // D1  - Track 3
        39, // D#1 - Track 4
    ];

    /// Calculate samples per step based on sample rate and tempo
    fn samples_per_step(&self, tempo_bpm: f32) -> u32 {
        (self.sample_rate * 60.0 / (tempo_bpm * Self::STEPS_PER_BEAT)) as u32
    }

    /// Stop notes for the current step
    fn stop_step_notes(&mut self, context: &mut impl ProcessContext<Self>, timing: u32) {
        for track_idx in 0..4 {
            if let Some(note) = self.active_notes[track_idx].take() {
                context.send_event(NoteEvent::NoteOff {
                    timing,
                    voice_id: None,
                    channel: 0,
                    note,
                    velocity: 0.0,
                });
            }
        }
    }

    /// Stop all active notes
    fn stop_all_notes(&mut self, context: &mut impl ProcessContext<Self>) {
        self.stop_step_notes(context, 0);
    }
}

impl Plugin for OrdiseqStep {
    const NAME: &'static str = "Ordiseq Step";
    const VENDOR: &'static str = "EnigmaCurry";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "ryan@enigmacurry.com";
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

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();
        let current_step_display = self.current_step_display.clone();

        create_egui_editor(
            self.params.editor_state.clone(),
            (),
            |_, _| {},
            move |egui_ctx, setter, _state| {
                egui::CentralPanel::default().show(egui_ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.heading("Ordiseq Step Sequencer");
                        ui.add_space(5.0);

                        // Pattern selector dropdown
                        ui.horizontal(|ui| {
                            ui.label("Pattern:");
                            let pattern_idx = params.current_pattern.value() as usize;
                            egui::ComboBox::from_id_salt("pattern_selector")
                                .selected_text(format!("Pattern {}", pattern_idx + 1))
                                .show_ui(ui, |ui| {
                                    for i in 0..16 {
                                        if ui.selectable_value(
                                            &mut (pattern_idx as i32),
                                            i,
                                            format!("Pattern {}", i + 1),
                                        ).clicked() {
                                            setter.set_parameter(&params.current_pattern, i);
                                        }
                                    }
                                });
                        });
                        ui.add_space(10.0);

                        let current_step = current_step_display.load(Ordering::Relaxed);
                        let mut pattern_state = params.pattern_state.write().unwrap();
                        let pattern_idx = params.current_pattern.value() as usize;

                        // Render 4 track rows (Track 4 at top, Track 1 at bottom)
                        for track_idx in (0..4).rev() {
                            ui.horizontal(|ui| {
                                ui.label(format!("Track {}", track_idx + 1));
                                ui.add_space(5.0);

                                // 16 checkboxes per track (grouped in sets of 4)
                                for step_idx in 0..16 {
                                    let is_current = step_idx == current_step;
                                    let is_active = &mut pattern_state.patterns[pattern_idx][track_idx][step_idx];

                                    let checkbox = egui::Checkbox::new(is_active, "");
                                    let response = ui.add_sized([20.0, 20.0], checkbox);

                                    // Highlight current step
                                    if is_current {
                                        ui.painter().rect_stroke(
                                            response.rect,
                                            0.0,
                                            egui::Stroke::new(2.0, egui::Color32::YELLOW),
                                            egui::epaint::StrokeKind::Outside,
                                        );
                                    }

                                    // Add extra spacing after every 4th step
                                    if step_idx % 4 == 3 && step_idx < 15 {
                                        ui.add_space(10.0);
                                    }
                                }

                                ui.add_space(10.0);

                                // Per-track clear button
                                if ui.button(format!("Clear Track {}", track_idx + 1)).clicked() {
                                    pattern_state.patterns[pattern_idx][track_idx] = [false; 16];
                                }
                            });
                            ui.add_space(5.0);
                        }

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);

                        // Parameters
                        ui.horizontal(|ui| {
                            ui.label("Velocity:");
                            ui.add(widgets::ParamSlider::for_param(&params.velocity, setter));

                            ui.add_space(20.0);

                            ui.label("Gate Length:");
                            ui.add(widgets::ParamSlider::for_param(&params.gate_length, setter));
                        });
                    });
                });
            },
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let transport = context.transport();

        // If transport is not playing, stop all notes and reset
        if !transport.playing {
            self.stop_all_notes(context);
            self.samples_elapsed = 0;
            return ProcessStatus::Normal;
        }

        // Read pattern from params (thread-safe)
        let pattern_idx = self.params.current_pattern.value() as usize;
        {
            let pattern_state = self.params.pattern_state.read().unwrap();
            self.pattern = pattern_state.patterns[pattern_idx];
        }

        let tempo_bpm = transport.tempo.unwrap_or(Self::DEFAULT_TEMPO_BPM as f64) as f32;
        let buffer_len = buffer.samples() as u32;
        let samples_per_step = self.samples_per_step(tempo_bpm);
        let gate_length = self.params.gate_length.value();
        let note_duration = (samples_per_step as f32 * gate_length) as u32;
        let velocity = self.params.velocity.value();

        let mut processed_samples = 0u32;

        // Process events across buffer boundaries
        while processed_samples < buffer_len {
            // Safety: wrap step index
            if self.current_step >= Self::PATTERN_LENGTH {
                self.current_step = 0;
                self.samples_elapsed = 0;
            }

            // Update GUI display
            self.current_step_display
                .store(self.current_step, Ordering::Relaxed);

            // Start of new step - send note-on events
            if self.samples_elapsed == 0 {
                for track_idx in 0..4 {
                    if self.pattern[track_idx][self.current_step] {
                        let note = Self::TRACK_NOTES[track_idx];

                        context.send_event(NoteEvent::NoteOn {
                            timing: processed_samples,
                            voice_id: None,
                            channel: 0,
                            note,
                            velocity,
                        });
                        self.active_notes[track_idx] = Some(note);
                    }
                }
            }

            // Calculate samples to process
            let samples_remaining_in_step = samples_per_step - self.samples_elapsed;
            let samples_remaining_in_buffer = buffer_len - processed_samples;
            let samples_to_process = samples_remaining_in_step.min(samples_remaining_in_buffer);

            self.samples_elapsed += samples_to_process;
            processed_samples += samples_to_process;

            // Send note-off at gate duration
            if self.samples_elapsed >= note_duration {
                self.stop_step_notes(context, processed_samples.saturating_sub(1));
            }

            // Advance to next step
            if self.samples_elapsed >= samples_per_step {
                self.stop_step_notes(context, processed_samples.saturating_sub(1));
                self.current_step = (self.current_step + 1) % Self::PATTERN_LENGTH;
                self.samples_elapsed = 0;
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for OrdiseqStep {
    const CLAP_ID: &'static str = "com.enigmacurry.ordiseq-step";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("4-track 16-step sequencer with visual step editor");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::Instrument];
}

impl Vst3Plugin for OrdiseqStep {
    const VST3_CLASS_ID: [u8; 16] = *b"0rd1s3qSt3p12345";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Instrument];
}

nih_export_clap!(OrdiseqStep);
nih_export_vst3!(OrdiseqStep);
