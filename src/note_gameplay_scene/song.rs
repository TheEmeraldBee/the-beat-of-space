use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Song {
    pub song_filepath: String,
    pub song_length: f32,
    pub bpm: f32,
    pub credits: String,
    pub notes: Vec<(f32, f32, f32)>
}
