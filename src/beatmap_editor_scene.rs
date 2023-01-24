use std::fs::File;
use std::io::Write;
use async_trait::async_trait;
use egui_macroquad::egui;
use kira::manager::{AudioManager, AudioManagerSettings};
use kira::manager::backend::cpal::CpalBackend;
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use macroquad::prelude::*;
use macroquad_aspect::prelude::*;

use crate::main_menu_scene::MainMenuScene;
use crate::note_gameplay_scene::constants::{ARROW_OFFSET, BEATS_TO_NOTE_HIT, DOWN_ARROW_POS, LEFT_ARROW_POS, NOTE_SIZE, NOTE_START_POS, RIGHT_ARROW_POS, UP_ARROW_POS};
use crate::note_gameplay_scene::{draw_hold, draw_note, NoteGameplayScene};
use crate::note_gameplay_scene::song::Song;
use crate::scene::Scene;
use crate::ui::draw_text_justified;
use crate::utils::{is_hovering_rect, quick_load_texture};

pub struct BeatmapEditorScene {
    pub window_context: WindowContext
}

#[async_trait]
impl Scene for BeatmapEditorScene {
    async fn run(&mut self) -> Option<Box<dyn Scene>> {

        let mut last_functional_song_path = "assets/songs/easy/dropit.json".to_string();
        let mut song_path = "assets/songs/easy/dropit.json".to_string();

        let song_json = load_string(&song_path).await.unwrap();
        let mut song = serde_json::from_str::<Song>(song_json.as_str()).unwrap();

        let mut beats_per_second = song.bpm / 60.0;
        let mut pixels_per_beat = (NOTE_START_POS - ARROW_OFFSET) / BEATS_TO_NOTE_HIT;

        let font = load_ttf_font("assets/fonts/pixel.ttf").await.unwrap();

        let mut sound_manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default()).unwrap();
        let mut sound = StaticSoundData::from_file(
            song.song_filepath.clone(),
            StaticSoundSettings::default(),
        ).unwrap();

        let mut music = sound_manager.play(sound).unwrap();
        let mut song_position;

        let mut reload = false;
        let mut test = false;

        let mut selected_note: usize = 10_000_000;
        let mut selected_attack: usize = 10_000_000;

        // Input Notes
        let input_note_up = quick_load_texture("assets/images/arrow_up.png").await;
        let input_note_down = quick_load_texture("assets/images/arrow_down.png").await;
        let input_note_left = quick_load_texture("assets/images/arrow_left.png").await;
        let input_note_right = quick_load_texture("assets/images/arrow_right.png").await;
        let hold_note = quick_load_texture("assets/images/hold.png").await;

        loop {
            clear_background(BLACK);
            set_camera(&self.window_context.camera);
            clear_background(BLACK);

            let beat = beats_per_second * ((music.position() * 1_000_000.0).round() / 1_000_000.0) as f32;

            song_position = music.position();
            egui_macroquad::ui(|egui_ctx| {
                egui::Window::new("Main Editor")
                    .show(egui_ctx, |ui| {

                        ui.text_edit_singleline(&mut song_path);

                        if ui.button("Save").clicked() {
                            let mut file = File::create(song_path.clone()).unwrap();
                            let cloned_song = song.clone();
                            file.write_all(serde_json::to_string_pretty(&cloned_song).unwrap().as_ref()).unwrap();
                            reload = true;
                        }

                        if ui.button("Load").clicked() {
                            reload = true;
                        }

                        if ui.button("Pause").clicked() {
                            music.pause(Default::default()).unwrap()
                        }
                        if ui.button("Play").clicked() {
                            music.resume(Default::default()).unwrap()
                        }

                        ui.add(egui::Slider::new(&mut song_position, 0.0..=song.song_length as f64).clamp_to_range(true));

                        if ui.button("Test Song").clicked() {
                            reload = true;
                            test = true;
                        }
                    });
                egui::Window::new("Note Editor")
                    .current_pos((0.0, 1200.0))
                    .show(egui_ctx, |ui| {
                        if selected_note != 10_000_000 {
                            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                ui.add(egui::DragValue::new(&mut song.notes[selected_note].0).speed(0.0));
                                ui.add(egui::DragValue::new(&mut song.notes[selected_note].2).speed(0.0));

                                if ui.button("Delete").clicked() || is_key_pressed(KeyCode::Delete) {
                                    song.notes.remove(selected_note);
                                    selected_note = 10_000_000;
                                }
                                if ui.button("Duplicate").clicked() || is_key_pressed(KeyCode::C) {
                                    let note = song.notes[selected_note].clone();
                                    song.notes.push((note.0 + 0.125, note.1, note.2));
                                    selected_note = song.notes.len() - 1;
                                }
                                if ui.button("Rotate").clicked() || is_key_pressed(KeyCode::R) {
                                    song.notes[selected_note].1 += 1.0;
                                    if song.notes[selected_note].1 > 4.0 {
                                        song.notes[selected_note].1 = 1.0;
                                    }
                                }
                            });
                        }

                        if ui.button("New").clicked() || is_key_pressed(KeyCode::N) {
                            song.notes.push(((beat + 1.5).floor(), 1.0, 0.0));
                            selected_note = song.notes.len() - 1;
                        }
                    });
                egui::Window::new("Attack Editor")
                    .current_pos((600.0, 0.0))
                    .show(egui_ctx, |ui| {
                        if selected_attack != 10_000_000 {
                            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                ui.add(egui::DragValue::new(&mut song.attacks[selected_attack].0).speed(0.0));
                                ui.add(egui::DragValue::new(&mut song.attacks[selected_attack].1).speed(0.0));
                                if ui.button("Rotate").clicked() {
                                    song.attacks[selected_attack].2 += 1.0;
                                    if song.attacks[selected_attack].2 > 4.0 {
                                        song.attacks[selected_attack].2 = 1.0;
                                    }
                                }

                                if ui.button("Delete").clicked() {
                                    song.attacks.remove(selected_attack);
                                    selected_attack = 10_000_000
                                }
                            });
                        }

                        if ui.button("New").clicked() {
                            song.attacks.push(((beat - 1.5).floor(), 4.0, 1.0));
                            selected_attack = song.attacks.len() - 1;
                        }
                    });

            });

            if reload {
                reload = false;

                if let Ok(song_json) = load_string(&song_path).await {
                    song = serde_json::from_str::<Song>(song_json.as_str()).unwrap();

                    beats_per_second = song.bpm / 60.0;
                    pixels_per_beat = (NOTE_START_POS - ARROW_OFFSET) / BEATS_TO_NOTE_HIT;

                    sound_manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default()).unwrap();
                    sound = StaticSoundData::from_file(
                        song.song_filepath.clone(),
                        StaticSoundSettings::default(),
                    ).unwrap();
                    music = sound_manager.play(sound).unwrap();
                    last_functional_song_path = song_path.clone();
                } else {
                    song_path = last_functional_song_path.clone();
                }
            }

            if test {
                return Some(Box::new(NoteGameplayScene::new(self.window_context.clone(), &song_path)))
            }

            if is_key_pressed(KeyCode::I) {
                song_position += match is_key_down(KeyCode::LeftShift) {
                    true => 1.0,
                    false => 0.5
                } * beats_per_second as f64;
            }
            else if is_key_pressed(KeyCode::K) {
                song_position -= match is_key_down(KeyCode::LeftShift) {
                    true => 1.0,
                    false => 0.5
                } * beats_per_second as f64;
            }

            if music.position() >= song.song_length as f64 {
                sound_manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default()).unwrap();
                sound = StaticSoundData::from_file(
                    song.song_filepath.clone(),
                    StaticSoundSettings::default(),
                ).unwrap();
                music = sound_manager.play(sound).unwrap();
            }

            if song_position != music.position() {
                music.seek_to(song_position).unwrap();
            }

            for i in 0..song.attacks.len() {
                let (attack_beat, last_length, note_type) = song.attacks[i];
                let note_offset = match note_type.clone() as i32 {
                    3 => UP_ARROW_POS,
                    4 => DOWN_ARROW_POS,
                    1 => RIGHT_ARROW_POS,
                    2 => LEFT_ARROW_POS,
                    _ => { panic!("Error! Note type: '{note_type}' unknown") }
                };

                if beat >= attack_beat.clone() - 2.0 && beat <= attack_beat.clone() {
                    draw_text_justified("!", vec2(0.0, note_offset), TextParams {
                        font,
                        font_size: 100,
                        font_scale: 0.25,
                        color: Color::new(0.8, 0.5, 0.4, 1.0),
                        ..Default::default()
                    }, vec2(0.0, 0.5));
                }

                if attack_beat.clone() >= beat || attack_beat.clone() + last_length.clone() <= beat {
                    continue;
                }

                if i == selected_attack {
                    draw_texture_ex(hold_note, 0.0, note_offset - 20.0,
                                    Color::new(1.0, 1.0, 1.0, 1.0), DrawTextureParams {
                            dest_size: Some(vec2(1000.0, 40.0)),
                            ..Default::default()
                        });
                } else {
                    draw_texture_ex(hold_note, 0.0, note_offset - 20.0,
                                    Color::new(1.0, 0.5, 0.6, 1.0), DrawTextureParams {
                            dest_size: Some(vec2(1000.0, 40.0)),
                            ..Default::default()
                        });
                }

                let mouse_pos = self.window_context.camera.screen_to_world(mouse_position().into());
                if is_hovering_rect(Rect::new(0.0, note_offset - 20.0, 708.0, 40.0), mouse_pos)
                    && is_mouse_button_released(MouseButton::Left) {
                    selected_attack = i;
                }

            }

            // Draw Every Note
            for i in 0..song.notes.len() {
                let (note_beat, note_type, hold_length) = song.notes[i];
                let note_offset = match note_type.clone() as i32 {
                    3 => UP_ARROW_POS,
                    4 => DOWN_ARROW_POS,
                    1 => RIGHT_ARROW_POS,
                    2 => LEFT_ARROW_POS,
                    _ => { panic!("Error! Note type: '{note_type}' unknown") }
                };

                let note_draw_pos = ((note_beat - beat) * pixels_per_beat) + (ARROW_OFFSET - NOTE_SIZE / 2.0);

                let hold_width = hold_length * pixels_per_beat;
                let hold_draw_pos = note_draw_pos + hold_width;

                if i == selected_note {
                    draw_hold_white(note_type.clone(), hold_draw_pos, hold_width, hold_note, 1.0);
                    draw_note_white(note_type.clone(), note_draw_pos, input_note_left, input_note_right, input_note_up, input_note_down);
                } else {
                    draw_hold(note_type.clone(), hold_draw_pos, hold_width, hold_note, 1.0);
                    draw_note(note_type.clone(), note_draw_pos, input_note_left, input_note_right, input_note_up, input_note_down);
                }



                let mouse_pos = self.window_context.camera.screen_to_world(mouse_position().into());
                if is_hovering_rect(Rect::new(note_draw_pos, note_offset - NOTE_SIZE / 2.0, NOTE_SIZE, NOTE_SIZE), mouse_pos) && is_mouse_button_released(MouseButton::Left) {
                    selected_note = i;
                }
            }

            if is_key_pressed(KeyCode::L) && selected_note != 10_000_000 {
                song.notes[selected_note].0 += match is_key_down(KeyCode::LeftShift) {
                    true => 0.25,
                    false => 0.125
                };
            }
            if is_key_pressed(KeyCode::J) && selected_note != 10_000_000 {
                song.notes[selected_note].0 -= match is_key_down(KeyCode::LeftShift) {
                    true => 0.25,
                    false => 0.125
                };
            }
            if is_key_pressed(KeyCode::Y) && selected_note != 10_000_000 {
                song.notes[selected_note].2 += match is_key_down(KeyCode::LeftShift) {
                    true => 0.25,
                    false => 0.125
                };

                song.notes[selected_note].2 = song.notes[selected_note].2.max(0.0);
            }
            if is_key_pressed(KeyCode::H) && selected_note != 10_000_000 {
                song.notes[selected_note].2 -= match is_key_down(KeyCode::LeftShift) {
                    true => 0.25,
                    false => 0.125
                };

                song.notes[selected_note].2 = song.notes[selected_note].2.max(0.0);
            }

            // Draw the Input Notes
            draw_texture_ex(input_note_left, ARROW_OFFSET - NOTE_SIZE / 2.0, LEFT_ARROW_POS - NOTE_SIZE / 2.0, GREEN, DrawTextureParams {
                dest_size: Some(vec2(NOTE_SIZE, NOTE_SIZE)),
                ..Default::default()
            });
            draw_texture_ex(input_note_up, ARROW_OFFSET - NOTE_SIZE / 2.0, UP_ARROW_POS - NOTE_SIZE / 2.0, SKYBLUE, DrawTextureParams {
                dest_size: Some(vec2(NOTE_SIZE, NOTE_SIZE)),
                ..Default::default()
            });
            draw_texture_ex(input_note_right, ARROW_OFFSET - NOTE_SIZE / 2.0, RIGHT_ARROW_POS - NOTE_SIZE / 2.0, ORANGE, DrawTextureParams {
                dest_size: Some(vec2(NOTE_SIZE, NOTE_SIZE)),
                ..Default::default()
            });
            draw_texture_ex(input_note_down, ARROW_OFFSET - NOTE_SIZE / 2.0, DOWN_ARROW_POS - NOTE_SIZE / 2.0, RED, DrawTextureParams {
                dest_size: Some(vec2(NOTE_SIZE, NOTE_SIZE)),
                ..Default::default()
            });

            draw_window(&mut self.window_context);

            set_default_camera();
            egui_macroquad::draw();

            // Quit Condition
            if is_key_pressed(KeyCode::Escape) {
                return Some(Box::new(MainMenuScene {
                    window_context: self.window_context.clone()
                }));
            }

            next_frame().await;
        }
    }
}

pub fn draw_note_white(direction: f32, location: f32, left_tex: Texture2D, right_tex: Texture2D, up_tex: Texture2D, down_tex: Texture2D) {
    let direction = direction.round() as i32;
    match direction {
        1 => { // Right
            draw_texture_ex(right_tex, location, RIGHT_ARROW_POS - NOTE_SIZE / 2.0, WHITE, DrawTextureParams {
                dest_size: Some(vec2(NOTE_SIZE, NOTE_SIZE)),
                ..Default::default()
            });
        },
        2 => { // Left
            draw_texture_ex(left_tex, location, LEFT_ARROW_POS - NOTE_SIZE / 2.0, WHITE, DrawTextureParams {
                dest_size: Some(vec2(NOTE_SIZE, NOTE_SIZE)),
                ..Default::default()
            });
        },
        3 => { // Up
            draw_texture_ex(up_tex, location, UP_ARROW_POS - NOTE_SIZE / 2.0, WHITE, DrawTextureParams {
                dest_size: Some(vec2(NOTE_SIZE, NOTE_SIZE)),
                ..Default::default()
            });
        },
        4 => { // Down
            draw_texture_ex(down_tex, location, DOWN_ARROW_POS - NOTE_SIZE / 2.0, WHITE, DrawTextureParams {
                dest_size: Some(vec2(NOTE_SIZE, NOTE_SIZE)),
                ..Default::default()
            });
        },
        _ => { todo!("Add direction drawing for note type.") }
    }
}

pub fn draw_hold_white(direction: f32, location: f32, width: f32, texture: Texture2D, thickness_multi: f32) {
    let direction = direction.round() as i32;
    let note_height = NOTE_SIZE * thickness_multi;
    let location = location + 20.0;
    match direction {
        1 => { // Right
            draw_texture_ex(texture, location, RIGHT_ARROW_POS - note_height / 2.0, WHITE, DrawTextureParams {
                dest_size: Some(vec2(-width, note_height)),
                ..Default::default()
            });
        },
        2 => { // Left
            draw_texture_ex(texture, location, LEFT_ARROW_POS - note_height / 2.0, WHITE, DrawTextureParams {
                dest_size: Some(vec2(-width, note_height)),
                ..Default::default()
            });
        },
        3 => { // Up
            draw_texture_ex(texture, location, UP_ARROW_POS - note_height / 2.0, WHITE, DrawTextureParams {
                dest_size: Some(vec2(-width, note_height)),
                ..Default::default()
            });
        },
        4 => { // Down
            draw_texture_ex(texture, location, DOWN_ARROW_POS - note_height / 2.0, WHITE, DrawTextureParams {
                dest_size: Some(vec2(-width, note_height)),
                ..Default::default()
            });
        },
        _ => { todo!("Add direction drawing for note type.") }
    }
}