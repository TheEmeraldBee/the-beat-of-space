use macroquad::color::{Color};
use macroquad::prelude::{TextParams};
use macroquad::text::{draw_text_ex, Font};
use macroquad::time::get_frame_time;
use crate::note_gameplay_scene::constants::{TEXT_LAST_TIME};

#[derive(PartialEq, Clone)]
pub enum ScoreType {
    Incorrect,
    Miss,
    Score(ScoreQuality)
}

#[derive(PartialEq, Clone)]
pub enum ScoreQuality {
    Perfect,
    Good,
    Ok
}

#[derive(PartialEq, Clone)]
pub struct ScoreText {
    pub timer: f32,
    pub score_type: ScoreType,
    pub y_offset: f32
}

impl ScoreText {
    pub fn update_and_draw(&mut self, text_font: Font) -> bool {
        self.timer -= get_frame_time();

        let text = match self.score_type.clone() {
            ScoreType::Incorrect => { "Incorrect".to_string() }
            ScoreType::Miss => { "Miss".to_string() }
            ScoreType::Score(score) => { match score {
                ScoreQuality::Perfect => { "Perfect".to_string() }
                ScoreQuality::Good => { "Good".to_string() }
                ScoreQuality::Ok => { "Ok".to_string() }
            } }
        };

        let mut color = match self.score_type.clone() {
            ScoreType::Incorrect => { Color::new(0.8, 0.2, 0.2, 1.0) }
            ScoreType::Miss => { Color::new(0.8, 0.0, 0.2, 1.0) }
            ScoreType::Score(score) => {
                match score {
                    ScoreQuality::Perfect => { Color::new(0.0, 1.0, 0.2, 1.0) }
                    ScoreQuality::Good => { Color::new(0.8, 0.2, 0.5, 1.0) }
                    ScoreQuality::Ok => { Color::new(0.6, 0.2, 0.2, 1.0) }
                }
            }
        };

        color.a = self.timer / TEXT_LAST_TIME;

        let complete_percent = TEXT_LAST_TIME / self.timer;

        let x_pos = 50.0 + (complete_percent * 15.0);
        let y_pos = self.y_offset + (complete_percent * 8.0);

        draw_text_ex(text.as_str(), x_pos, y_pos, TextParams {
            font: text_font,
            font_size: 100,
            font_scale: 0.25,
            color,
            ..Default::default()
        });

        self.timer <= 0.0
    }
}