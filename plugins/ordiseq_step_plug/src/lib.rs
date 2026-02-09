use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, widgets, EguiState};
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, RwLock,
};

#[cfg(windows)]
mod midi_drag {
    use ordiseq::prelude::*;

    /// Convert a MIDI note number to a klib Note
    fn note_from_midi(midi: u8) -> Note {
        let octave = (midi / 12).saturating_sub(1);
        let pitch = midi % 12;
        Note::new(
            NamedPitch::from(Pitch::try_from(pitch).unwrap()),
            Octave::try_from(octave).unwrap(),
        )
    }

    /// Generate MIDI file bytes from a step pattern using ordiseq's Sequence
    pub fn generate_midi_bytes(
        pattern: [[bool; 16]; 4],
        track_midi_notes: [u8; 4],
        velocity: f32,
        gate_length: f32,
    ) -> Vec<u8> {
        let time_sig = common_time(); // 4/4, 96 TPQN
        let mut seq = Sequence::new("Ordiseq Step", time_sig).unwrap();

        let ticks_per_16th: u32 = 24; // 96 TPQN / 4 subdivisions per beat
        let gate_ticks = (ticks_per_16th as f32 * gate_length).max(1.0) as u32;
        let gate_time = Time { ticks: gate_ticks };

        let notes: Vec<Note> = track_midi_notes
            .iter()
            .map(|&midi| note_from_midi(midi))
            .collect();

        for step in 0..16usize {
            let time = Time {
                ticks: step as u32 * ticks_per_16th,
            };
            let chord_notes: Vec<(Note, f32, Time)> = (0..4)
                .filter(|&track| pattern[track][step])
                .map(|track| (notes[track].clone(), velocity, gate_time))
                .collect();

            if !chord_notes.is_empty() {
                seq.add_chord(time, chord_notes);
            }
        }

        let smf = seq.to_midi();
        let mut buffer = Vec::new();
        smf.write_std(&mut buffer).unwrap();
        buffer
    }

    // --- Windows COM drag-and-drop ---

    use windows::core::*;
    use windows::Win32::Foundation::*;
    use windows::Win32::System::Com::*;
    use windows::Win32::System::Memory::*;
    use windows::Win32::System::Ole::*;
    use windows::Win32::System::SystemServices::MODIFIERKEYS_FLAGS;
    use windows::Win32::UI::Shell::DROPFILES;

    const CF_HDROP_VALUE: u16 = 15;

    // --- IEnumFORMATETC implementation (advertises CF_HDROP to drop targets) ---

    #[implement(IEnumFORMATETC)]
    struct HDropFormatEnum {
        index: std::cell::Cell<usize>,
    }

    impl IEnumFORMATETC_Impl for HDropFormatEnum {
        fn Next(
            &self,
            celt: u32,
            rgelt: *mut FORMATETC,
            pceltfetched: *mut u32,
        ) -> HRESULT {
            unsafe {
                if celt == 0 {
                    return S_OK;
                }

                let idx = self.index.get();
                if idx >= 1 {
                    if !pceltfetched.is_null() {
                        *pceltfetched = 0;
                    }
                    return S_FALSE;
                }

                *rgelt = FORMATETC {
                    cfFormat: CF_HDROP_VALUE,
                    ptd: std::ptr::null_mut(),
                    dwAspect: 1, // DVASPECT_CONTENT
                    lindex: -1,
                    tymed: 1, // TYMED_HGLOBAL
                };

                self.index.set(1);

                if !pceltfetched.is_null() {
                    *pceltfetched = 1;
                }

                if celt == 1 { S_OK } else { S_FALSE }
            }
        }

        fn Skip(&self, celt: u32) -> Result<()> {
            let new_idx = self.index.get() + celt as usize;
            self.index.set(new_idx.min(1));
            Ok(())
        }

        fn Reset(&self) -> Result<()> {
            self.index.set(0);
            Ok(())
        }

        fn Clone(&self) -> Result<IEnumFORMATETC> {
            let clone: IEnumFORMATETC = HDropFormatEnum {
                index: std::cell::Cell::new(self.index.get()),
            }
            .into();
            Ok(clone)
        }
    }

    // --- IDropSource implementation ---

    #[implement(IDropSource)]
    struct FileDropSource;

    impl IDropSource_Impl for FileDropSource {
        fn QueryContinueDrag(
            &self,
            fescapepressed: BOOL,
            grfkeystate: MODIFIERKEYS_FLAGS,
        ) -> HRESULT {
            if fescapepressed.as_bool() {
                DRAGDROP_S_CANCEL
            } else if !grfkeystate.contains(MODIFIERKEYS_FLAGS(0x0001)) {
                // MK_LBUTTON released
                DRAGDROP_S_DROP
            } else {
                S_OK
            }
        }

        fn GiveFeedback(&self, _dweffect: DROPEFFECT) -> HRESULT {
            DRAGDROP_S_USEDEFAULTCURSORS
        }
    }

    #[implement(IDataObject)]
    struct FileDataObject {
        file_path: String,
    }

    impl IDataObject_Impl for FileDataObject {
        fn GetData(&self, pformatetcin: *const FORMATETC) -> Result<STGMEDIUM> {
            unsafe {
                let fmt = &*pformatetcin;
                if fmt.cfFormat != CF_HDROP_VALUE {
                    return Err(Error::from(DV_E_FORMATETC));
                }

                // Build UTF-16 path with double null terminator
                let path_wide: Vec<u16> = self
                    .file_path
                    .encode_utf16()
                    .chain(std::iter::once(0)) // null terminator for path
                    .chain(std::iter::once(0)) // double null terminator for list
                    .collect();

                let dropfiles_size = std::mem::size_of::<DROPFILES>();
                let total_size = dropfiles_size + path_wide.len() * 2;

                let hglobal = GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, total_size)?;
                let ptr = GlobalLock(hglobal);
                if ptr.is_null() {
                    let _ = GlobalFree(hglobal);
                    return Err(Error::from(E_OUTOFMEMORY));
                }

                // Fill DROPFILES header
                let dropfiles = ptr as *mut DROPFILES;
                (*dropfiles).pFiles = dropfiles_size as u32;
                (*dropfiles).fWide = TRUE;

                // Copy file path after DROPFILES header
                let str_dest = (ptr as *mut u8).add(dropfiles_size) as *mut u16;
                std::ptr::copy_nonoverlapping(path_wide.as_ptr(), str_dest, path_wide.len());

                let _ = GlobalUnlock(hglobal);

                let mut medium: STGMEDIUM = std::mem::zeroed();
                medium.tymed = TYMED_HGLOBAL.0 as u32;
                medium.u.hGlobal = hglobal;
                Ok(medium)
            }
        }

        fn GetDataHere(
            &self,
            _pformatetc: *const FORMATETC,
            _pmedium: *mut STGMEDIUM,
        ) -> Result<()> {
            Err(Error::from(E_NOTIMPL))
        }

        fn QueryGetData(&self, pformatetc: *const FORMATETC) -> HRESULT {
            unsafe {
                let fmt = &*pformatetc;
                if fmt.cfFormat == CF_HDROP_VALUE
                    && (fmt.tymed & 1) != 0 // TYMED_HGLOBAL
                {
                    S_OK
                } else {
                    DV_E_FORMATETC
                }
            }
        }

        fn GetCanonicalFormatEtc(
            &self,
            _pformatectin: *const FORMATETC,
            _pformatetcout: *mut FORMATETC,
        ) -> HRESULT {
            E_NOTIMPL
        }

        fn SetData(
            &self,
            _pformatetc: *const FORMATETC,
            _pmedium: *const STGMEDIUM,
            _frelease: BOOL,
        ) -> Result<()> {
            Err(Error::from(E_NOTIMPL))
        }

        fn EnumFormatEtc(&self, dwdirection: u32) -> Result<IEnumFORMATETC> {
            if dwdirection == 1 {
                // DATADIR_GET
                let enumerator: IEnumFORMATETC = HDropFormatEnum {
                    index: std::cell::Cell::new(0),
                }
                .into();
                Ok(enumerator)
            } else {
                Err(Error::from(E_NOTIMPL))
            }
        }

        fn DAdvise(
            &self,
            _pformatetc: *const FORMATETC,
            _advf: u32,
            _padvsink: Option<&IAdviseSink>,
        ) -> Result<u32> {
            Err(Error::from(OLE_E_ADVISENOTSUPPORTED))
        }

        fn DUnadvise(&self, _dwconnection: u32) -> Result<()> {
            Err(Error::from(OLE_E_ADVISENOTSUPPORTED))
        }

        fn EnumDAdvise(&self) -> Result<IEnumSTATDATA> {
            Err(Error::from(OLE_E_ADVISENOTSUPPORTED))
        }
    }

    // --- Deferred drag via SetTimer ---
    // DoDragDrop must run on the GUI thread (for mouse state) but NOT inside
    // the egui callback (re-entrancy crash). SetTimer defers it to the next
    // message loop iteration, after the egui frame has completed.

    use windows::Win32::UI::WindowsAndMessaging::{KillTimer, SetTimer};

    static DRAG_FILE_PATH: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

    unsafe extern "system" fn drag_timer_proc(
        _hwnd: HWND,
        _msg: u32,
        id_event: usize,
        _dw_time: u32,
    ) {
        unsafe {
            let _ = KillTimer(HWND(0), id_event);
        }

        let path = DRAG_FILE_PATH.lock().unwrap().take();
        if let Some(file_path) = path {
            unsafe {
                let _ = OleInitialize(None);

                let data_object: IDataObject = FileDataObject {
                    file_path,
                }
                .into();
                let drop_source: IDropSource = FileDropSource.into();

                let mut effect = DROPEFFECT(0);
                let _ = DoDragDrop(
                    &data_object,
                    &drop_source,
                    DROPEFFECT_COPY,
                    &mut effect,
                );
            }
        }
    }

    /// Initiate an OS-level file drag, deferred to next message loop iteration.
    pub fn start_file_drag(file_path: &str) {
        *DRAG_FILE_PATH.lock().unwrap() = Some(file_path.to_string());
        unsafe {
            SetTimer(HWND(0), 0, 1, Some(drag_timer_proc));
        }
    }
}

/// A 4-track, 16-step sequencer with interactive GUI
struct OrdiseqStep {
    params: Arc<OrdiseqStepParams>,

    // Timing (same as Euclid/Topograph)
    current_step: usize,
    samples_elapsed: u32,
    sample_rate: f32,

    // Transport tracking
    was_playing: bool,

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
            editor_state: EguiState::from_size(700, 430),
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
            was_playing: false,
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
                        let pattern_idx = params.current_pattern.value() as usize;

                        // Pattern grid (write lock scoped to this block)
                        {
                            let mut pattern_state =
                                params.pattern_state.write().unwrap();

                            // Render 4 track rows (Track 4 at top, Track 1 at bottom)
                            for track_idx in (0..4).rev() {
                                ui.horizontal(|ui| {
                                    ui.label(format!("Track {}", track_idx + 1));
                                    ui.add_space(5.0);

                                    // 16 checkboxes per track (grouped in sets of 4)
                                    for step_idx in 0..16 {
                                        let is_current = step_idx == current_step;
                                        let is_active = &mut pattern_state.patterns
                                            [pattern_idx][track_idx][step_idx];

                                        let checkbox =
                                            egui::Checkbox::new(is_active, "");
                                        let response =
                                            ui.add_sized([20.0, 20.0], checkbox);

                                        // Highlight current step
                                        if is_current {
                                            ui.painter().rect_stroke(
                                                response.rect,
                                                0.0,
                                                egui::Stroke::new(
                                                    2.0,
                                                    egui::Color32::YELLOW,
                                                ),
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
                                    if ui
                                        .button(format!(
                                            "Clear Track {}",
                                            track_idx + 1
                                        ))
                                        .clicked()
                                    {
                                        pattern_state.patterns[pattern_idx]
                                            [track_idx] = [false; 16];
                                    }
                                });
                                ui.add_space(5.0);
                            }
                        } // write lock dropped

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);

                        // Parameters
                        ui.horizontal(|ui| {
                            ui.label("Velocity:");
                            ui.add(widgets::ParamSlider::for_param(
                                &params.velocity,
                                setter,
                            ));

                            ui.add_space(20.0);

                            ui.label("Gate Length:");
                            ui.add(widgets::ParamSlider::for_param(
                                &params.gate_length,
                                setter,
                            ));
                        });

                        // MIDI drag-and-drop export widget
                        #[cfg(windows)]
                        {
                            ui.add_space(10.0);
                            ui.separator();
                            ui.add_space(10.0);

                            ui.horizontal(|ui| {
                                let (rect, response) = ui.allocate_exact_size(
                                    egui::vec2(200.0, 40.0),
                                    egui::Sense::drag(),
                                );

                                let bg_color = if response.hovered() {
                                    egui::Color32::from_rgb(70, 70, 90)
                                } else {
                                    egui::Color32::from_rgb(50, 50, 70)
                                };

                                ui.painter().rect(
                                    rect,
                                    4.0,
                                    bg_color,
                                    egui::Stroke::new(
                                        1.0,
                                        egui::Color32::from_rgb(120, 120, 140),
                                    ),
                                    egui::epaint::StrokeKind::Outside,
                                );

                                ui.painter().text(
                                    rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    "Drag MIDI to DAW",
                                    egui::FontId::proportional(14.0),
                                    egui::Color32::WHITE,
                                );

                                if response.drag_started() {
                                    // Read pattern (re-acquire read lock)
                                    let pattern = {
                                        let ps =
                                            params.pattern_state.read().unwrap();
                                        ps.patterns[pattern_idx]
                                    };
                                    let velocity = params.velocity.value();
                                    let gate_length = params.gate_length.value();

                                    let midi_bytes =
                                        midi_drag::generate_midi_bytes(
                                            pattern,
                                            OrdiseqStep::TRACK_NOTES,
                                            velocity,
                                            gate_length,
                                        );

                                    let temp_path = std::env::temp_dir()
                                        .join("ordiseq_step_export.mid");
                                    if std::fs::write(&temp_path, &midi_bytes)
                                        .is_ok()
                                    {
                                        midi_drag::start_file_drag(
                                            &temp_path.to_string_lossy(),
                                        );
                                    }
                                }
                            });
                        }
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
            self.was_playing = false;
            return ProcessStatus::Normal;
        }

        // Reset position when transport starts playing
        if !self.was_playing {
            self.current_step = 0;
            self.samples_elapsed = 0;
            self.current_step_display.store(0, Ordering::Relaxed);
            self.was_playing = true;
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
