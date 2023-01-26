use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Song {
    pub song_filepath: String,
    pub song_length: f32,
    pub bpm: f32,
    pub credits: String,
    pub high_score: i32,
    // Beat, Type, Hold Length
    pub notes: Vec<(f32, f32, f32)>,
    // Beat, Last Time, Type
    pub attacks: Vec<(f32, f32, f32)>
}
