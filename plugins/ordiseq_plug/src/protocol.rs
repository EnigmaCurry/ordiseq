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
    },
    ClipAck {
        clip_id: String,
    },
}

// -- App -> Plugin --

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AppMessage {
    Clip {
        clip_id: String,
        clip: MidiClip,
    },
    Rename {
        name: String,
    },
    Ping,
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
