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
    /// Tempo in BPM (16th notes)
    const TEMPO_BPM: f32 = 120.0;
    /// Subdivision: 16th notes (4 per beat)
    const STEPS_PER_BEAT: f32 = 4.0;
    /// The MIDI note to play for hits
    const HIT_NOTE: u8 = 60; // Middle C
    /// Note duration as fraction of step duration (0.5 = 50% gate)
    const NOTE_GATE: f32 = 0.5;

    /// Calculate samples per step based on sample rate and tempo
    fn samples_per_step(&self) -> u32 {
        (self.sample_rate * 60.0 / (Self::TEMPO_BPM * Self::STEPS_PER_BEAT)) as u32
    }

    /// Generate a Euclidean rhythm pattern using Bjorklund's algorithm
    fn generate_euclidean_pattern(length: usize, hits: usize) -> Vec<bool> {
        if length == 0 || hits == 0 {
            return vec![false; length.max(1)];
        }

        let hits = hits.min(length);
        let mut pattern = vec![false; length];

        if hits == 0 {
            return pattern;
        }

        // Bjorklund's algorithm
        let mut counts = vec![0; length];
        let mut remainders = vec![0; length];

        let mut divisor = length - hits;
        remainders[0] = hits;

        let mut level = 0;
        loop {
            counts[level] = divisor / remainders[level];
            remainders[level + 1] = divisor % remainders[level];
            divisor = remainders[level];
            level += 1;

            if remainders[level] <= 1 {
                break;
            }
        }

        counts[level] = divisor;

        // Build the pattern
        let mut index = 0;
        fn build(
            pattern: &mut Vec<bool>,
            index: &mut usize,
            level: usize,
            counts: &[usize],
            remainders: &[usize],
        ) {
            if level == 0 {
                pattern[*index] = true;
                *index += 1;
            } else if level == 1 {
                for _ in 0..counts[0] {
                    pattern[*index] = true;
                    *index += 1;
                    pattern[*index] = false;
                    *index += 1;
                }
                if remainders[0] > 0 {
                    pattern[*index] = true;
                    *index += 1;
                }
            } else {
                for _ in 0..counts[level - 1] {
                    build(pattern, index, level - 1, counts, remainders);
                    build(pattern, index, level - 2, counts, remainders);
                }
                if remainders[level - 1] > 0 {
                    build(pattern, index, level - 1, counts, remainders);
                }
            }
        }

        build(&mut pattern, &mut index, level, &counts, &remainders);
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

        // Transport is playing, generate the rhythm
        let buffer_len = buffer.samples() as u32;
        let samples_per_step = self.samples_per_step();
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
                context.send_event(NoteEvent::NoteOn {
                    timing: processed_samples,
                    voice_id: None,
                    channel: 0,
                    note: Self::HIT_NOTE,
                    velocity: 0.8,
                });
                self.active_note = Some(Self::HIT_NOTE);
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

                // Move to next step in the pattern
                self.current_step = (self.current_step + 1) % self.pattern.len();
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
