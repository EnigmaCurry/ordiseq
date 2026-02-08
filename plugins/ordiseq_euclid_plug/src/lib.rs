use nih_plug::prelude::*;
use std::sync::Arc;

/// A Euclidean rhythm generator plugin that creates rhythmic patterns
/// using the Bjorklund algorithm
struct OrdiseqEuclid {
    params: Arc<OrdiseqEuclidParams>,
    /// Current position in the pattern (step index)
    current_step: usize,
    /// Samples elapsed since the current step started
    samples_elapsed: u32,
    /// Currently playing note (to track when to send note off)
    active_note: Option<u8>,
    /// Sample rate for timing calculations
    sample_rate: f32,
    /// Cached euclidean pattern
    pattern: Vec<bool>,
    /// Track parameter changes to regenerate pattern
    last_length: i32,
    last_hits: i32,
    last_rotate: i32,
}

#[derive(Params)]
struct OrdiseqEuclidParams {
    #[id = "length"]
    pub length: IntParam,

    #[id = "hits"]
    pub hits: IntParam,

    #[id = "rotate"]
    pub rotate: IntParam,

    #[id = "transpose"]
    pub transpose: IntParam,

    #[id = "velocity"]
    pub velocity: FloatParam,
}

impl Default for OrdiseqEuclidParams {
    fn default() -> Self {
        Self {
            length: IntParam::new(
                "Length",
                16,
                IntRange::Linear { min: 1, max: 32 },
            ),
            hits: IntParam::new(
                "Hits",
                4,
                IntRange::Linear { min: 0, max: 32 },
            ),
            rotate: IntParam::new(
                "Rotate",
                0,
                IntRange::Linear { min: 0, max: 31 },
            ),
            transpose: IntParam::new(
                "Transpose",
                0,
                IntRange::Linear { min: -24, max: 24 },
            ),
            velocity: FloatParam::new(
                "Velocity",
                0.8,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
        }
    }
}

impl Default for OrdiseqEuclid {
    fn default() -> Self {
        Self {
            params: Arc::new(OrdiseqEuclidParams::default()),
            current_step: 0,
            samples_elapsed: 0,
            active_note: None,
            sample_rate: 44100.0,
            pattern: vec![true, false, false, false], // Default 4/16 pattern
            last_length: 16,
            last_hits: 4,
            last_rotate: 0,
        }
    }
}

impl OrdiseqEuclid {
    /// Default tempo in BPM if DAW doesn't provide one
    const DEFAULT_TEMPO_BPM: f32 = 120.0;
    /// Subdivision: 16th notes (4 per beat)
    const STEPS_PER_BEAT: f32 = 4.0;
    /// The MIDI note to play for hits
    const HIT_NOTE: u8 = 60; // Middle C
    /// Note duration as fraction of step duration (0.5 = 50% gate)
    const NOTE_GATE: f32 = 0.5;

    /// Calculate samples per step based on sample rate and tempo
    fn samples_per_step(&self, tempo_bpm: f32) -> u32 {
        (self.sample_rate * 60.0 / (tempo_bpm * Self::STEPS_PER_BEAT)) as u32
    }

    /// Generate a Euclidean rhythm pattern using the Bresenham line algorithm
    /// This is simpler and more reliable than Bjorklund's algorithm
    fn generate_euclidean_pattern(length: usize, hits: usize) -> Vec<bool> {
        if length == 0 {
            return vec![false];
        }

        if hits == 0 {
            return vec![false; length];
        }

        let hits = hits.min(length);
        let mut pattern = vec![false; length];

        // Use Bresenham's line algorithm to evenly distribute hits
        let mut error = 0i32;
        for i in 0..length {
            error += hits as i32;
            if error >= length as i32 {
                pattern[i] = true;
                error -= length as i32;
            }
        }

        pattern
    }

    /// Apply rotation to the pattern
    fn rotate_pattern(pattern: Vec<bool>, rotate: usize) -> Vec<bool> {
        if pattern.is_empty() || rotate == 0 {
            return pattern;
        }

        let rotate = rotate % pattern.len();
        let mut rotated = Vec::with_capacity(pattern.len());
        rotated.extend_from_slice(&pattern[rotate..]);
        rotated.extend_from_slice(&pattern[..rotate]);
        rotated
    }

    /// Calculate the transposed note, clamped to valid MIDI range (0-127)
    fn get_transposed_note(&self) -> u8 {
        let base_note = Self::HIT_NOTE as i32;
        let transpose = self.params.transpose.value();
        let transposed = (base_note + transpose).clamp(0, 127);
        transposed as u8
    }

    /// Update the pattern if parameters have changed
    fn update_pattern_if_needed(&mut self) {
        let length = self.params.length.value();
        let hits = self.params.hits.value();
        let rotate = self.params.rotate.value();

        if length != self.last_length || hits != self.last_hits || rotate != self.last_rotate {
            // Parameters changed, regenerate pattern
            let mut new_pattern = Self::generate_euclidean_pattern(
                length as usize,
                hits as usize,
            );
            new_pattern = Self::rotate_pattern(new_pattern, rotate as usize);

            // Safety check: ensure pattern is never empty
            if new_pattern.is_empty() {
                new_pattern = vec![false];
            }

            self.pattern = new_pattern;
            self.last_length = length;
            self.last_hits = hits;
            self.last_rotate = rotate;

            // Reset playback position when pattern changes
            self.current_step = 0;
            self.samples_elapsed = 0;
        }
    }
}

impl Plugin for OrdiseqEuclid {
    const NAME: &'static str = "Ordiseq Euclid";
    const VENDOR: &'static str = "EnigmaCurry";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "ryan@enigmacurry.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // This plugin doesn't have any audio IO
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
        let is_playing = transport.playing;

        // If transport is not playing, stop any active notes and reset
        if !is_playing {
            if let Some(note) = self.active_note.take() {
                context.send_event(NoteEvent::NoteOff {
                    timing: 0,
                    voice_id: None,
                    channel: 0,
                    note,
                    velocity: 0.0,
                });
            }
            self.samples_elapsed = 0;
            return ProcessStatus::Normal;
        }

        // Update pattern if parameters changed
        self.update_pattern_if_needed();

        // Get tempo from DAW, or use default if not available
        let tempo_bpm = transport.tempo.unwrap_or(Self::DEFAULT_TEMPO_BPM as f64) as f32;

        // Transport is playing, generate the rhythm
        let buffer_len = buffer.samples() as u32;
        let samples_per_step = self.samples_per_step(tempo_bpm);
        let note_duration_samples = (samples_per_step as f32 * Self::NOTE_GATE) as u32;

        let mut processed_samples = 0u32;

        // Process rhythm events within this buffer
        while processed_samples < buffer_len {
            if self.pattern.is_empty() {
                break;
            }

            // Safety check: ensure current_step is within bounds
            if self.current_step >= self.pattern.len() {
                self.current_step = 0;
                self.samples_elapsed = 0;
            }

            let is_hit = self.pattern[self.current_step];

            // Start of a new step
            if self.samples_elapsed == 0 && is_hit {
                // Send note on at the current sample offset
                let note = self.get_transposed_note();
                let velocity = self.params.velocity.value();
                context.send_event(NoteEvent::NoteOn {
                    timing: processed_samples,
                    voice_id: None,
                    channel: 0,
                    note,
                    velocity,
                });
                self.active_note = Some(note);
            }

            // Calculate how many samples remain in the current step
            let samples_remaining_in_step = samples_per_step - self.samples_elapsed;
            let samples_remaining_in_buffer = buffer_len - processed_samples;
            let samples_to_process = samples_remaining_in_step.min(samples_remaining_in_buffer);

            self.samples_elapsed += samples_to_process;
            processed_samples += samples_to_process;

            // Check if we need to send note off (before step ends)
            if is_hit && self.active_note.is_some() && self.samples_elapsed >= note_duration_samples {
                if let Some(note) = self.active_note.take() {
                    context.send_event(NoteEvent::NoteOff {
                        timing: processed_samples.saturating_sub(1),
                        voice_id: None,
                        channel: 0,
                        note,
                        velocity: 0.0,
                    });
                }
            }

            // Check if we've reached the end of the current step
            if self.samples_elapsed >= samples_per_step {
                // Make sure note is off before moving to next step
                if let Some(note) = self.active_note.take() {
                    context.send_event(NoteEvent::NoteOff {
                        timing: processed_samples.saturating_sub(1),
                        voice_id: None,
                        channel: 0,
                        note,
                        velocity: 0.0,
                    });
                }

                // Move to next step in the pattern (with safety check)
                let pattern_len = self.pattern.len();
                if pattern_len > 0 {
                    self.current_step = (self.current_step + 1) % pattern_len;
                } else {
                    self.current_step = 0;
                }
                self.samples_elapsed = 0;
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for OrdiseqEuclid {
    const CLAP_ID: &'static str = "com.enigmacurry.ordiseq-euclid";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Euclidean rhythm generator using Bjorklund's algorithm");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::Instrument];
}

impl Vst3Plugin for OrdiseqEuclid {
    const VST3_CLASS_ID: [u8; 16] = *b"0rd1s3qEucl1d123";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument];
}

nih_export_clap!(OrdiseqEuclid);
nih_export_vst3!(OrdiseqEuclid);
