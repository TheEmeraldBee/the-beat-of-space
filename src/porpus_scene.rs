use async_trait::async_trait;
use kira::manager::backend::cpal::CpalBackend;
use kira::manager::{AudioManager, AudioManagerSettings};
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use macroquad::prelude::*;
use macroquad_aspect::prelude::*;

use crate::note_gameplay_scene::constants::*;
use crate::note_gameplay_scene::score_texts::ScoreType::Score;
use crate::note_gameplay_scene::score_texts::{ScoreQuality, ScoreText, ScoreType};
use crate::note_gameplay_scene::song::Song;

use crate::game_end_scene::GameEndScene;
use crate::main_menu_scene::MainMenuScene;
use crate::note_gameplay_scene::{draw_hold, draw_note, ReturnTo};
use thousands::Separable;
use crate::beatmap_editor_scene::BeatmapEditorScene;

use crate::ui::draw_text_justified;
use crate::utils::*;
use crate::Scene;

pub struct PorpusScene {
    pub window_context: WindowContext,
    pub song_path: String,
    pub return_to: ReturnTo
}

impl PorpusScene {
    pub fn new(window_context: WindowContext, song_path: &str, return_to: ReturnTo) -> Self {
        Self {
            window_context,
            song_path: song_path.to_string(),
            return_to
        }
    }
}

#[async_trait]
impl Scene for PorpusScene {
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

        let mut perfect_notes = 0;

        // Color Changing
        let mut red_increasing = false;
        let mut red_value = 1.0;
        let mut blue_increasing = false;
        let mut blue_value = 1.0;
        let mut green_increasing = false;
        let mut green_value = 1.0;

        // Load the Song
        let song_json = load_string(self.song_path.as_str()).await.unwrap();
        let song = serde_json::from_str::<Song>(song_json.as_str()).unwrap();
        let mut active_notes = song.notes.clone();
        let mut song_attacks = song.attacks.clone();

        let mut drawn_holds = song.notes.clone();
        drawn_holds.retain(|x| x.2 != 0.0);
        let mut active_holds: Vec<(f32, f32, f32)> = vec![];
        let mut hold_thickness_multi: f32 = 1.0;
        let mut thickness_multi_growing: bool = true;

        let beats_per_second = song.bpm / 60.0;
        let pixels_per_beat = (NOTE_START_POS - ARROW_OFFSET) / BEATS_TO_NOTE_HIT;

        let mut sound_manager =
            AudioManager::<CpalBackend>::new(AudioManagerSettings::default()).unwrap();
        let sound =
            StaticSoundData::from_file(song.song_filepath.clone(), StaticSoundSettings::default())
                .unwrap();

        let mut music = sound_manager.play(sound).unwrap();

        let config =
            serde_json::from_str::<Config>(&load_string("assets/config.json").await.unwrap())
                .unwrap();

        music.set_volume(config.volume, Default::default()).unwrap();

        // Background
        let background_texture =
            quick_load_texture("assets/images/backgrounds/Space Background (3).png").await;

        let ship = quick_load_texture("assets/images/ship.png").await;
        let mut ship_position = SHIP_FAR_RIGHT / 2.0;
        let mut ship_height = 200.0;
        let mut wanted_ship_height = RIGHT_ARROW_POS;

        let mut ship_invincibility = 0.25;
        let mut ship_alpha = 1.0;
        let mut ship_alpha_growing = false;

        // Input Notes
        let input_note_up = quick_load_texture("assets/images/arrow_up.png").await;
        let input_note_down = quick_load_texture("assets/images/arrow_down.png").await;
        let input_note_left = quick_load_texture("assets/images/arrow_left.png").await;
        let input_note_right = quick_load_texture("assets/images/arrow_right.png").await;

        let mut up_scale = 1.0;
        let mut down_scale = 1.0;
        let mut left_scale = 1.0;
        let mut right_scale = 1.0;

        let hold_note = quick_load_texture("assets/images/hold.png").await;
        let laser = quick_load_texture("assets/images/laser.png").await;

        let mut game_over_timer = Timer::new(3.0, 0);

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

            let mut hit_notes = vec![];

            for (note_beat, note_type, hold_length) in &active_notes {
                let note_offset = match note_type.clone() as i32 {
                    3 => UP_ARROW_POS,
                    4 => DOWN_ARROW_POS,
                    1 => RIGHT_ARROW_POS,
                    2 => LEFT_ARROW_POS,
                    _ => {
                        panic!("Error! Note type: '{note_type}' unknown")
                    }
                };

                if note_beat.clone() < beat - NOTE_CORRECT_RANGE
                    || note_beat.clone() > beat + NOTE_CORRECT_RANGE
                {
                    continue;
                }

                let diff = note_beat - beat;
                if diff < PERFECT_HIT_RANGE / 1.5 {
                    hit_notes.push((note_beat.clone(), note_type.clone(), hold_length.clone()));

                    if hold_length.clone() != 0.0 {
                        active_holds.push((
                            note_beat.clone(),
                            note_type.clone(),
                            hold_length.clone(),
                        ));
                    } else {
                        health += CORRECT_HEALTH_GAIN;
                        score += (PERFECT_HIT_SCORE as f32 * combo_multiplier).round() as i32;
                        combo_multiplier *= 1.05;
                        score_texts.push(ScoreText {
                            timer: TEXT_LAST_TIME,
                            score_type: Score(ScoreQuality::Perfect),
                            y_offset: note_offset,
                        });
                        perfect_notes += 1;
                    }

                    match note_type.clone() as i32 {
                        3 => up_scale = ON_NOTE_PRESS_SCALE_FACTOR,
                        4 => down_scale = ON_NOTE_PRESS_SCALE_FACTOR,
                        2 => left_scale = ON_NOTE_PRESS_SCALE_FACTOR,
                        1 => right_scale = ON_NOTE_PRESS_SCALE_FACTOR,
                        _ => {}
                    }
                }
            }

            for hit_note in &hit_notes {
                active_notes
                    .retain(|x| x.0 != hit_note.0 || x.1 != hit_note.1 || x.2 != hit_note.2);
            }

            ship_height += (wanted_ship_height - ship_height) * 6.0 * get_frame_time();

            // Draw the SHIP (AKA: Health Bar)!
            let health_percentage = health as f32 / MAX_HEALTH as f32;
            let wanted_ship_position = (SHIP_FAR_RIGHT * health_percentage) - 150.0;
            ship_position += (wanted_ship_position - ship_position) * get_frame_time();

            let alpha = match ship_invincibility > 0.0 {
                true => ship_alpha,
                false => 1.0,
            };

            draw_texture_ex(
                ship,
                ship_position,
                ship_height - SHIP_PIXEL_SIZE / 2.0,
                Color::new(1.0, 1.0, 1.0, alpha),
                DrawTextureParams {
                    dest_size: Some(vec2(SHIP_PIXEL_SIZE, SHIP_PIXEL_SIZE)),
                    ..Default::default()
                },
            );

            // Check For Hold notes failed or completed
            let mut remove_holds = vec![];
            for (note_beat, note_type, hold_length) in &active_holds {
                let note_offset = match note_type.clone() as i32 {
                    3 => UP_ARROW_POS,
                    4 => DOWN_ARROW_POS,
                    1 => RIGHT_ARROW_POS,
                    2 => LEFT_ARROW_POS,
                    _ => {
                        panic!("Error! Note type: '{note_type}' unknown")
                    }
                };
                let percent_done = ((beat - note_beat) / hold_length).clamp(0.0, 1.0);

                if percent_done >= 1.0 {
                    score += ((HOLD_SCORE_PER_BEAT as f32 / hold_length) * combo_multiplier).round()
                        as i32;
                    remove_holds.push((note_beat.clone(), note_type.clone(), hold_length.clone()));
                    combo_multiplier *= 1.08;
                    score_texts.push(ScoreText {
                        timer: TEXT_LAST_TIME,
                        score_type: Score(ScoreQuality::Perfect),
                        y_offset: note_offset,
                    })
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
                let note_draw_pos =
                    ((note_beat - beat) * pixels_per_beat) + (ARROW_OFFSET - NOTE_SIZE / 2.0);
                let mut hold_width = hold_length * pixels_per_beat;
                let hold_draw_pos = note_draw_pos + hold_width;

                let is_active = active_holds.contains(&(
                    note_beat.clone(),
                    note_type.clone(),
                    hold_length.clone(),
                ));

                if hold_draw_pos <= 15.0 && is_active {
                    remove_holds.push((note_beat.clone(), note_type.clone(), hold_length.clone()))
                } else if hold_draw_pos <= -15.0 && !is_active {
                    remove_holds.push((note_beat.clone(), note_type.clone(), hold_length.clone()))
                }

                if is_active && hold_draw_pos - hold_width < 15.0 {
                    hold_width = hold_draw_pos - 13.0;
                }

                draw_hold(
                    note_type.clone(),
                    hold_draw_pos,
                    hold_width,
                    hold_note,
                    match is_active {
                        true => hold_thickness_multi,
                        false => 1.0,
                    },
                );
            }

            for remove_hold in &remove_holds {
                drawn_holds.retain(|x| x != remove_hold);
            }

            // Draw the active Notes
            for (note_beat, note_type, _hold_length) in &active_notes {
                let note_draw_pos =
                    ((note_beat - beat) * pixels_per_beat) + (ARROW_OFFSET - NOTE_SIZE / 2.0);
                draw_note(
                    note_type.clone(),
                    note_draw_pos,
                    input_note_left,
                    input_note_right,
                    input_note_up,
                    input_note_down,
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

            // ATTACKS!
            // Scale the thickness
            if ship_alpha_growing {
                ship_alpha += get_frame_time() * SCALE_ALPHA_PER_SECOND;
                if ship_alpha >= 1.0 {
                    ship_alpha_growing = false
                }
            } else {
                ship_alpha -= get_frame_time() * SCALE_ALPHA_PER_SECOND;
                if ship_alpha <= 0.25 {
                    ship_alpha_growing = true
                }
            }

            ship_invincibility -= get_frame_time();

            let mut remove_attacks = vec![];
            let mut warning_range_attacks = vec![];
            for (attack_beat, last_length, note_type) in &song_attacks {
                let note_offset = match note_type.clone() as i32 {
                    3 => UP_ARROW_POS,
                    4 => DOWN_ARROW_POS,
                    1 => RIGHT_ARROW_POS,
                    2 => LEFT_ARROW_POS,
                    _ => {
                        panic!("Error! Note type: '{note_type}' unknown")
                    }
                };

                if attack_beat.clone() + last_length.clone() <= beat {
                    remove_attacks.push((
                        attack_beat.clone(),
                        last_length.clone(),
                        note_type.clone(),
                    ));
                    continue;
                }

                if beat >= attack_beat.clone() - 5.0 && beat <= attack_beat.clone() {
                    let difference = 5.0 - (attack_beat.clone() - beat);

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

                    warning_range_attacks.push((
                        attack_beat.clone(),
                        last_length.clone(),
                        note_type.clone(),
                    ));
                }

                if attack_beat.clone() >= beat {
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

                if ship_height <= note_offset + 40.0
                    && ship_height >= note_offset - 40.0
                    && ship_invincibility <= 0.0
                {
                    health -= HEALTH_LOSS_LASER;
                    score -= SCORE_LOSS_LASER;
                    score_texts.push(ScoreText {
                        timer: TEXT_LAST_TIME,
                        score_type: ScoreType::Miss,
                        y_offset: note_offset,
                    });

                    ship_invincibility = 1.0;
                }
            }

            for (_, _, note_type) in &warning_range_attacks {
                let note_offset = match note_type.clone() as i32 {
                    3 => UP_ARROW_POS,
                    4 => DOWN_ARROW_POS,
                    1 => RIGHT_ARROW_POS,
                    2 => LEFT_ARROW_POS,
                    _ => {
                        panic!("Error! Note type: '{note_type}' unknown")
                    }
                };

                if !(wanted_ship_height <= note_offset + 40.0
                    && wanted_ship_height >= note_offset - 40.0
                    && ship_invincibility <= 0.0)
                {
                    continue;
                }

                let mut safe = false;

                let start_pos = wanted_ship_height;

                // Check Up
                for _ in 0..3 {
                    let (moved, location) = can_move(wanted_ship_height, true);

                    if is_laser(&song_attacks, beat, location) {
                        break;
                    }

                    if moved {
                        wanted_ship_height = location;
                        let (warning, _) = is_warning(&song_attacks, beat, location);
                        if !warning {
                            safe = true;
                            break;
                        }
                    } else {
                        break;
                    }
                }

                // Check Down
                if !safe {
                    wanted_ship_height = start_pos;
                    for _ in 0..3 {
                        let (moved, location) = can_move(wanted_ship_height, false);

                        if is_laser(&song_attacks, beat, location) {
                            break;
                        }

                        if moved {
                            wanted_ship_height = location;
                            let (warning, _) = is_warning(&song_attacks, beat, location);
                            if !warning {
                                safe = true;
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }

                if !safe {
                    wanted_ship_height = start_pos;
                    let mut safest_diff = is_warning(&song_attacks, beat, wanted_ship_height).1;
                    let mut safest_position = wanted_ship_height;

                    let start_height = wanted_ship_height;

                    for _ in 0..3 {
                        let (moved, location) = can_move(wanted_ship_height, true);

                        if is_laser(&song_attacks, beat, location) {
                            break;
                        }

                        if moved {
                            let (_, diff) = is_warning(&song_attacks, beat, location);
                            wanted_ship_height = location;
                            if safest_diff < diff {
                                safest_position = location;
                                safest_diff = diff;
                            }
                        }
                    }

                    wanted_ship_height = start_height;

                    for _ in 0..3 {
                        let (moved, location) = can_move(wanted_ship_height, false);

                        if is_laser(&song_attacks, beat, location) {
                            break;
                        }

                        if moved {
                            let (_, diff) = is_warning(&song_attacks, beat, location);
                            wanted_ship_height = location;
                            if safest_diff < diff {
                                safest_position = location;
                                safest_diff = diff;
                            }
                        }
                    }

                    wanted_ship_height = safest_position;
                }
            }

            for remove_attack in &remove_attacks {
                song_attacks.retain(|x| x != remove_attack);
            }

            let mut remove_texts = vec![];
            for score_text in &mut score_texts {
                if score_text.update_and_draw(font) {
                    remove_texts.push(score_text.clone());
                }
            }

            for remove_text in &remove_texts {
                score_texts.retain(|x| x != remove_text);
            }

            draw_text_justified(
                format!("SCORE: {}", score.separate_with_commas()).as_str(),
                vec2(5.0, 5.0),
                TextParams {
                    font,
                    font_size: 75,
                    font_scale: 0.25,
                    color: WHITE,
                    ..Default::default()
                },
                vec2(0.0, 1.0),
            );

            draw_text_justified(
                format!("{}", song.credits).as_str(),
                vec2(self.window_context.active_screen_size.x - 5.0, self.window_context.active_screen_size.y - 5.0),
                TextParams {
                    font,
                    font_size: 40,
                    font_scale: 0.25,
                    color: WHITE,
                    ..Default::default()
                },
                vec2(1.0, 0.0),
            );

            draw_text_justified(
                "Performed By Porpus",
                vec2(self.window_context.active_screen_size.x - 5.0, 5.0),
                TextParams {
                    font,
                    font_size: 50,
                    font_scale: 0.25,
                    color: WHITE,
                    ..Default::default()
                },
                vec2(1.0, 1.0),
            );

            // Clamp the health value to a max
            health = health.clamp(0, MAX_HEALTH);

            // Close Conditions
            if is_key_pressed(KeyCode::Escape) {
                return match self.return_to.clone() {
                    ReturnTo::MainMenu(difficulty, song_idx) => {
                        Some(Box::new(MainMenuScene {
                            window_context: self.window_context.clone(),
                            selected_difficulty: Some(difficulty),
                            selected_song_idx: Some(song_idx)
                        }))
                    }
                    ReturnTo::Editor => {
                        Some(Box::new(BeatmapEditorScene {
                            window_context: self.window_context.clone(),
                            song_path: self.song_path.clone()
                        }))
                    }
                }
            }

            if health <= 0 {
                game_over_timer.start();
            }

            game_over_timer.update();

            if game_over_timer.running {
                music
                    .set_playback_rate(
                        1.0f64 - game_over_timer.percent_done() as f64,
                        Default::default(),
                    )
                    .unwrap();

                draw_text_justified(
                    "GAME OVER",
                    vec2(
                        self.window_context.active_screen_size.x / 2.0,
                        (self.window_context.active_screen_size.y / 2.0)
                            * game_over_timer.percent_done(),
                    ),
                    TextParams {
                        font,
                        font_size: 250,
                        font_scale: 0.25,
                        color: Color::new(0.9, 0.8, 0.8, game_over_timer.percent_done()),
                        ..Default::default()
                    },
                    vec2(0.5, 0.5),
                );
            }

            if game_over_timer.is_done() {
                return match self.return_to.clone() {
                    ReturnTo::MainMenu(_, _) => {
                        Some(Box::new(GameEndScene {
                            return_to: self.return_to.clone(),
                            window_context: self.window_context.clone(),
                            file_path: self.song_path.clone(),
                            beat_level: false,
                            score,
                            perfect_notes,
                            good_notes: 0,
                            ok_notes: 0,
                            incorrect_notes: 0,
                            missed_notes: 0,
                        }))}
                    ReturnTo::Editor => { Some(Box::new(BeatmapEditorScene {
                        window_context: self.window_context.clone(),
                        song_path: self.song_path.clone()
                    })) }
                }
            }

            if music.position() >= song.song_length as f64 {
                return match self.return_to.clone() {
                    ReturnTo::MainMenu(_, _) => {
                        Some(Box::new(GameEndScene {
                            return_to: self.return_to.clone(),
                            window_context: self.window_context.clone(),
                            file_path: self.song_path.clone(),
                            beat_level: false,
                            score,
                            perfect_notes,
                            good_notes: 0,
                            ok_notes: 0,
                            incorrect_notes: 0,
                            missed_notes: 0,
                        }))}
                    ReturnTo::Editor => { Some(Box::new(BeatmapEditorScene {
                        window_context: self.window_context.clone(),
                        song_path: self.song_path.clone()
                    })) }
                }
            }

            draw_window(&mut self.window_context);

            next_frame().await;
        }
    }
}

pub fn can_move(current_loc: f32, up: bool) -> (bool, f32) {
    return if up {
        if current_loc == RIGHT_ARROW_POS {
            (true, UP_ARROW_POS)
        } else if current_loc == UP_ARROW_POS {
            (true, LEFT_ARROW_POS)
        } else if current_loc == DOWN_ARROW_POS {
            (true, RIGHT_ARROW_POS)
        } else {
            (false, 0.0)
        }
    } else {
        if current_loc == RIGHT_ARROW_POS {
            (true, DOWN_ARROW_POS)
        } else if current_loc == LEFT_ARROW_POS {
            (true, UP_ARROW_POS)
        } else if current_loc == UP_ARROW_POS {
            (true, RIGHT_ARROW_POS)
        } else {
            (false, 0.0)
        }
    };
}

pub fn is_warning(song_attacks: &Vec<(f32, f32, f32)>, beat: f32, check_type: f32) -> (bool, f32) {
    for (other_beat, _, other_type) in song_attacks {
        let other_offset = match other_type.clone() as i32 {
            3 => UP_ARROW_POS,
            4 => DOWN_ARROW_POS,
            1 => RIGHT_ARROW_POS,
            2 => LEFT_ARROW_POS,
            _ => {
                panic!("Error! Note type: '{other_type}' unknown")
            }
        };

        if other_beat.clone() - 5.0 <= beat
            && other_offset == check_type
            && other_beat.clone() >= beat
        {
            return (true, beat - other_beat.clone());
        }
    }

    (false, 0.0)
}

pub fn is_laser(song_attacks: &Vec<(f32, f32, f32)>, beat: f32, check_type: f32) -> bool {
    for (other_beat, _, other_type) in song_attacks {
        let other_offset = match other_type.clone() as i32 {
            3 => UP_ARROW_POS,
            4 => DOWN_ARROW_POS,
            1 => RIGHT_ARROW_POS,
            2 => LEFT_ARROW_POS,
            _ => {
                panic!("Error! Note type: '{other_type}' unknown")
            }
        };

        if other_beat.clone() <= beat && other_offset == check_type {
            return true;
        }
    }

    false
}
