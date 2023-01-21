use macroquad::prelude::FilterMode;
use macroquad::prelude::Texture2D;
use macroquad::texture::load_texture;
use macroquad::time::get_frame_time;

pub async fn quick_load_texture(path: &str) -> Texture2D {
    let texture = load_texture(path).await.unwrap();
    texture.set_filter(FilterMode::Nearest);
    texture
}


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Timer<T> {
    pub running: bool,
    pub timer: f32,
    max_timer: f32,
    pub data: T
}

impl<T> Timer<T> where T: Send {
    pub fn new(time: f32, data: T) -> Self {
        Self {
            running: false,
            timer: time,
            max_timer: time,
            data
        }
    }

    pub fn start(&mut self) {
        self.running = true;
    }

    pub fn update(&mut self) {
        if self.running && self.timer >= 0.0 {
            self.timer -= get_frame_time()
        }
    }

    pub fn is_done(&self) -> bool {
        self.timer <= 0.0
    }

    pub fn percent_done(&self) -> f32 {
        1.0 - (self.timer / self.max_timer)
    }
}