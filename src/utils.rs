use macroquad::prelude::FilterMode;
use macroquad::prelude::Texture2D;
use macroquad::texture::load_texture;

pub async fn quick_load_texture(path: &str) -> Texture2D {
    let texture = load_texture(path).await.unwrap();
    texture.set_filter(FilterMode::Nearest);
    texture
}