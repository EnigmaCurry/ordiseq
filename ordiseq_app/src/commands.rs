use ordiseq::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

static TEMP_FILES: Mutex<Vec<tempfile::NamedTempFile>> = Mutex::new(Vec::new());

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
