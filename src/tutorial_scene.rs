use std::time::Duration;
use async_trait::async_trait;
use egui_macroquad::egui::Key::P;
use kira::manager::{AudioManager, AudioManagerSettings};
use kira::manager::backend::cpal::CpalBackend;
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use kira::tween::Tween;
use macroquad::prelude::*;
use macroquad_aspect::prelude::*;
use crate::error_scene::ErrorScene;
use crate::main_menu_scene::MainMenuScene;
use crate::note_gameplay_scene::constants::*;
use crate::note_gameplay_scene::{draw_hold, draw_note, NoteGameplayScene};
use crate::note_gameplay_scene::score_texts::ScoreText;
use crate::scene::Scene;
use crate::ui::draw_text_justified;
use crate::utils::{Config, quick_load_texture, u32_to_key_code};

pub struct ScrollingText {
    pub text: String,
    pub interval: f32,
    pub cur_interval: f32,
    pub letters_drawn: usize,
    pub draw_location: Vec2,
    pub draw_justification: Vec2,
    pub text_options: TextParams
}

impl ScrollingText {
    pub fn new(text: String, interval: f32, draw_location: Vec2, draw_justification: Vec2, text_options: TextParams) -> Self {
        Self {
            text,
            interval,
            cur_interval: 0.0,
            letters_drawn: 0,
            draw_location,
            draw_justification,
            text_options
        }
    }

    pub fn update(&mut self) {
        if self.letters_drawn < self.text.len() {
            self.cur_interval += get_frame_time();

            if self.cur_interval > self.interval {
                self.cur_interval = 0.0;
                self.letters_drawn += 1;
            }
        }

        draw_text_justified(&self.text[0..self.letters_drawn], self.draw_location, self.text_options, self.draw_justification);
    }

    pub fn replace_text(&mut self, text: String) {
        self.text = text;
        self.cur_interval = 0.0;
        self.letters_drawn = 0;
    }
}


pub struct TutorialScene {
    pub(crate) window_context: WindowContext
}

#[async_trait]
impl Scene for TutorialScene {
    async fn run(&mut self) -> Option<Box<dyn Scene>> {
        // Fonts
        let font = match load_ttf_font("assets/fonts/pixel.ttf").await {
            Ok(font) => font,
            Err(_) => return Some(Box::new(ErrorScene::new("Assets Missing (Verify Game Files or Reinstall)", self.window_context.clone())))
        };

        let mut scrolling_text = ScrollingText::new(
            "Hello!".to_string(), 0.05,
            vec2(self.window_context.active_screen_size.x / 2.0, 50.0),
            vec2(0.5, 0.0),
            TextParams {
                font,
                font_size: 75,
                font_scale: 0.25,
                color: WHITE,
                ..Default::default()
            }
        );

        let mut scrolling_text_line_2 = ScrollingText::new(
            "".to_string(), 0.05,
            vec2(self.window_context.active_screen_size.x / 2.0, 75.0),
            vec2(0.5, 0.0),
            TextParams {
                font,
                font_size: 75,
                font_scale: 0.25,
                color: WHITE,
                ..Default::default()
            }
        );

        let mut active_notes = vec![
            (12.0, 2.0, 0.0),
            (36.0, 1.0, 6.0)
        ];
        let mut holds = vec![
            (36.0, 1.0, 6.0)
        ];
        let mut song_attacks = vec![
            (24.0, 4.0, 1.0)
        ];

        let mut sound_manager =
            AudioManager::<CpalBackend>::new(AudioManagerSettings::default()).unwrap();
        let sound =
            StaticSoundData::from_file("assets/songs/music_files/Goldn.wav", StaticSoundSettings::default())
                .unwrap();

        let mut music = sound_manager.play(sound).unwrap();

        let config =
            match serde_json::from_str::<Config>(&match load_string("assets/config.json").await {
                Ok(text) => text,
                Err(_) => return Some(Box::new(ErrorScene::new("Config File Missing", self.window_context.clone())))
            }) {
                Ok(config) => config,
                Err(_) => return Some(Box::new(ErrorScene::new("Config File Error", self.window_context.clone())))
            };

        music.set_volume(config.volume, Default::default()).unwrap();

        // Background
        let background_texture =
            match quick_load_texture("assets/images/backgrounds/Space Background (3).png").await {
                Ok(texture) => texture,
                Err(_) => return Some(Box::new(ErrorScene::new("Assets Missing (Verify Game Files or Reinstall)", self.window_context.clone())))
            };

        let ship = match quick_load_texture("assets/images/ship.png").await  {
            Ok(texture) => texture,
            Err(_) => return Some(Box::new(ErrorScene::new("Assets Missing (Verify Game Files or Reinstall)", self.window_context.clone())))
        };
        let mut ship_position = SHIP_FAR_RIGHT / 2.0;
        let mut ship_height = 200.0;
        let mut wanted_ship_height = RIGHT_ARROW_POS;

        let mut ship_invincibility = 0.25;
        let mut ship_alpha = 1.0;
        let mut ship_alpha_growing = false;

        // Input Notes
        let input_note_up = match quick_load_texture("assets/images/arrow_up.png").await {
            Ok(texture) => texture,
            Err(_) => return Some(Box::new(ErrorScene::new("Assets Missing (Verify Game Files or Reinstall)", self.window_context.clone())))
        };
        let input_note_down = match quick_load_texture("assets/images/arrow_down.png").await {
            Ok(texture) => texture,
            Err(_) => return Some(Box::new(ErrorScene::new("Assets Missing (Verify Game Files or Reinstall)", self.window_context.clone())))
        };
        let input_note_left = match quick_load_texture("assets/images/arrow_left.png").await {
            Ok(texture) => texture,
            Err(_) => return Some(Box::new(ErrorScene::new("Assets Missing (Verify Game Files or Reinstall)", self.window_context.clone())))
        };
        let input_note_right = match quick_load_texture("assets/images/arrow_right.png").await {
            Ok(texture) => texture,
            Err(_) => return Some(Box::new(ErrorScene::new("Assets Missing (Verify Game Files or Reinstall)", self.window_context.clone())))
        };

        let mut up_scale = 1.0;
        let mut down_scale = 1.0;
        let mut left_scale = 1.0;
        let mut right_scale = 1.0;

        let hold_note = match quick_load_texture("assets/images/hold.png").await {
            Ok(texture) => texture,
            Err(_) => return Some(Box::new(ErrorScene::new("Assets Missing (Verify Game Files or Reinstall)", self.window_context.clone())))
        };
        let laser = match quick_load_texture("assets/images/laser.png").await {
            Ok(texture) => texture,
            Err(_) => return Some(Box::new(ErrorScene::new("Assets Missing (Verify Game Files or Reinstall)", self.window_context.clone())))
        };

        let left_control = u32_to_key_code(config.controls.left_arrow);
        let right_control = u32_to_key_code(config.controls.right_arrow);

        let ship_up_control = u32_to_key_code(config.controls.ship_up);
        let ship_down_control = u32_to_key_code(config.controls.ship_down);

        let mut song_progression = 0;

        let beats_per_second = 146.0 / 60.0;
        let pixels_per_beat = (NOTE_START_POS - ARROW_OFFSET) / BEATS_TO_NOTE_HIT;

        let mut red_increasing = false;
        let mut red_value = 1.0;
        let mut blue_increasing = false;
        let mut blue_value = 1.0;
        let mut green_increasing = false;
        let mut green_value = 1.0;

        let mut thickness_multi_growing: bool = true;
        let mut hold_thickness_multi: f32 = 1.0;

        loop {
            clear_background(BLACK);
            set_camera(&self.window_context.camera);
            clear_background(DARKGRAY);

            draw_texture(background_texture, 0.0, 0.0, Color::new(0.5, 0.5, 0.5, 1.0));

            let beat =
                beats_per_second * ((music.position() * 1_000_000.0).round() / 1_000_000.0) as f32;

            // Color Fixing
            red_value += get_frame_time()
                * 2.0
                * match red_increasing {
                false => -1.0,
                true => 1.0,
            };
            if red_value >= 1.0 {
                red_increasing = false;
            } else if red_value <= 0.2 {
                red_increasing = true;
            }

            green_value += get_frame_time()
                * 1.6
                * match green_increasing {
                false => -1.0,
                true => 1.0,
            };
            if green_value >= 1.0 {
                green_increasing = false;
            } else if green_value <= 0.2 {
                green_increasing = true;
            }

            blue_value += get_frame_time()
                * 1.2
                * match blue_increasing {
                false => -1.0,
                true => 1.0,
            };
            if blue_value >= 1.0 {
                blue_increasing = false;
            } else if blue_value <= 0.2 {
                blue_increasing = true;
            }

            // Scale the thickness
            if thickness_multi_growing {
                hold_thickness_multi += get_frame_time() * SCALE_HOLD_PER_SECOND;
                if hold_thickness_multi >= MAX_HOLD_THICKNESS_MULTI {
                    thickness_multi_growing = false
                }
            } else {
                hold_thickness_multi -= get_frame_time() * SCALE_HOLD_PER_SECOND;
                if hold_thickness_multi <= MIN_HOLD_THICKNESS_MULTI {
                    thickness_multi_growing = true
                }
            }

            match song_progression {
                0 => {
                    if beat >= 11.9 {
                        music.pause(Tween::default()).unwrap();
                        song_progression += 1;

                        scrolling_text.replace_text(format!("Press {:?} to hit the note!", left_control));
                    }
                }
                1 => {
                    if is_key_pressed(left_control) {
                        left_scale = ON_NOTE_PRESS_SCALE_FACTOR;
                        music.resume(Tween::default()).unwrap();
                        song_progression += 1;
                        scrolling_text.replace_text("Nice Job!".to_string());
                        active_notes.remove(0);
                    }
                }
                2 => {
                    if beat >= 23.5 {
                        music.pause(Tween::default()).unwrap();
                        song_progression += 1;
                        scrolling_text.replace_text("Look out! A laser! Move your ship".to_string());
                        scrolling_text_line_2.replace_text(
                            format!("out of the way with {:?} or {:?}", ship_up_control, ship_down_control)
                        );
                    }
                }
                3 => {
                    let mut done = false;
                    // Check for ship position changes
                    if is_key_pressed(ship_up_control) {
                        if wanted_ship_height == RIGHT_ARROW_POS {
                            wanted_ship_height = UP_ARROW_POS;
                        } else if wanted_ship_height == UP_ARROW_POS {
                            wanted_ship_height = LEFT_ARROW_POS;
                        } else if wanted_ship_height == DOWN_ARROW_POS {
                            wanted_ship_height = RIGHT_ARROW_POS;
                        }
                        done = true;
                    }
                    if is_key_pressed(ship_down_control) {
                        if wanted_ship_height == RIGHT_ARROW_POS {
                            wanted_ship_height = DOWN_ARROW_POS;
                        } else if wanted_ship_height == LEFT_ARROW_POS {
                            wanted_ship_height = UP_ARROW_POS;
                        } else if wanted_ship_height == UP_ARROW_POS {
                            wanted_ship_height = RIGHT_ARROW_POS;
                        }
                        done = true;
                    }

                    if done {
                        song_progression += 1;
                        music.resume(Tween::default()).unwrap();
                        scrolling_text.replace_text("Yikes! That was close!".to_string());
                        scrolling_text_line_2.replace_text("".to_string());
                    }
                }
                4 => {
                    if beat >= 35.9 {
                        song_progression += 1;
                        music.pause(Tween::default()).unwrap();
                        scrolling_text.replace_text(format!("Hold {:?}", right_control));
                        active_notes.remove(0);
                    }
                }
                5 => {
                    if is_key_down(right_control) {
                        song_progression += 1;
                        music.resume(Tween::default()).unwrap();
                    }
                }
                6 => {
                    if beat >= 42.0 {
                        song_progression += 1;
                        holds.remove(0);
                        scrolling_text.replace_text("Congratulations! You Passed".to_string());
                        scrolling_text_line_2.replace_text("Here's Easy Goldn! Good Luck!".to_string());
                    }
                    else if !is_key_down(right_control) {
                        song_progression -= 1;
                        music.pause(Tween::default()).unwrap();
                        scrolling_text.replace_text(format!("Keep Holding {:?}!", right_control))
                    }
                }
                7 => {
                    if beat >= 60.0 {
                        return Some(Box::new(NoteGameplayScene::new(
                            self.window_context.clone(),
                            "assets/songs/easy/goldn.json",
                        )))
                    }
                }
                _ => {
                    return Some(Box::new(MainMenuScene {
                        window_context: self.window_context.clone()
                    }))
                }
            }

            ship_height += (wanted_ship_height - ship_height) * 6.0 * get_frame_time();

            // Draw the active Notes
            for (note_beat, note_type, _hold_length) in &active_notes {
                if *note_beat - beat < 15.0 {
                    let note_draw_pos =
                        ((note_beat - beat) * pixels_per_beat) + (ARROW_OFFSET - NOTE_SIZE / 2.0);
                    draw_note(
                        *note_type,
                        note_draw_pos,
                        input_note_left,
                        input_note_right,
                        input_note_up,
                        input_note_down,
                    );
                }
            }

            draw_texture_ex(
                ship,
                ship_position,
                ship_height - SHIP_PIXEL_SIZE / 2.0,
                Color::new(1.0, 1.0, 1.0, 1.0),
                DrawTextureParams {
                    dest_size: Some(vec2(SHIP_PIXEL_SIZE, SHIP_PIXEL_SIZE)),
                    ..Default::default()
                },
            );

            let mut remove_attacks = vec![];
            for (attack_beat, last_length, note_type) in &song_attacks {
                let note_offset = match *note_type as i32 {
                    3 => UP_ARROW_POS,
                    4 => DOWN_ARROW_POS,
                    1 => RIGHT_ARROW_POS,
                    2 => LEFT_ARROW_POS,
                    _ => {
                        panic!("Error! Note type: '{note_type}' unknown")
                    }
                };

                if *attack_beat + *last_length <= beat {
                    remove_attacks.push((
                        *attack_beat,
                        *last_length,
                        *note_type,
                    ));
                    continue;
                }

                if beat >= *attack_beat - 5.0 && beat <= *attack_beat {
                    let difference = 5.0 - (*attack_beat - beat);

                    draw_texture_ex(
                        laser,
                        0.0,
                        note_offset - (40.0 * hold_thickness_multi) / 2.0,
                        Color::new(red_value, green_value, blue_value, 1.0),
                        DrawTextureParams {
                            dest_size: Some(vec2(
                                difference * difference * difference * 2.0,
                                40.0 * hold_thickness_multi,
                            )),
                            ..Default::default()
                        },
                    );
                }

                if *attack_beat >= beat {
                    continue;
                }

                draw_texture_ex(
                    laser,
                    0.0,
                    note_offset - (40.0 * hold_thickness_multi) / 2.0,
                    Color::new(red_value, green_value, blue_value, 1.0),
                    DrawTextureParams {
                        dest_size: Some(vec2(1000.0, 40.0 * hold_thickness_multi)),
                        ..Default::default()
                    },
                );
            }

            for remove_attack in &remove_attacks {
                song_attacks.retain(|x| x != remove_attack);
            }

            for (note_beat, note_type, hold_length) in &holds {
                if hold_length.clone() == 0.0 {
                    continue;
                }

                let note_draw_pos =
                    ((note_beat - beat) * pixels_per_beat) + (ARROW_OFFSET - NOTE_SIZE / 2.0);
                let mut hold_width = hold_length * pixels_per_beat;
                let hold_draw_pos = note_draw_pos + hold_width;

                if hold_draw_pos - hold_width < 15.0 {
                    hold_width = hold_draw_pos - 13.0;
                }

                let thick = note_beat.clone() <= beat;

                draw_hold(
                    *note_type,
                    hold_draw_pos,
                    hold_width,
                    hold_note,
                    match thick {
                        true => hold_thickness_multi,
                        false => 1.0
                    }
                );
            }

            // Scale Back Down
            if left_scale > 1.0 {
                left_scale -= get_frame_time() * SCALE_PER_SECOND_DECREASE;
                if left_scale < 1.0 {
                    left_scale = 1.0;
                }
            }
            if up_scale > 1.0 {
                up_scale -= get_frame_time() * SCALE_PER_SECOND_DECREASE;
                if up_scale < 1.0 {
                    up_scale = 1.0;
                }
            }
            if right_scale > 1.0 {
                right_scale -= get_frame_time() * SCALE_PER_SECOND_DECREASE;
                if right_scale < 1.0 {
                    right_scale = 1.0;
                }
            }
            if down_scale > 1.0 {
                down_scale -= get_frame_time() * SCALE_PER_SECOND_DECREASE;
                if down_scale < 1.0 {
                    down_scale = 1.0;
                }
            }

            // Draw the Input Notes
            draw_texture_ex(
                input_note_left,
                ARROW_OFFSET - (NOTE_SIZE * left_scale) / 2.0,
                LEFT_ARROW_POS - (NOTE_SIZE * left_scale) / 2.0,
                GREEN,
                DrawTextureParams {
                    dest_size: Some(vec2(NOTE_SIZE * left_scale, NOTE_SIZE * left_scale)),
                    ..Default::default()
                },
            );
            draw_texture_ex(
                input_note_up,
                ARROW_OFFSET - (NOTE_SIZE * up_scale) / 2.0,
                UP_ARROW_POS - (NOTE_SIZE * up_scale) / 2.0,
                SKYBLUE,
                DrawTextureParams {
                    dest_size: Some(vec2(NOTE_SIZE * up_scale, NOTE_SIZE * up_scale)),
                    ..Default::default()
                },
            );
            draw_texture_ex(
                input_note_right,
                ARROW_OFFSET - (NOTE_SIZE * right_scale) / 2.0,
                RIGHT_ARROW_POS - (NOTE_SIZE * right_scale) / 2.0,
                ORANGE,
                DrawTextureParams {
                    dest_size: Some(vec2(NOTE_SIZE * right_scale, NOTE_SIZE * right_scale)),
                    ..Default::default()
                },
            );
            draw_texture_ex(
                input_note_down,
                ARROW_OFFSET - (NOTE_SIZE * down_scale) / 2.0,
                DOWN_ARROW_POS - (NOTE_SIZE * down_scale) / 2.0,
                RED,
                DrawTextureParams {
                    dest_size: Some(vec2(NOTE_SIZE * down_scale, NOTE_SIZE * down_scale)),
                    ..Default::default()
                },
            );

            scrolling_text.update();
            scrolling_text_line_2.update();

            draw_window(&mut self.window_context);
            next_frame().await;
        }
    }
}
