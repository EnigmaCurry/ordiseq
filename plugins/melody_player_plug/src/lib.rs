use nih_plug::prelude::*;
use std::sync::Arc;

/// A plugin that plays melodies continuously when the transport is playing
struct MelodyPlayer {
    params: Arc<MelodyPlayerParams>,
    /// Current position in the melody (note index)
    current_note: usize,
    /// Samples elapsed since the current note started
    samples_elapsed: u32,
    /// Currently playing note (to track when to send note off)
    active_note: Option<u8>,
    /// Sample rate for timing calculations
    sample_rate: f32,
    /// Track the last selected melody to detect changes
    last_melody: MelodyType,
}

#[derive(Debug, Clone, Copy, PartialEq, Enum)]
pub enum MelodyType {
    #[name = "Jingle Bells"]
    JingleBells,
    #[name = "Row Row Your Boat"]
    RowRowYourBoat,
}

#[derive(Params)]
struct MelodyPlayerParams {
    #[id = "melody"]
    pub melody: EnumParam<MelodyType>,
}

/// A note in the melody with its MIDI note number and duration in beats
#[derive(Clone, Copy)]
struct MelodyNote {
    note: u8,
    duration_beats: f32,
}

impl MelodyPlayer {
    /// The Jingle Bells melody (first line)
    /// E E E - E E E - E G C D E
    const JINGLE_BELLS: &'static [MelodyNote] = &[
        MelodyNote { note: 64, duration_beats: 1.0 },  // E4
        MelodyNote { note: 64, duration_beats: 1.0 },  // E4
        MelodyNote { note: 64, duration_beats: 2.0 },  // E4 (half note)
        MelodyNote { note: 64, duration_beats: 1.0 },  // E4
        MelodyNote { note: 64, duration_beats: 1.0 },  // E4
        MelodyNote { note: 64, duration_beats: 2.0 },  // E4 (half note)
        MelodyNote { note: 64, duration_beats: 1.0 },  // E4
        MelodyNote { note: 67, duration_beats: 1.0 },  // G4
        MelodyNote { note: 60, duration_beats: 1.5 },  // C4 (dotted quarter)
        MelodyNote { note: 62, duration_beats: 0.5 },  // D4 (eighth note)
        MelodyNote { note: 64, duration_beats: 4.0 },  // E4 (whole note)
    ];

    /// Row Row Row Your Boat melody
    /// C C C D E - E D E F G - (high) C C C G G G E E E C C C - G F E D C
    const ROW_ROW_YOUR_BOAT: &'static [MelodyNote] = &[
        MelodyNote { note: 60, duration_beats: 1.0 },  // C4
        MelodyNote { note: 60, duration_beats: 1.0 },  // C4
        MelodyNote { note: 60, duration_beats: 0.75 }, // C4
        MelodyNote { note: 62, duration_beats: 0.25 }, // D4
        MelodyNote { note: 64, duration_beats: 2.0 },  // E4
        MelodyNote { note: 64, duration_beats: 0.75 }, // E4
        MelodyNote { note: 62, duration_beats: 0.25 }, // D4
        MelodyNote { note: 64, duration_beats: 0.75 }, // E4
        MelodyNote { note: 65, duration_beats: 0.25 }, // F4
        MelodyNote { note: 67, duration_beats: 2.0 },  // G4
        MelodyNote { note: 72, duration_beats: 0.5 },  // C5
        MelodyNote { note: 72, duration_beats: 0.5 },  // C5
        MelodyNote { note: 72, duration_beats: 0.5 },  // C5
        MelodyNote { note: 67, duration_beats: 0.5 },  // G4
        MelodyNote { note: 67, duration_beats: 0.5 },  // G4
        MelodyNote { note: 67, duration_beats: 0.5 },  // G4
        MelodyNote { note: 64, duration_beats: 0.5 },  // E4
        MelodyNote { note: 64, duration_beats: 0.5 },  // E4
        MelodyNote { note: 64, duration_beats: 0.5 },  // E4
        MelodyNote { note: 60, duration_beats: 0.5 },  // C4
        MelodyNote { note: 60, duration_beats: 0.5 },  // C4
        MelodyNote { note: 60, duration_beats: 0.5 },  // C4
        MelodyNote { note: 67, duration_beats: 0.75 }, // G4
        MelodyNote { note: 65, duration_beats: 0.25 }, // F4
        MelodyNote { note: 64, duration_beats: 0.75 }, // E4
        MelodyNote { note: 62, duration_beats: 0.25 }, // D4
        MelodyNote { note: 60, duration_beats: 4.0 },  // C4
    ];

    /// Tempo in BPM
    const TEMPO_BPM: f32 = 120.0;

    /// Calculate samples per beat based on sample rate and tempo
    fn samples_per_beat(&self) -> u32 {
        (self.sample_rate * 60.0 / Self::TEMPO_BPM) as u32
    }

    /// Get the current melody based on the parameter
    fn current_melody(&self) -> &'static [MelodyNote] {
        match self.params.melody.value() {
            MelodyType::JingleBells => Self::JINGLE_BELLS,
            MelodyType::RowRowYourBoat => Self::ROW_ROW_YOUR_BOAT,
        }
    }
}

impl Default for MelodyPlayerParams {
    fn default() -> Self {
        Self {
            melody: EnumParam::new("Melody", MelodyType::JingleBells),
        }
    }
}

impl Default for MelodyPlayer {
    fn default() -> Self {
        Self {
            params: Arc::new(MelodyPlayerParams::default()),
            current_note: 0,
            samples_elapsed: 0,
            active_note: None,
            sample_rate: 44100.0,
            last_melody: MelodyType::JingleBells,
        }
    }
}

impl Plugin for MelodyPlayer {
    const NAME: &'static str = "Ordiseq Melody";
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

        // Transport is playing, generate the melody
        let buffer_len = buffer.samples() as u32;
        let samples_per_beat = self.samples_per_beat();

        // Check if the melody has changed and reset if needed
        let current_melody_type = self.params.melody.value();
        if current_melody_type != self.last_melody {
            // Melody changed - stop any active note and reset playback
            if let Some(note) = self.active_note.take() {
                context.send_event(NoteEvent::NoteOff {
                    timing: 0,
                    voice_id: None,
                    channel: 0,
                    note,
                    velocity: 0.0,
                });
            }
            self.current_note = 0;
            self.samples_elapsed = 0;
            self.last_melody = current_melody_type;
        }

        let mut processed_samples = 0u32;

        // Process note events within this buffer
        while processed_samples < buffer_len {
            let melody = self.current_melody();

            // Safety check: ensure current_note is within bounds
            if self.current_note >= melody.len() {
                self.current_note = 0;
                self.samples_elapsed = 0;
            }

            let current_melody_note = melody[self.current_note];
            let note_duration_samples = (current_melody_note.duration_beats * samples_per_beat as f32) as u32;

            // Start of a new note
            if self.samples_elapsed == 0 {
                // Send note on at the current sample offset
                context.send_event(NoteEvent::NoteOn {
                    timing: processed_samples,
                    voice_id: None,
                    channel: 0,
                    note: current_melody_note.note,
                    velocity: 0.8,
                });
                self.active_note = Some(current_melody_note.note);
            }

            // Calculate how many samples remain in the current note
            let samples_remaining_in_note = note_duration_samples - self.samples_elapsed;
            let samples_remaining_in_buffer = buffer_len - processed_samples;
            let samples_to_process = samples_remaining_in_note.min(samples_remaining_in_buffer);

            self.samples_elapsed += samples_to_process;
            processed_samples += samples_to_process;

            // Check if we've reached the end of the current note
            if self.samples_elapsed >= note_duration_samples {
                // Send note off
                if let Some(note) = self.active_note.take() {
                    context.send_event(NoteEvent::NoteOff {
                        timing: processed_samples.saturating_sub(1),
                        voice_id: None,
                        channel: 0,
                        note,
                        velocity: 0.0,
                    });
                }

                // Move to next note in the melody
                self.current_note = (self.current_note + 1) % melody.len();
                self.samples_elapsed = 0;
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for MelodyPlayer {
    const CLAP_ID: &'static str = "com.enigmacurry.melody-player";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Plays melodies continuously when transport is playing");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::Instrument];
}

impl Vst3Plugin for MelodyPlayer {
    const VST3_CLASS_ID: [u8; 16] = *b"M3l0dyPlayr12345";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument];
}

nih_export_clap!(MelodyPlayer);
nih_export_vst3!(MelodyPlayer);
