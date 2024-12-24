// use klib::core::chord::Chord;
// use klib::core::named_pitch::NamedPitch;
// use serde::Deserialize;
// use serde_json::json;
// use std::collections::HashMap;
// use std::str::FromStr;
// use std::sync::OnceLock;

// #[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
// pub struct Scale {
//     pub name: String,
//     pub intervals: Vec<u8>,
//     pub pitches: Vec<NamedPitch>,
//     pub origin: Option<String>,
// }

// impl Scale {
//     pub fn all_notes(&self, duration: f64) -> Chord {
//         let notes = self
//             .pitches
//             .iter()
//             .map(|pitch| MusicalNote {
//                 note: Note::new(pitch.clone(), 4),
//                 duration: 1.0,
//                 velocity: 0.7,
//             })
//             .collect();
//         Chord { notes, duration }
//     }
// }

// static SCALES: OnceLock<Vec<Scale>> = OnceLock::new();

// fn load_scales() -> &'static Vec<Scale> {
//     SCALES.get_or_init(|| {
//         // Load the YAML data at compile time
//         const SCALES_YAML: &str = include_str!("../../data/scales.yaml");

//         // Parse the YAML data into a vector of Scale structs
//         let scales: Vec<serde_yaml::Value> =
//             serde_yaml::from_str(SCALES_YAML).expect("Invalid YAML format");

//         scales
//             .into_iter()
//             .filter_map(|item| {
//                 // Check for required fields and skip entries with ascending/descending keys
//                 if item.get("intervals_ascending").is_some()
//                     || item.get("notes_ascending").is_some()
//                 {
//                     return None;
//                 }

//                 let name = item.get("name")?.as_str()?.to_string();
//                 let intervals = item
//                     .get("intervals")?
//                     .as_sequence()?
//                     .iter()
//                     .filter_map(|i| i.as_u64().map(|n| n as u8))
//                     .collect::<Vec<u8>>();
//                 let notes = item
//                     .get("notes")?
//                     .as_sequence()?
//                     .iter()
//                     .filter_map(|n| n.as_u64().and_then(|n| Pitch::from_repr(n as usize)))
//                     .collect::<Vec<Pitch>>();
//                 let origin = item
//                     .get("origin")
//                     .and_then(|o| o.as_str().map(|s| s.to_string()));

//                 Some(Scale {
//                     name,
//                     intervals,
//                     pitches: notes,
//                     origin,
//                 })
//             })
//             .collect()
//     })
// }

// pub fn get_scale(name: &str) -> Result<&Scale, OrdiseqError> {
//     load_scales()
//         .into_iter()
//         .find(|scale| scale.name.to_lowercase() == name.to_lowercase())
//         .ok_or_else(|| OrdiseqError::ScaleNotFoundError(name.to_string()))
// }

// pub fn get_scale_names() -> Vec<String> {
//     load_scales()
//         .iter()
//         .map(|scale| scale.name.clone())
//         .collect()
// }

// pub fn get_scale_sequence(
//     scale: &Scale,
//     time_signature: &str,
// ) -> Result<MusicSequence, OrdiseqError> {
//     let octave = 4;
//     let duration = 1; // quarter note
//     let velocity = 0.7;

//     let mut sequence = MusicSequence::new(&scale.name, time_signature);
//     for pitch in scale.pitches.clone() {
//         sequence = sequence.add_note(Note::new(pitch, octave), duration, velocity)?;
//     }
//     // Add the final note as the octave above the first pitch:
//     sequence = sequence.add_note(
//         Note::new(scale.pitches[0].clone(), octave + 1),
//         duration,
//         velocity,
//     )?;
//     Ok(sequence)
// }

// pub fn filter_scales<F>(filter: F) -> Result<Vec<Scale>, OrdiseqError>
// where
//     F: Fn(&Scale) -> bool,
// {
//     Ok(get_scale_names()
//         .into_iter()
//         .filter_map(|name| match get_scale(&name) {
//             Ok(scale) if filter(&scale) => Some(scale.clone()),
//             _ => None,
//         })
//         .collect::<Vec<Scale>>())
// }
