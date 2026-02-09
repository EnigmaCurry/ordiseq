use ordiseq::prelude::*;
use ordiseq::synth::{Player, SoundFontSource};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::protocol::MidiClip;

static TEMP_FILES: Mutex<Vec<tempfile::NamedTempFile>> = Mutex::new(Vec::new());
static STOP_FLAG: Mutex<Option<Arc<AtomicBool>>> = Mutex::new(None);
static PLAYBACK_ACTIVE: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Deserialize)]
pub struct GenerateMidiParams {
    pub sequence_type: String,
}

#[derive(Debug, Serialize)]
pub struct MidiResult {
    pub path: String,
    pub title: String,
    pub note_count: usize,
}

#[tauri::command]
pub fn generate_midi(params: GenerateMidiParams) -> Result<MidiResult, String> {
    let time_signature = common_time();

    let (title, notes) = match params.sequence_type.as_str() {
        "c_major_scale" => build_c_major_scale(),
        "simple_melody" => build_simple_melody(),
        _ => return Err(format!("Unknown sequence type: {}", params.sequence_type)),
    };

    let mut seq =
        Sequence::new(&title, time_signature).map_err(|e| format!("Failed to create sequence: {:?}", e))?;

    seq.load(&notes)
        .map_err(|e| format!("Failed to load notes: {}", e))?;

    let note_count = notes.iter().filter(|(n, _, _, _)| !matches!(n, NoteOrRest::Rest)).count();

    let temp_file = tempfile::Builder::new()
        .prefix("ordiseq_")
        .suffix(".mid")
        .tempfile()
        .map_err(|e| format!("Failed to create temp file: {}", e))?;

    let path = temp_file.path().to_path_buf();

    seq.to_midi()
        .save(&path)
        .map_err(|e| format!("Failed to save MIDI: {}", e))?;

    let path_str = path.to_string_lossy().to_string();

    // Keep temp file alive
    TEMP_FILES.lock().unwrap().push(temp_file);

    Ok(MidiResult {
        path: path_str,
        title,
        note_count,
    })
}

fn build_c_major_scale() -> (String, Vec<(NoteOrRest, u32, f32, f32)>) {
    let title = "C Major Scale".to_string();
    let v = 0.7;
    let r = 0.8;

    let scale_notes = ["C4", "D4", "E4", "F4", "G4", "A4", "B4", "C5"];
    let notes: Vec<(NoteOrRest, u32, f32, f32)> = scale_notes
        .iter()
        .filter_map(|s| Note::parse(s).ok())
        .map(|note| (NoteOrRest::Note(note), 2, v, r))
        .collect();

    (title, notes)
}

fn build_simple_melody() -> (String, Vec<(NoteOrRest, u32, f32, f32)>) {
    let title = "Simple Melody".to_string();
    let v = 0.7;
    let r = 0.5;

    let melody = vec![
        ("E4", 2),
        ("E4", 2),
        ("E4", 4),
        ("E4", 2),
        ("E4", 2),
        ("E4", 4),
        ("E4", 2),
        ("G4", 2),
        ("C4", 3),
        ("D4", 1),
        ("E4", 8),
    ];

    let notes: Vec<(NoteOrRest, u32, f32, f32)> = melody
        .into_iter()
        .filter_map(|(s, beats)| Note::parse(s).ok().map(|note| (NoteOrRest::Note(note), beats, v, r)))
        .collect();

    (title, notes)
}

#[derive(Debug, Deserialize)]
pub struct PlayMidiParams {
    pub midi_path: String,
}

#[derive(Debug, Serialize)]
pub struct PlaybackStatus {
    pub playing: bool,
}

#[tauri::command]
pub fn play_midi(params: PlayMidiParams) -> Result<(), String> {
    // Check if already playing
    if PLAYBACK_ACTIVE.load(Ordering::SeqCst) {
        return Err("Playback already in progress".to_string());
    }

    // Read the MIDI file
    let midi_data =
        std::fs::read(&params.midi_path).map_err(|e| format!("Failed to read MIDI file: {}", e))?;

    // Find a soundfont
    let soundfont = SoundFontSource::default_search().map_err(|e| format!("No soundfont found: {:?}", e))?;

    // Create stop flag
    let stop_flag = Arc::new(AtomicBool::new(false));
    *STOP_FLAG.lock().unwrap() = Some(Arc::clone(&stop_flag));

    // Set playback active
    PLAYBACK_ACTIVE.store(true, Ordering::SeqCst);

    // Spawn playback in a separate thread
    std::thread::spawn(move || {
        let result = (|| -> Result<(), String> {
            let player = Player::new(soundfont).map_err(|e| format!("Failed to create player: {:?}", e))?;
            player
                .play_midi_bytes_stoppable(&midi_data, &stop_flag)
                .map_err(|e| format!("Playback error: {:?}", e))?;
            Ok(())
        })();

        // Clear playback state
        PLAYBACK_ACTIVE.store(false, Ordering::SeqCst);
        *STOP_FLAG.lock().unwrap() = None;

        if let Err(e) = result {
            eprintln!("Playback error: {}", e);
        }
    });

    Ok(())
}

#[tauri::command]
pub fn stop_midi() -> Result<(), String> {
    if let Some(stop_flag) = STOP_FLAG.lock().unwrap().as_ref() {
        stop_flag.store(true, Ordering::SeqCst);
    }
    Ok(())
}

#[tauri::command]
pub fn get_playback_status() -> PlaybackStatus {
    PlaybackStatus {
        playing: PLAYBACK_ACTIVE.load(Ordering::SeqCst),
    }
}

#[tauri::command]
pub fn clip_to_midi_file(clip: MidiClip) -> Result<MidiResult, String> {
    const REPEATS: u32 = 2;
    let mut raw_notes = Vec::with_capacity(clip.notes.len() * REPEATS as usize);
    for rep in 0..REPEATS {
        let offset = rep as f32 * clip.length_beats;
        for n in &clip.notes {
            raw_notes.push(RawClipNote {
                note: n.note,
                channel: n.channel,
                velocity: n.velocity,
                start_beats: n.start_beats + offset,
                duration_beats: n.duration_beats,
            });
        }
    }

    let smf = clip_to_midi(clip.length_beats * REPEATS as f32, &raw_notes);

    let temp_file = tempfile::Builder::new()
        .prefix("ordiseq_")
        .suffix(".mid")
        .tempfile()
        .map_err(|e| format!("Failed to create temp file: {}", e))?;

    let path = temp_file.path().to_path_buf();

    smf.save(&path)
        .map_err(|e| format!("Failed to save MIDI: {}", e))?;

    let path_str = path.to_string_lossy().to_string();
    TEMP_FILES.lock().unwrap().push(temp_file);

    Ok(MidiResult {
        path: path_str,
        title: clip.name,
        note_count: clip.notes.len(),
    })
}
