use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct World {
    pub name: String,
    pub difficulty: String,
    pub levels: Vec<String>
}