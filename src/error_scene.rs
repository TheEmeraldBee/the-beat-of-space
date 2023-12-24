use async_trait::async_trait;
use macroquad::prelude::*;
use macroquad_aspect::prelude::WindowContext;
use macroquad_aspect::window::draw_window;
use crate::main_menu_scene::MainMenuScene;
use crate::scene::Scene;
use crate::ui::draw_text_justified;

pub struct ErrorScene {
    error: String,
    window_context: WindowContext
}

impl ErrorScene {
    pub fn new(error: &str, window_context: WindowContext) -> Self {
        Self {
            error: error.to_string(),
            window_context
        }
    }
}

#[async_trait]
impl Scene for ErrorScene {
    async fn run(&mut self) -> Option<Box<dyn Scene>> {
        loop {
            set_camera(&self.window_context.camera);

            clear_background(BLACK);
            draw_text_justified(&format!("Error: {}", self.error),
                                vec2(self.window_context.active_screen_size.x / 2.0, self.window_context.active_screen_size.y / 2.0),
                                TextParams {
                                    font: Default::default(),
                                    font_size: 100,
                                    font_scale: 0.25,
                                    color: WHITE,
                                    ..Default::default()
                                },
                                vec2(0.5, 0.5));
            draw_text_justified(&"Space: Reload, Escape: Quit Game",
                                vec2(self.window_context.active_screen_size.x / 2.0, self.window_context.active_screen_size.y / 2.0 + 150.0),
                                TextParams {
                                    font: Default::default(),
                                    font_size: 50,
                                    font_scale: 0.25,
                                    color: WHITE,
                                    ..Default::default()
                                },
                                vec2(0.5, 0.5));

            if is_key_pressed(KeyCode::Escape) {
                return None
            }

            if is_key_pressed(KeyCode::Space) {
                return Some(Box::new(MainMenuScene {
                    window_context: self.window_context.clone()
                }))
            }

            draw_window(&mut self.window_context);

            next_frame().await
        }
    }
}