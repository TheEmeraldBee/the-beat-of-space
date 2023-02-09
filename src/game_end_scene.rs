use async_trait::async_trait;
use macroquad::prelude::*;
use macroquad_aspect::prelude::{draw_window, WindowContext};
use thousands::Separable;
use crate::main_menu_scene::MainMenuScene;
use crate::note_gameplay_scene::{NoteGameplayScene, ReturnTo};
use crate::ui::*;
use crate::utils::quick_load_texture;
use crate::scene::Scene;

pub struct GameEndScene {
    pub return_to: ReturnTo,
    pub window_context: WindowContext,
    pub file_path: String,
    pub beat_level: bool,
    pub score: i32,
    pub perfect_notes: i32,
    pub good_notes: i32,
    pub ok_notes: i32,
    pub incorrect_notes: i32,
    pub missed_notes: i32
}

#[async_trait]
impl Scene for GameEndScene {
    async fn run(&mut self) -> Option<Box<dyn Scene>> {
        let background = quick_load_texture("assets/images/backgrounds/Space Background (9).png").await;

        let font = load_ttf_font("assets/fonts/pixel.ttf").await.unwrap();

        let frame = quick_load_texture("assets/images/ui/frame.png").await;
        let nine_slice_frame = Element {
            tex: frame,
            element_type: ElementType::NineSlice(vec2(10.0, 10.0))
        };

        let nine_slice_button = Element {
            tex: quick_load_texture("assets/images/ui/button.png").await,
            element_type: ElementType::NineSlice(vec2(10.0, 10.0))
        };

        let button_template = UITemplate::new(
            nine_slice_button,
            Color::new(1.0, 1.0, 1.0, 1.0),
            Some(Color::new(0.8, 0.8, 0.8, 1.0))
        );
        
        loop {
            set_camera(&self.window_context.camera);

            let mouse_pos = self.window_context.camera.screen_to_world(mouse_position().into());

            draw_texture(background, 0.0, 0.0, WHITE);

            nine_slice_frame.draw(justify_rect(self.window_context.active_screen_size.x / 2.0, 20.0, self.window_context.active_screen_size.x - 100.0, 300.0, vec2(0.5, 0.0)), WHITE);

            if element_text_template(
                justify_rect(self.window_context.active_screen_size.x / 4.0, 400.0 - 15.0, 96.0 * 2.0, 26.0 * 2.0, vec2(0.5, 1.0)),
                button_template, mouse_pos, "Done",
                TextParams {
                    font,
                    font_size: 80,
                    font_scale: 0.25,
                    ..Default::default()
                }
            ).clicked() {
                let (difficulty, value) = match self.return_to.clone() {
                    ReturnTo::MainMenu(difficulty, value) => { (Some(difficulty), Some(value)) }
                    ReturnTo::Editor => { (None, None) }
                };

                return Some(Box::new(MainMenuScene {
                    window_context: self.window_context.clone(),
                    selected_difficulty: difficulty,
                    selected_song_idx: value
                }));
            }

            if element_text_template(
                justify_rect(self.window_context.active_screen_size.x - self.window_context.active_screen_size.x / 4.0, 400.0 - 15.0, 96.0 * 2.0, 26.0 * 2.0, vec2(0.5, 1.0)),
                button_template, mouse_pos, "Retry",
                TextParams {
                    font,
                    font_size: 80,
                    font_scale: 0.25,
                    ..Default::default()
                }
            ).clicked() {
                return Some(Box::new(NoteGameplayScene::new(self.window_context.clone(), &self.file_path.clone(), self.return_to.clone())));
            }

            let status_text = match self.beat_level {
                true => { "Ship Escaped" }
                false => { "Ship Destroyed" }
            };

            draw_text_justified(status_text, vec2(self.window_context.active_screen_size.x / 2.0, 30.0), TextParams {
                font,
                font_size: 100,
                font_scale: 0.25,
                color: WHITE,
                ..Default::default()
            }, vec2(0.5, 1.0));

            draw_text_justified(&format!("Score: {}", self.score.separate_with_commas()), vec2(70.0, 60.0), TextParams {
                font,
                font_size: 70,
                font_scale: 0.25,
                color: LIGHTGRAY,
                ..Default::default()
            }, vec2(0.0, 1.0));

            draw_text_justified("NOTES", vec2(70.0, 100.0), TextParams {
                font,
                font_size: 90,
                font_scale: 0.25,
                color: WHITE,
                ..Default::default()
            }, vec2(0.0, 1.0));

            draw_text_justified(&format!("Perfect: {}", self.perfect_notes.separate_with_commas()), vec2(70.0, 140.0), TextParams {
                font,
                font_size: 70,
                font_scale: 0.25,
                color: LIGHTGRAY,
                ..Default::default()
            }, vec2(0.0, 1.0));

            draw_text_justified(&format!("Good: {}", self.good_notes.separate_with_commas()), vec2(70.0, 180.0), TextParams {
                font,
                font_size: 65,
                font_scale: 0.25,
                color: LIGHTGRAY,
                ..Default::default()
            }, vec2(0.0, 1.0));

            draw_text_justified(&format!("Ok: {}", self.ok_notes.separate_with_commas()), vec2(70.0, 220.0), TextParams {
                font,
                font_size: 60,
                font_scale: 0.25,
                color: LIGHTGRAY,
                ..Default::default()
            }, vec2(0.0, 1.0));

            draw_text_justified(&format!("Incorrect: {}", self.incorrect_notes.separate_with_commas()), vec2(70.0, 260.0), TextParams {
                font,
                font_size: 55,
                font_scale: 0.25,
                color: LIGHTGRAY,
                ..Default::default()
            }, vec2(0.0, 1.0));

            draw_text_justified(&format!("Missed: {}", self.missed_notes.separate_with_commas()), vec2(70.0, 300.0), TextParams {
                font,
                font_size: 50,
                font_scale: 0.25,
                color: LIGHTGRAY,
                ..Default::default()
            }, vec2(0.0, 1.0));
            
            if is_key_pressed(KeyCode::Escape) {
                let (difficulty, idx) = match self.return_to.clone() {
                    ReturnTo::MainMenu(difficulty, idx) => {(Some(difficulty), Some(idx))}
                    ReturnTo::Editor => {(None, None)}
                };

                return Some(Box::new(MainMenuScene {
                    window_context: self.window_context.clone(),
                    selected_difficulty: difficulty,
                    selected_song_idx: idx
                }));
            }

            draw_window(&mut self.window_context);

            next_frame().await;
        }
    }
}