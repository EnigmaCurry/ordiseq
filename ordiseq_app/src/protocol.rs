use serde::{Deserialize, Serialize};

// -- Plugin -> App --

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PluginMessage {
    Register {
        name: String,
        version: String,
    },
    Transport {
        bpm: f32,
        playing: bool,
        #[serde(default)]
        program: u8,
    },
    ClipAck {
        clip_id: String,
    },
    TimeSync {
        beat_position: f64,
        bpm: f64,
        playing: bool,
    },
}

// -- App -> Plugin --

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AppMessage {
    Clip {
        clip_id: String,
        clip: MidiClip,
        #[serde(default)]
        program: u8,
    },
    Rename {
        name: String,
    },
    Ping,
    StartSync,
    StopSync,
}

// -- Shared Types --

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MidiClip {
    pub name: String,
    pub length_beats: f32,
    pub notes: Vec<ClipNote>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClipNote {
    pub note: u8,
    pub channel: u8,
    pub velocity: f32,
    pub start_beats: f32,
    pub duration_beats: f32,
}
