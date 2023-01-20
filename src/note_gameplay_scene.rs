use async_trait::async_trait;
use kira::manager::{AudioManager, AudioManagerSettings};
use kira::manager::backend::cpal::CpalBackend;
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use macroquad::prelude::*;
use macroquad_aspect::prelude::*;

use crate::note_gameplay_scene::constants::*;
use crate::note_gameplay_scene::score_texts::{ScoreQuality, ScoreText, ScoreType};
use crate::note_gameplay_scene::score_texts::ScoreType::Score;
use crate::note_gameplay_scene::song::Song;

use thousands::Separable;

use crate::Scene;
use crate::utils::*;

mod constants;
mod song;
mod score_texts;

pub struct NoteGameplayScene {
    pub window_context: WindowContext,
    pub song_path: String,
}

impl NoteGameplayScene {
    pub fn new(window_context: WindowContext, song_path: &str) -> Self {
        Self {
            window_context,
            song_path: song_path.to_string()
        }
    }
}

#[async_trait]
impl Scene for NoteGameplayScene {
    async fn run(&mut self) -> Option<Box<dyn Scene>> {

        // Fonts
        let font = load_ttf_font("assets/fonts/pixel.ttf").await.unwrap();

        // Score Texts
        let mut score_texts: Vec<ScoreText> = vec![];

        // Health
        let mut health = MAX_HEALTH;

        // Score
        let mut score = 0;
        let mut combo_multiplier = 1.0;

        // Load the Song
        let song_json = load_string(self.song_path.as_str()).await.unwrap();
        let song = serde_json::from_str::<Song>(song_json.as_str()).unwrap();
        let mut active_notes = song.notes.clone();

        let mut drawn_holds = song.notes.clone();
        drawn_holds.retain(|x| x.2 != 0.0);
        let mut active_holds: Vec<(f32, f32, f32)> = vec![];
        let mut hold_thickness_multi: f32 = 1.0;
        let mut thickness_multi_growing: bool = true;

        let beats_per_second = song.bpm / 60.0;
        let pixels_per_beat = (NOTE_START_POS - ARROW_OFFSET) / BEATS_TO_NOTE_HIT;

        let mut sound_manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default()).unwrap();
        let sound = StaticSoundData::from_file(
            song.song_filepath,
            StaticSoundSettings::default(),
        ).unwrap();

        let music = sound_manager.play(sound).unwrap();

        // Background
        let background_texture = quick_load_texture("assets/images/backgrounds/Space Background (3).png").await;

        let ship = quick_load_texture("assets/images/ship.png").await;
        let mut ship_position = SHIP_FAR_RIGHT / 2.0;

        // Input Notes
        let input_note_up = quick_load_texture("assets/images/icons/Up-Arrow.png").await;
        let input_note_down = quick_load_texture("assets/images/icons/Down-Arrow.png").await;
        let input_note_left = quick_load_texture("assets/images/icons/Left-Arrow.png").await;
        let input_note_right = quick_load_texture("assets/images/icons/Right-Arrow.png").await;

        let mut up_scale = 1.0;
        let mut down_scale = 1.0;
        let mut left_scale = 1.0;
        let mut right_scale = 1.0;

        let hold_note = quick_load_texture("assets/images/hold.png").await;

        loop {
            clear_background(BLACK);
            set_camera(&self.window_context.camera);
            clear_background(DARKGRAY);

            draw_texture(background_texture, 0.0, 0.0, Color::new(0.5, 0.5, 0.5, 1.0));

            let beat = beats_per_second * ((music.position() * 1_000_000.0).round() / 1_000_000.0) as f32;

            // Scale the thickness
            if thickness_multi_growing {
                hold_thickness_multi += get_frame_time() * SCALE_HOLD_PER_SECOND;
                if hold_thickness_multi >= MAX_HOLD_THICKNESS_MULTI { thickness_multi_growing = false }
            } else {
                hold_thickness_multi -= get_frame_time() * SCALE_HOLD_PER_SECOND;
                if hold_thickness_multi <= MIN_HOLD_THICKNESS_MULTI { thickness_multi_growing = true }
            }

            // Check For Hit Notes
            let mut correct_up = false;
            let mut correct_down = false;
            let mut correct_left = false;
            let mut correct_right = false;

            let mut hit_notes = vec![];

            for (note_beat, note_type, hold_length) in &active_notes {

                let note_offset = match note_type.clone() as i32 {
                    3 => UP_ARROW_POS,
                    4 => DOWN_ARROW_POS,
                    1 => RIGHT_ARROW_POS,
                    2 => LEFT_ARROW_POS,
                    _ => { panic!("Error! Note type: '{note_type}' unknown") }
                };

                if note_beat.clone() < beat - 1.0 {
                    hit_notes.push((note_beat.clone(), note_type.clone(), hold_length.clone()));
                    score_texts.push(ScoreText {timer: TEXT_LAST_TIME, score_type: ScoreType::Miss, y_offset: note_offset});
                    health -= HEALTH_LOSS_MISS;
                    combo_multiplier = 1.0;

                    continue;
                }

                if note_beat.clone() < beat - NOTE_CORRECT_RANGE || note_beat.clone() > beat + NOTE_CORRECT_RANGE { continue }

                let diff = note_beat - beat;
                if (is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W)) && !correct_up && note_type.floor() == 3.0 {
                    hit_notes.push((note_beat.clone(), note_type.clone(), hold_length.clone()));
                    correct_up = true;

                    if hold_length.clone() != 0.0 {
                        active_holds.push((note_beat.clone(), note_type.clone(), hold_length.clone()));
                    } else {
                        health += CORRECT_HEALTH_GAIN;

                        if diff <= PERFECT_HIT_RANGE {
                            score += (PERFECT_HIT_SCORE as f32 * combo_multiplier).round() as i32;
                            combo_multiplier *= 1.05;
                            score_texts.push(ScoreText {
                                timer: TEXT_LAST_TIME,
                                score_type: Score(ScoreQuality::Perfect),
                                y_offset: note_offset
                            })
                        } else if diff <= GOOD_HIT_RANGE {
                            score += (GOOD_HIT_SCORE as f32 * combo_multiplier).round() as i32;
                            combo_multiplier *= 1.025;
                            score_texts.push(ScoreText {
                                timer: TEXT_LAST_TIME,
                                score_type: Score(ScoreQuality::Good),
                                y_offset: note_offset
                            })
                        } else {
                            score += (OK_HIT_SCORE as f32 * combo_multiplier).round() as i32;
                            score_texts.push(ScoreText {
                                timer: TEXT_LAST_TIME,
                                score_type: Score(ScoreQuality::Ok),
                                y_offset: note_offset
                            })
                        }
                    }
                }
                if (is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S)) && !correct_down && note_type.floor() == 4.0 {
                    hit_notes.push((note_beat.clone(), note_type.clone(), hold_length.clone()));
                    correct_down = true;

                    if hold_length.clone() != 0.0 {
                        active_holds.push((note_beat.clone(), note_type.clone(), hold_length.clone()));
                    } else {
                        health += CORRECT_HEALTH_GAIN;

                        if diff <= PERFECT_HIT_RANGE {
                            score += (PERFECT_HIT_SCORE as f32 * combo_multiplier).round() as i32;
                            combo_multiplier *= 1.05;
                            score_texts.push(ScoreText {
                                timer: TEXT_LAST_TIME,
                                score_type: Score(ScoreQuality::Perfect),
                                y_offset: note_offset
                            })
                        } else if diff <= GOOD_HIT_RANGE {
                            score += (GOOD_HIT_SCORE as f32 * combo_multiplier).round() as i32;
                            combo_multiplier *= 1.025;
                            score_texts.push(ScoreText {
                                timer: TEXT_LAST_TIME,
                                score_type: Score(ScoreQuality::Good),
                                y_offset: note_offset
                            })
                        } else {
                            score += (OK_HIT_SCORE as f32 * combo_multiplier).round() as i32;
                            score_texts.push(ScoreText {
                                timer: TEXT_LAST_TIME,
                                score_type: Score(ScoreQuality::Ok),
                                y_offset: note_offset
                            })
                        }
                    }
                }
                if (is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D)) && !correct_right && note_type.floor() == 1.0 {
                    hit_notes.push((note_beat.clone(), note_type.clone(), hold_length.clone()));
                    correct_right = true;

                    if hold_length.clone() != 0.0 {
                        active_holds.push((note_beat.clone(), note_type.clone(), hold_length.clone()));
                    } else {
                        health += CORRECT_HEALTH_GAIN;

                        if diff <= PERFECT_HIT_RANGE {
                            score += (PERFECT_HIT_SCORE as f32 * combo_multiplier).round() as i32;
                            combo_multiplier *= 1.05;
                            score_texts.push(ScoreText {
                                timer: TEXT_LAST_TIME,
                                score_type: Score(ScoreQuality::Perfect),
                                y_offset: note_offset
                            })
                        } else if diff <= GOOD_HIT_RANGE {
                            score += (GOOD_HIT_SCORE as f32 * combo_multiplier).round() as i32;
                            combo_multiplier *= 1.025;
                            score_texts.push(ScoreText {
                                timer: TEXT_LAST_TIME,
                                score_type: Score(ScoreQuality::Good),
                                y_offset: note_offset
                            })
                        } else {
                            score += (OK_HIT_SCORE as f32 * combo_multiplier).round() as i32;
                            score_texts.push(ScoreText {
                                timer: TEXT_LAST_TIME,
                                score_type: Score(ScoreQuality::Ok),
                                y_offset: note_offset
                            })
                        }
                    }
                }
                if (is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A)) && !correct_left && note_type.floor() == 2.0 {
                    hit_notes.push((note_beat.clone(), note_type.clone(), hold_length.clone()));
                    correct_left = true;

                    if hold_length.clone() != 0.0 {
                        active_holds.push((note_beat.clone(), note_type.clone(), hold_length.clone()));
                    } else {
                        health += CORRECT_HEALTH_GAIN;

                        if diff <= PERFECT_HIT_RANGE {
                            score += (PERFECT_HIT_SCORE as f32 * combo_multiplier).round() as i32;
                            combo_multiplier *= 1.05;
                            score_texts.push(ScoreText {
                                timer: TEXT_LAST_TIME,
                                score_type: Score(ScoreQuality::Perfect),
                                y_offset: note_offset
                            })
                        } else if diff <= GOOD_HIT_RANGE {
                            score += (GOOD_HIT_SCORE as f32 * combo_multiplier).round() as i32;
                            combo_multiplier *= 1.025;
                            score_texts.push(ScoreText {
                                timer: TEXT_LAST_TIME,
                                score_type: Score(ScoreQuality::Good),
                                y_offset: note_offset
                            })
                        } else {
                            score += (OK_HIT_SCORE as f32 * combo_multiplier).round() as i32;
                            score_texts.push(ScoreText {
                                timer: TEXT_LAST_TIME,
                                score_type: Score(ScoreQuality::Ok),
                                y_offset: note_offset
                            })
                        }
                    }
                }
            }

            for hit_note in &hit_notes {
                active_notes.retain(|x| x.0 != hit_note.0 || x.1 != hit_note.1 || x.2 != hit_note.2);
            }

            // Check for missed notes
            if (is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W)) && !correct_up {
                health -= HEALTH_LOSS_INCORRECT;
                score_texts.push(ScoreText {timer: TEXT_LAST_TIME, score_type: ScoreType::Incorrect, y_offset: UP_ARROW_POS});
                combo_multiplier = 1.0;
            }
            if (is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S)) && !correct_down {
                health -= HEALTH_LOSS_INCORRECT;
                score_texts.push(ScoreText {timer: TEXT_LAST_TIME, score_type: ScoreType::Incorrect, y_offset: DOWN_ARROW_POS});
                combo_multiplier = 1.0;
            }
            if (is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A)) && !correct_left {
                health -= HEALTH_LOSS_INCORRECT;
                score_texts.push(ScoreText {timer: TEXT_LAST_TIME, score_type: ScoreType::Incorrect, y_offset: LEFT_ARROW_POS});
                combo_multiplier = 1.0;
            }
            if (is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D)) && !correct_right {
                health -= HEALTH_LOSS_INCORRECT;
                score_texts.push(ScoreText {timer: TEXT_LAST_TIME, score_type: ScoreType::Incorrect, y_offset: RIGHT_ARROW_POS});
                combo_multiplier = 1.0;
            }

            // Draw the SHIP (AKA: Health Bar)!
            let health_percentage = health as f32 / MAX_HEALTH as f32;
            let wanted_ship_position = (SHIP_FAR_RIGHT * health_percentage) - 150.0;
            ship_position += (wanted_ship_position - ship_position) * get_frame_time();

            draw_texture_ex(ship, ship_position, 200.0 - SHIP_PIXEL_SIZE / 2.0, WHITE, DrawTextureParams {
                dest_size: Some(vec2(SHIP_PIXEL_SIZE, SHIP_PIXEL_SIZE)),
                ..Default::default()
            });

            // Draw the active Notes
            for (note_beat, note_type, _hold_length) in &active_notes {
                let note_draw_pos = ((note_beat - beat) * pixels_per_beat) + (ARROW_OFFSET - NOTE_SIZE / 2.0);
                draw_note(note_type.clone(), note_draw_pos, input_note_left, input_note_right, input_note_up, input_note_down);
            }

            // Check For Hold notes failed or completed
            let mut remove_holds = vec![];
            for (note_beat, note_type, hold_length) in &active_holds {

                let note_offset = match note_type.clone() as i32 {
                    3 => UP_ARROW_POS,
                    4 => DOWN_ARROW_POS,
                    1 => RIGHT_ARROW_POS,
                    2 => LEFT_ARROW_POS,
                    _ => { panic!("Error! Note type: '{note_type}' unknown") }
                };

                let mut stay_active = false;
                let percent_done = ((beat - note_beat) / hold_length).clamp(0.0, 1.0);
                if (is_key_down(KeyCode::Up) || is_key_down(KeyCode::W)) && note_type.floor() == 3.0 {
                    stay_active = true
                }
                if (is_key_down(KeyCode::Down) || is_key_down(KeyCode::S)) && note_type.floor() == 4.0 {
                    stay_active = true
                }
                if (is_key_down(KeyCode::Right) || is_key_down(KeyCode::D)) && note_type.floor() == 1.0 {
                    stay_active = true
                }
                if (is_key_down(KeyCode::Left) || is_key_down(KeyCode::A)) && note_type.floor() == 2.0 {
                    stay_active = true
                }

                if stay_active && percent_done >= 1.0 {
                    score += ((HOLD_SCORE_PER_BEAT as f32 / hold_length) * combo_multiplier).round() as i32;
                    remove_holds.push((note_beat.clone(), note_type.clone(), hold_length.clone()));
                    combo_multiplier *= 1.08;
                    score_texts.push(ScoreText {
                        timer: TEXT_LAST_TIME,
                        score_type: Score(ScoreQuality::Perfect),
                        y_offset: note_offset
                    })
                }

                if !stay_active && percent_done <= 1.0 {
                    score += (((HOLD_SCORE_PER_BEAT as f32 / hold_length) * percent_done) * combo_multiplier).round() as i32;
                    remove_holds.push((note_beat.clone(), note_type.clone(), hold_length.clone()));
                    combo_multiplier *= 0.98;
                    score_texts.push(ScoreText {
                        timer: TEXT_LAST_TIME,
                        score_type: Score(ScoreQuality::Ok),
                        y_offset: note_offset
                    });

                    drawn_holds.push((beat, note_type.clone(), (1.0 - percent_done) * hold_length.clone()));
                }
            }

            for remove_hold in &remove_holds {
                active_holds.retain(|x| x != remove_hold);
                drawn_holds.retain(|x| x != remove_hold);
            }

            combo_multiplier = combo_multiplier.clamp(1.0, MAX_COMBO_MULTI);

            // Draw the active Holds
            let mut remove_holds = vec![];
            for (note_beat, note_type, hold_length) in &drawn_holds {
                let note_draw_pos = ((note_beat - beat) * pixels_per_beat) + (ARROW_OFFSET - NOTE_SIZE / 2.0);
                let mut hold_width = hold_length * pixels_per_beat;
                let hold_draw_pos = note_draw_pos + hold_width;

                let is_active = active_holds.contains(&(note_beat.clone(), note_type.clone(), hold_length.clone()));

                if hold_draw_pos <= 15.0 && is_active {
                    remove_holds.push((note_beat.clone(), note_type.clone(), hold_length.clone()))
                } else if hold_draw_pos <= -15.0 && !is_active {
                    remove_holds.push((note_beat.clone(), note_type.clone(), hold_length.clone()))
                }

                if is_active && hold_draw_pos - hold_width < 15.0 {
                    hold_width = hold_draw_pos - 13.0;
                }

                draw_hold(note_type.clone(), hold_draw_pos, hold_width, hold_note, match is_active {
                    true => hold_thickness_multi,
                    false => 1.0
                });
            }

            for remove_hold in &remove_holds {
                drawn_holds.retain(|x| x != remove_hold);
            }

            // Check Scale Up
            if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                left_scale = ON_NOTE_PRESS_SCALE_FACTOR;
            }
            if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                up_scale = ON_NOTE_PRESS_SCALE_FACTOR;
            }
            if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                right_scale = ON_NOTE_PRESS_SCALE_FACTOR;
            }
            if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                down_scale = ON_NOTE_PRESS_SCALE_FACTOR;
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
            draw_texture_ex(input_note_left, ARROW_OFFSET - (NOTE_SIZE * left_scale) / 2.0, LEFT_ARROW_POS - (NOTE_SIZE * left_scale) / 2.0, GREEN, DrawTextureParams {
                dest_size: Some(vec2(NOTE_SIZE * left_scale, NOTE_SIZE * left_scale)),
                ..Default::default()
            });
            draw_texture_ex(input_note_up, ARROW_OFFSET - (NOTE_SIZE * up_scale) / 2.0, UP_ARROW_POS - (NOTE_SIZE * up_scale) / 2.0, SKYBLUE, DrawTextureParams {
                dest_size: Some(vec2(NOTE_SIZE * up_scale, NOTE_SIZE * up_scale)),
                ..Default::default()
            });
            draw_texture_ex(input_note_right, ARROW_OFFSET - (NOTE_SIZE * right_scale) / 2.0, RIGHT_ARROW_POS - (NOTE_SIZE * right_scale) / 2.0, ORANGE, DrawTextureParams {
                dest_size: Some(vec2(NOTE_SIZE * right_scale, NOTE_SIZE * right_scale)),
                ..Default::default()
            });
            draw_texture_ex(input_note_down, ARROW_OFFSET - (NOTE_SIZE * down_scale) / 2.0, DOWN_ARROW_POS - (NOTE_SIZE * down_scale) / 2.0, RED, DrawTextureParams {
                dest_size: Some(vec2(NOTE_SIZE * down_scale, NOTE_SIZE * down_scale)),
                ..Default::default()
            });

            let mut remove_texts = vec![];
            for score_text in &mut score_texts {
                if score_text.update_and_draw(font) {
                    remove_texts.push(score_text.clone());
                }
            }

            for remove_text in &remove_texts {
                score_texts.retain(|x| x != remove_text);
            }

            draw_text_ex(format!("SCORE: {}", score.separate_with_commas()).as_str(), 5.0, 15.0, TextParams {
                font,
                font_size: 50,
                font_scale: 0.25,
                color: WHITE,
                ..Default::default()
            });

            draw_text_ex(format!("{}", song.credits).as_str(), 535.0, 385.0, TextParams {
                font,
                font_size: 40,
                font_scale: 0.25,
                color: WHITE,
                ..Default::default()
            });


            // Clamp the health value to a max
            health = health.clamp(0, MAX_HEALTH);

            draw_window(&mut self.window_context);

            // Close Conditions
            if is_key_pressed(KeyCode::Escape) {
                return None
            }

            if health <= 0 {
                return None
            }

            if music.position() >= song.song_length as f64 {
                return None
            }

            next_frame().await;
        }
    }
}

pub fn draw_note(direction: f32, location: f32, left_tex: Texture2D, right_tex: Texture2D, up_tex: Texture2D, down_tex: Texture2D) {
    let direction = direction.round() as i32;
    match direction {
        1 => { // Right
            draw_texture_ex(right_tex, location, RIGHT_ARROW_POS - NOTE_SIZE / 2.0, ORANGE, DrawTextureParams {
                dest_size: Some(vec2(NOTE_SIZE, NOTE_SIZE)),
                ..Default::default()
            });
        },
        2 => { // Left
            draw_texture_ex(left_tex, location, LEFT_ARROW_POS - NOTE_SIZE / 2.0, GREEN, DrawTextureParams {
                dest_size: Some(vec2(NOTE_SIZE, NOTE_SIZE)),
                ..Default::default()
            });
        },
        3 => { // Up
            draw_texture_ex(up_tex, location, UP_ARROW_POS - NOTE_SIZE / 2.0, SKYBLUE, DrawTextureParams {
                dest_size: Some(vec2(NOTE_SIZE, NOTE_SIZE)),
                ..Default::default()
            });
        },
        4 => { // Down
            draw_texture_ex(down_tex, location, DOWN_ARROW_POS - NOTE_SIZE / 2.0, RED, DrawTextureParams {
                dest_size: Some(vec2(NOTE_SIZE, NOTE_SIZE)),
                ..Default::default()
            });
        },
        _ => { todo!("Add direction drawing for note type.") }
    }
}

pub fn draw_hold(direction: f32, location: f32, width: f32, texture: Texture2D, thickness_multi: f32) {
    let direction = direction.round() as i32;
    let note_height = (NOTE_SIZE / 1.5) * thickness_multi;
    let location = location + 15.0;
    match direction {
        1 => { // Right
            draw_texture_ex(texture, location, RIGHT_ARROW_POS - note_height / 2.0, ORANGE, DrawTextureParams {
                dest_size: Some(vec2(-width, note_height)),
                ..Default::default()
            });
        },
        2 => { // Left
            draw_texture_ex(texture, location, LEFT_ARROW_POS - note_height / 2.0, GREEN, DrawTextureParams {
                dest_size: Some(vec2(-width, note_height)),
                ..Default::default()
            });
        },
        3 => { // Up
            draw_texture_ex(texture, location, UP_ARROW_POS - note_height / 2.0, SKYBLUE, DrawTextureParams {
                dest_size: Some(vec2(-width, note_height)),
                ..Default::default()
            });
        },
        4 => { // Down
            draw_texture_ex(texture, location, DOWN_ARROW_POS - note_height / 2.0, RED, DrawTextureParams {
                dest_size: Some(vec2(-width, note_height)),
                ..Default::default()
            });
        },
        _ => { todo!("Add direction drawing for note type.") }
    }
}