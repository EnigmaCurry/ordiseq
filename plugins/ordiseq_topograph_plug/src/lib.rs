// Ordiseq Topograph - Topographic drum sequencer based on Mutable Instruments Grids
//
// Copyright 2011 Emilie Gillet (Mutable Instruments)
// Copyright 2017 Dale Johnson (Valley Audio - VCV Rack port)
// Copyright 2024 EnigmaCurry (Rust/NIH-plug port)
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.
//
// Pattern data and core algorithm derived from:
// - Mutable Instruments Grids: https://github.com/pichenettes/eurorack/tree/master/grids
// - Valley Audio Topograph: https://github.com/ValleyAudio/ValleyRackFree/tree/main/src/Topograph

use nih_plug::prelude::*;
use std::sync::Arc;

mod patterns;
use patterns::{DRUM_MAP, PATTERN_NODES};

/// Ordiseq Topograph - A topographic drum sequencer based on Mutable Instruments Grids
/// Port of the Grids/Topograph algorithm for generating 3-channel drum patterns
struct OrdiseqTopograph {
    params: Arc<OrdiseqTopographParams>,

    // Timing state
    current_step: usize,
    samples_elapsed: u32,
    sample_rate: f32,

    // Active note tracking for 5 channels
    active_bd_note: Option<u8>,
    active_sd_note: Option<u8>,
    active_hh_note: Option<u8>,
    active_shaker_note: Option<u8>,
    active_perc_note: Option<u8>,

    // Pattern cache (32 steps × 5 channels)
    cached_pattern: [DrumPattern; 32],

    // Parameter change detection
    last_map_x: i32,
    last_map_y: i32,
    last_bd_density: i32,
    last_sd_density: i32,
    last_hh_density: i32,
    last_randomness: i32,
    last_shaker_density: i32,
    last_perc_density: i32,

    // RNG state for deterministic randomness
    rng_state: u32,
}

#[derive(Clone, Copy)]
struct DrumPattern {
    bd: bool,      // bass drum hit
    sd: bool,      // snare drum hit
    hh: bool,      // hi-hat hit
    shaker: bool,  // shaker/tambourine (derived from HH probability)
    perc: bool,    // syncopated percussion (inverted BD pattern)
}

impl Default for DrumPattern {
    fn default() -> Self {
        Self {
            bd: false,
            sd: false,
            hh: false,
            shaker: false,
            perc: false,
        }
    }
}

#[derive(Params)]
struct OrdiseqTopographParams {
    #[id = "map_x"]
    pub map_x: IntParam,

    #[id = "map_y"]
    pub map_y: IntParam,

    #[id = "bd_density"]
    pub bd_density: IntParam,

    #[id = "sd_density"]
    pub sd_density: IntParam,

    #[id = "hh_density"]
    pub hh_density: IntParam,

    #[id = "randomness"]
    pub randomness: IntParam,

    #[id = "shaker_density"]
    pub shaker_density: IntParam,

    #[id = "perc_density"]
    pub perc_density: IntParam,

    #[id = "speed"]
    pub speed: IntParam,
}

impl Default for OrdiseqTopographParams {
    fn default() -> Self {
        Self {
            map_x: IntParam::new(
                "Map X",
                128,
                IntRange::Linear { min: 0, max: 255 },
            ),
            map_y: IntParam::new(
                "Map Y",
                128,
                IntRange::Linear { min: 0, max: 255 },
            ),
            bd_density: IntParam::new(
                "BD Density",
                192,
                IntRange::Linear { min: 0, max: 255 },
            ),
            sd_density: IntParam::new(
                "SD Density",
                128,
                IntRange::Linear { min: 0, max: 255 },
            ),
            hh_density: IntParam::new(
                "HH Density",
                160,
                IntRange::Linear { min: 0, max: 255 },
            ),
            randomness: IntParam::new(
                "Randomness",
                0,
                IntRange::Linear { min: 0, max: 255 },
            ),
            shaker_density: IntParam::new(
                "Shaker Density",
                0,
                IntRange::Linear { min: 0, max: 255 },
            ),
            perc_density: IntParam::new(
                "Perc Density",
                0,
                IntRange::Linear { min: 0, max: 255 },
            ),
            speed: IntParam::new(
                "Speed",
                1,
                IntRange::Linear { min: 0, max: 2 },
            )
            .with_value_to_string(Arc::new(|value| {
                match value {
                    0 => "Slow".to_string(),
                    1 => "Medium".to_string(),
                    2 => "Fast".to_string(),
                    _ => "Medium".to_string(),
                }
            }))
            .with_string_to_value(Arc::new(|string| {
                match string {
                    "Slow" => Some(0),
                    "Medium" => Some(1),
                    "Fast" => Some(2),
                    _ => None,
                }
            })),
        }
    }
}

impl Default for OrdiseqTopograph {
    fn default() -> Self {
        Self {
            params: Arc::new(OrdiseqTopographParams::default()),
            current_step: 0,
            samples_elapsed: 0,
            sample_rate: 44100.0,
            active_bd_note: None,
            active_sd_note: None,
            active_hh_note: None,
            active_shaker_note: None,
            active_perc_note: None,
            cached_pattern: [DrumPattern::default(); 32],
            last_map_x: -1,
            last_map_y: -1,
            last_bd_density: -1,
            last_sd_density: -1,
            last_hh_density: -1,
            last_randomness: -1,
            last_shaker_density: -1,
            last_perc_density: -1,
            rng_state: 12345,
        }
    }
}

impl OrdiseqTopograph {
    const DEFAULT_TEMPO_BPM: f32 = 120.0;
    const PATTERN_LENGTH: usize = 32;
    const NOTE_GATE: f32 = 0.5;  // 50% gate length

    // GM Drum mapping (standard MIDI drums)
    const BD_NOTE: u8 = 36;  // C1 - Bass Drum
    const SD_NOTE: u8 = 38;  // D1 - Snare Drum
    const HH_NOTE: u8 = 42;  // F#1 - Closed Hi-Hat
    const SHAKER_NOTE: u8 = 51;  // Eb2 - Ride Cymbal 1 (algorithmically derived from HH)
    const PERC_NOTE: u8 = 50;    // D2 - High Tom (inverted BD pattern)

    /// Get steps per beat based on speed parameter
    fn steps_per_beat(&self) -> f32 {
        match self.params.speed.value() {
            0 => 2.0,  // Slow: 8th notes
            1 => 4.0,  // Medium: 16th notes (default)
            2 => 8.0,  // Fast: 32nd notes
            _ => 4.0,  // Default to medium
        }
    }

    /// Calculate samples per step based on tempo and speed
    fn samples_per_step(&self, tempo_bpm: f32) -> u32 {
        let steps_per_beat = self.steps_per_beat();
        (self.sample_rate * 60.0 / (tempo_bpm * steps_per_beat)) as u32
    }

    /// Check if parameters changed and regenerate pattern if needed
    fn update_pattern_if_needed(&mut self) {
        let map_x = self.params.map_x.value();
        let map_y = self.params.map_y.value();
        let bd_density = self.params.bd_density.value();
        let sd_density = self.params.sd_density.value();
        let hh_density = self.params.hh_density.value();
        let randomness = self.params.randomness.value();
        let shaker_density = self.params.shaker_density.value();
        let perc_density = self.params.perc_density.value();

        // Check if any parameter changed
        if map_x == self.last_map_x
            && map_y == self.last_map_y
            && bd_density == self.last_bd_density
            && sd_density == self.last_sd_density
            && hh_density == self.last_hh_density
            && randomness == self.last_randomness
            && shaker_density == self.last_shaker_density
            && perc_density == self.last_perc_density
        {
            return; // No change needed
        }

        // Regenerate pattern
        self.generate_pattern(map_x, map_y, bd_density, sd_density, hh_density, randomness, shaker_density, perc_density);

        // Update cache tracking
        self.last_map_x = map_x;
        self.last_map_y = map_y;
        self.last_bd_density = bd_density;
        self.last_sd_density = sd_density;
        self.last_hh_density = hh_density;
        self.last_randomness = randomness;
        self.last_shaker_density = shaker_density;
        self.last_perc_density = perc_density;

        // Don't reset playback position - let pattern continue smoothly
        // This prevents rapid retriggering when adjusting parameters

        // Reset RNG for deterministic randomness per pattern
        self.rng_state = 12345;
    }

    /// Generate pattern using bilinear interpolation
    fn generate_pattern(
        &mut self,
        map_x: i32,
        map_y: i32,
        bd_density: i32,
        sd_density: i32,
        hh_density: i32,
        randomness: i32,
        shaker_density: i32,
        perc_density: i32,
    ) {
        // Convert 0-255 map coordinates to floating-point grid position (0.0-4.0)
        let x_float = (map_x as f32 / 255.0) * 4.0;
        let y_float = (map_y as f32 / 255.0) * 4.0;

        // Find surrounding grid nodes
        let x0 = x_float.floor() as usize;
        let y0 = y_float.floor() as usize;
        let x1 = (x0 + 1).min(4);
        let y1 = (y0 + 1).min(4);

        // Calculate interpolation weights (0.0-1.0)
        let wx = x_float - x0 as f32;
        let wy = y_float - y0 as f32;

        // Get the 4 corner node indices from the drum map
        let node_00_idx = DRUM_MAP[y0][x0];
        let node_10_idx = DRUM_MAP[y0][x1];
        let node_01_idx = DRUM_MAP[y1][x0];
        let node_11_idx = DRUM_MAP[y1][x1];

        // Get the actual pattern nodes
        let node_00 = &PATTERN_NODES[node_00_idx];
        let node_10 = &PATTERN_NODES[node_10_idx];
        let node_01 = &PATTERN_NODES[node_01_idx];
        let node_11 = &PATTERN_NODES[node_11_idx];

        // Generate pattern for each of the 32 steps
        for step in 0..Self::PATTERN_LENGTH {
            // Get random offset for this step
            let rand_offset = self.get_random_offset(randomness);

            // Interpolate BD channel
            let bd_val = self.interpolate_channel(
                node_00.get_bd(step),
                node_10.get_bd(step),
                node_01.get_bd(step),
                node_11.get_bd(step),
                wx,
                wy,
            );
            let bd_hit = (bd_val + rand_offset) > (255 - bd_density);

            // Interpolate SD channel
            let sd_val = self.interpolate_channel(
                node_00.get_sd(step),
                node_10.get_sd(step),
                node_01.get_sd(step),
                node_11.get_sd(step),
                wx,
                wy,
            );
            let sd_hit = (sd_val + rand_offset) > (255 - sd_density);

            // Interpolate HH channel
            let hh_val = self.interpolate_channel(
                node_00.get_hh(step),
                node_10.get_hh(step),
                node_01.get_hh(step),
                node_11.get_hh(step),
                wx,
                wy,
            );
            let hh_hit = (hh_val + rand_offset) > (255 - hh_density);

            // Algorithmically-derived channels:

            // Shaker: Use HH interpolated value as probability generator
            // When HH pattern has high values, shaker is more likely to trigger
            let shaker_hit = if shaker_density > 0 {
                // Use HH pattern value as probability threshold
                // Higher HH values = more likely to trigger shaker
                let shaker_threshold = 255 - ((hh_val as f32 * shaker_density as f32) / 255.0) as i32;
                (rand_offset + 128) > shaker_threshold
            } else {
                false
            };

            // Perc: Inverted BD pattern for syncopation
            // Triggers where BD doesn't, creating counterpoint rhythm
            let perc_hit = if perc_density > 0 {
                let inverted_bd_val = 255 - bd_val;
                (inverted_bd_val + rand_offset) > (255 - perc_density)
            } else {
                false
            };

            // Store in cached pattern
            self.cached_pattern[step] = DrumPattern {
                bd: bd_hit,
                sd: sd_hit,
                hh: hh_hit,
                shaker: shaker_hit,
                perc: perc_hit,
            };
        }
    }

    /// Bilinear interpolation for a single channel value
    /// v00, v10, v01, v11 are the four corner values
    /// wx, wy are the interpolation weights (0.0-1.0)
    fn interpolate_channel(&self, v00: u8, v10: u8, v01: u8, v11: u8, wx: f32, wy: f32) -> i32 {
        // Convert u8 values to f32 for interpolation
        let v00_f = v00 as f32;
        let v10_f = v10 as f32;
        let v01_f = v01 as f32;
        let v11_f = v11 as f32;

        // Bilinear interpolation formula:
        // result = (1-wx)(1-wy)v00 + wx(1-wy)v10 + (1-wx)wy*v01 + wx*wy*v11
        let result = (1.0 - wx) * (1.0 - wy) * v00_f
            + wx * (1.0 - wy) * v10_f
            + (1.0 - wx) * wy * v01_f
            + wx * wy * v11_f;

        result.round() as i32
    }

    /// Generate random offset using simple LCG (Linear Congruential Generator)
    fn get_random_offset(&mut self, randomness: i32) -> i32 {
        if randomness == 0 {
            return 0;
        }

        // Simple LCG: X(n+1) = (a * X(n) + c) mod m
        self.rng_state = self.rng_state.wrapping_mul(1103515245).wrapping_add(12345);

        // Extract random byte from state
        let rand_value = ((self.rng_state >> 16) & 0xFF) as i32;

        // Scale by randomness parameter and center around 0
        ((rand_value * randomness) / 255) - (randomness / 2)
    }

    /// Stop all currently active notes for the current step
    fn stop_step_notes(&mut self, context: &mut impl ProcessContext<Self>, timing: u32) {
        if let Some(note) = self.active_bd_note.take() {
            context.send_event(NoteEvent::NoteOff {
                timing,
                voice_id: None,
                channel: 0,
                note,
                velocity: 0.0,
            });
        }
        if let Some(note) = self.active_sd_note.take() {
            context.send_event(NoteEvent::NoteOff {
                timing,
                voice_id: None,
                channel: 0,
                note,
                velocity: 0.0,
            });
        }
        if let Some(note) = self.active_hh_note.take() {
            context.send_event(NoteEvent::NoteOff {
                timing,
                voice_id: None,
                channel: 0,
                note,
                velocity: 0.0,
            });
        }
        if let Some(note) = self.active_shaker_note.take() {
            context.send_event(NoteEvent::NoteOff {
                timing,
                voice_id: None,
                channel: 0,
                note,
                velocity: 0.0,
            });
        }
        if let Some(note) = self.active_perc_note.take() {
            context.send_event(NoteEvent::NoteOff {
                timing,
                voice_id: None,
                channel: 0,
                note,
                velocity: 0.0,
            });
        }
    }

    /// Stop all notes (called when transport stops)
    fn stop_all_notes(&mut self, context: &mut impl ProcessContext<Self>) {
        self.stop_step_notes(context, 0);
    }
}

impl Plugin for OrdiseqTopograph {
    const NAME: &'static str = "Ordiseq Topograph";
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

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;
        // Generate initial pattern
        self.update_pattern_if_needed();
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let transport = context.transport();

        // Stop and reset if transport is not playing
        if !transport.playing {
            self.stop_all_notes(context);
            self.samples_elapsed = 0;
            return ProcessStatus::Normal;
        }

        // Update pattern if parameters changed
        self.update_pattern_if_needed();

        // Get tempo from DAW or use default
        let tempo_bpm = transport.tempo.unwrap_or(Self::DEFAULT_TEMPO_BPM as f64) as f32;
        let buffer_len = buffer.samples() as u32;
        let samples_per_step = self.samples_per_step(tempo_bpm);
        let note_duration = (samples_per_step as f32 * Self::NOTE_GATE) as u32;

        let mut processed_samples = 0u32;

        while processed_samples < buffer_len {
            // Safety check: wrap step if needed
            if self.current_step >= Self::PATTERN_LENGTH {
                self.current_step = 0;
                self.samples_elapsed = 0;
            }

            let pattern = self.cached_pattern[self.current_step];

            // Start of new step - send note-on events
            if self.samples_elapsed == 0 {
                if pattern.bd {
                    context.send_event(NoteEvent::NoteOn {
                        timing: processed_samples,
                        voice_id: None,
                        channel: 0,
                        note: Self::BD_NOTE,
                        velocity: 0.8,
                    });
                    self.active_bd_note = Some(Self::BD_NOTE);
                }
                if pattern.sd {
                    context.send_event(NoteEvent::NoteOn {
                        timing: processed_samples,
                        voice_id: None,
                        channel: 0,
                        note: Self::SD_NOTE,
                        velocity: 0.8,
                    });
                    self.active_sd_note = Some(Self::SD_NOTE);
                }
                if pattern.hh {
                    context.send_event(NoteEvent::NoteOn {
                        timing: processed_samples,
                        voice_id: None,
                        channel: 0,
                        note: Self::HH_NOTE,
                        velocity: 0.8,
                    });
                    self.active_hh_note = Some(Self::HH_NOTE);
                }
                if pattern.shaker {
                    context.send_event(NoteEvent::NoteOn {
                        timing: processed_samples,
                        voice_id: None,
                        channel: 0,
                        note: Self::SHAKER_NOTE,
                        velocity: 0.6,
                    });
                    self.active_shaker_note = Some(Self::SHAKER_NOTE);
                }
                if pattern.perc {
                    context.send_event(NoteEvent::NoteOn {
                        timing: processed_samples,
                        voice_id: None,
                        channel: 0,
                        note: Self::PERC_NOTE,
                        velocity: 0.7,
                    });
                    self.active_perc_note = Some(Self::PERC_NOTE);
                }
            }

            // Calculate how many samples to process
            let samples_remaining_in_step = samples_per_step - self.samples_elapsed;
            let samples_remaining_in_buffer = buffer_len - processed_samples;
            let samples_to_process = samples_remaining_in_step.min(samples_remaining_in_buffer);

            self.samples_elapsed += samples_to_process;
            processed_samples += samples_to_process;

            // Send note-off events at gate duration
            if self.samples_elapsed >= note_duration {
                self.stop_step_notes(context, processed_samples.saturating_sub(1));
            }

            // Advance to next step when step completes
            if self.samples_elapsed >= samples_per_step {
                // Safety: stop any lingering notes
                self.stop_step_notes(context, processed_samples.saturating_sub(1));

                self.current_step = (self.current_step + 1) % Self::PATTERN_LENGTH;
                self.samples_elapsed = 0;
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for OrdiseqTopograph {
    const CLAP_ID: &'static str = "com.enigmacurry.ordiseq-topograph";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Topographic drum sequencer - port of Mutable Instruments Grids");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::Instrument];
}

impl Vst3Plugin for OrdiseqTopograph {
    const VST3_CLASS_ID: [u8; 16] = *b"0rd1s3qT0p0gr4ph";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Drum];
}

nih_export_clap!(OrdiseqTopograph);
nih_export_vst3!(OrdiseqTopograph);
