use nih_plug::prelude::*;
use std::sync::Arc;

/// A MIDI effect that inverts notes around a center point
struct MidiInverter {
    params: Arc<MidiInverterParams>,
}

#[derive(Params)]
struct MidiInverterParams {
    #[id = "center"]
    pub center_note: IntParam,
}

impl Default for MidiInverterParams {
    fn default() -> Self {
        Self {
            center_note: IntParam::new(
                "Center Note",
                60, // Middle C
                IntRange::Linear { min: 0, max: 127 },
            )
            .with_unit(" (C4 = 60)")
            .with_value_to_string(formatters::v2s_i32_note_formatter())
            .with_string_to_value(formatters::s2v_i32_note_formatter()),
        }
    }
}

impl Default for MidiInverter {
    fn default() -> Self {
        Self {
            params: Arc::new(MidiInverterParams::default()),
        }
    }
}

impl Plugin for MidiInverter {
    const NAME: &'static str = "Ordiseq Inverter";
    const VENDOR: &'static str = "EnigmaCurry";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "ryan@enigmacurry.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // This plugin doesn't have any audio IO
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: None,
        main_output_channels: None,
        aux_input_ports: &[],
        aux_output_ports: &[],
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::Basic;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn process(
        &mut self,
        _buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Process incoming MIDI events
        while let Some(event) = context.next_event() {
            match event {
                NoteEvent::NoteOn {
                    timing,
                    voice_id,
                    channel,
                    note,
                    velocity,
                } => {
                    let center = self.params.center_note.value() as u8;
                    let inverted_note = invert_note(note, center);

                    context.send_event(NoteEvent::NoteOn {
                        timing,
                        voice_id,
                        channel,
                        note: inverted_note,
                        velocity,
                    });
                }
                NoteEvent::NoteOff {
                    timing,
                    voice_id,
                    channel,
                    note,
                    velocity,
                } => {
                    let center = self.params.center_note.value() as u8;
                    let inverted_note = invert_note(note, center);

                    context.send_event(NoteEvent::NoteOff {
                        timing,
                        voice_id,
                        channel,
                        note: inverted_note,
                        velocity,
                    });
                }
                NoteEvent::PolyPressure {
                    timing,
                    voice_id,
                    channel,
                    note,
                    pressure,
                } => {
                    let center = self.params.center_note.value() as u8;
                    let inverted_note = invert_note(note, center);

                    context.send_event(NoteEvent::PolyPressure {
                        timing,
                        voice_id,
                        channel,
                        note: inverted_note,
                        pressure,
                    });
                }
                // Pass through other MIDI events unchanged
                _ => context.send_event(event),
            }
        }

        ProcessStatus::Normal
    }
}

/// Invert a note around a center point
fn invert_note(note: u8, center: u8) -> u8 {
    let distance = note as i16 - center as i16;
    let inverted = center as i16 - distance;
    inverted.clamp(0, 127) as u8
}

impl ClapPlugin for MidiInverter {
    const CLAP_ID: &'static str = "com.enigmacurry.midi-inverter";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Inverts MIDI notes around a center point");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::NoteEffect,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for MidiInverter {
    const VST3_CLASS_ID: [u8; 16] = *b"M1d1Inv3rt3r1234";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx];
}

nih_export_clap!(MidiInverter);
nih_export_vst3!(MidiInverter);
