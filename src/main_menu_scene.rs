use async_trait::async_trait;
use kira::manager::{AudioManager, AudioManagerSettings};
use kira::manager::backend::cpal::CpalBackend;
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use macroquad::prelude::*;
use macroquad_aspect::prelude::*;
use serde::{Deserialize, Serialize};
use thousands::Separable;

use crate::note_gameplay_scene::NoteGameplayScene;
use crate::scene::Scene;
use crate::ui::*;
use crate::utils::{quick_load_texture, Timer};

pub enum MenuState {
    MainMenu,
    PlayMenu,
    Settings,
    Loading
}

pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Expert,
    Extreme
}

#[derive(Serialize, Deserialize)]
pub struct SongDatabase {
    pub songs: Vec<SongData>
}

#[derive(Serialize, Deserialize)]
pub struct SongData {
    pub name: String,
    pub json_name: String,
    pub song_length: f32,
    pub high_score: i32
}

impl Difficulty {
    pub fn to_string(&self) -> String {
        match self {
            Difficulty::Easy => { "easy".to_string() }
            Difficulty::Medium => { "medium".to_string() }
            Difficulty::Hard => { "hard".to_string() }
            Difficulty::Expert => { "expert".to_string() }
            Difficulty::Extreme => { "extreme".to_string() }
        }
    }
}

pub struct MainMenuScene {
    pub window_context: WindowContext
}

#[async_trait]
impl Scene for MainMenuScene {
    async fn run(&mut self) -> Option<Box<dyn Scene>> {

        let mut state = MenuState::MainMenu;

        let background = quick_load_texture("assets/images/backgrounds/Space Background (15).png").await;

        let font = load_ttf_font("assets/fonts/pixel.ttf").await.unwrap();

        let frame = quick_load_texture("assets/images/ui/frame.png").await;
        let nine_slice_frame = NineSliceElement {
            tex: frame,
            corner_size: vec2(32.0, 32.0),
            vertical_size: vec2(32.0, 32.0),
            horizontal_size: vec2(32.0, 32.0)
        };

        let button_template = UITemplate::new(
            quick_load_texture("assets/images/ui/button.png").await,
            Color::new(1.0, 1.0, 1.0, 1.0),
            Some(Color::new(0.8, 0.8, 0.8, 1.0))
        );

        let faint_button_template = UITemplate::new(
            quick_load_texture("assets/images/ui/button.png").await,
            Color::new(1.0, 1.0, 1.0, 0.5),
            Some(Color::new(0.8, 0.8, 0.8, 0.5))
        );

        let mut sound_manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default()).unwrap();
        let sound = StaticSoundData::from_file(
            "assets/songs/music_files/ForestLullaby.wav",
            StaticSoundSettings::default(),
        ).unwrap();

        let mut music = sound_manager.play(sound).unwrap();

        let mut load_scene_timer = Timer::new(3.5, false);

        let mut quit_timer = Timer::new(1.5, false);

        let mut active_difficulty = Difficulty::Easy;

        let song_database = serde_json::from_str::<SongDatabase>(&load_string("assets/songs/song_data.json").await.unwrap()).unwrap();
        let mut chosen_song_idx = 0usize;

        let mut changing_song = false;

        loop {
            set_camera(&self.window_context.camera);

            draw_texture_ex(background, 0.0, 0.0, WHITE, Default::default());

            let mouse_pos = self.window_context.camera.screen_to_world(mouse_position().into());

            match state {
                MenuState::MainMenu => {
                    if element_text_template(
                        justify_rect(708.0 / 2.0, 50.0, 96.0 * 2.5, 26.0 * 2.25, vec2(0.5, 0.5)),
                        button_template, mouse_pos, "Play",
                        TextParams {
                            font,
                            font_size: 100,
                            font_scale: 0.25,
                            ..Default::default()
                        }
                    ) {
                        state = MenuState::PlayMenu;
                    }

                    if element_text_template(
                        justify_rect(708.0 / 2.0, 140.0, 96.0 * 2.5, 26.0 * 2.25, vec2(0.5, 0.5)),
                        button_template, mouse_pos, "Settings",
                        TextParams {
                            font,
                            font_size: 100,
                            font_scale: 0.25,
                            ..Default::default()
                        }
                    ) {
                        state = MenuState::Settings;
                    }

                    if element_text_template(
                        justify_rect(708.0 / 2.0, 400.0 - 15.0, 96.0 * 2.0, 26.0 * 2.0, vec2(0.5, 1.0)),
                        button_template, mouse_pos, "Quit",
                        TextParams {
                            font,
                            font_size: 80,
                            font_scale: 0.25,
                            ..Default::default()
                        }
                    ) {
                        quit_timer.start();
                    }
                }
                MenuState::PlayMenu => {
                    // Difficulty Selection Menu
                    nine_slice_frame.draw(
                        justify_rect(50.0, 50.0, 96.0 * 2.0, 96.0 * 2.5, vec2(0.0, 0.0))
                    );

                    // Easy Button
                    if element_text_template(
                        justify_rect(50.0 + 96.0, 75.0, 96.0 * 1.5, 30.0, vec2(0.5, 0.0)),
                        match active_difficulty {
                            Difficulty::Easy => { button_template }
                            _ => { faint_button_template }
                        },
                        mouse_pos,
                        "Easy",
                        TextParams {
                            font,
                            font_size: 50,
                            font_scale: 0.25,
                            ..Default::default()
                        }
                    ) {
                        active_difficulty = Difficulty::Easy
                    }

                    // Medium Button
                    if element_text_template(
                        justify_rect(50.0 + 96.0, 75.0 + 40.0, 96.0 * 1.5, 30.0, vec2(0.5, 0.0)),
                        match active_difficulty {
                            Difficulty::Medium => { button_template }
                            _ => { faint_button_template }
                        },
                        mouse_pos,
                        "Medium",
                        TextParams {
                            font,
                            font_size: 50,
                            font_scale: 0.25,
                            ..Default::default()
                        }
                    ) {
                        active_difficulty = Difficulty::Medium
                    }

                    // Hard Button
                    if element_text_template(
                        justify_rect(50.0 + 96.0, 75.0 + 80.0, 96.0 * 1.5, 30.0, vec2(0.5, 0.0)),
                        match active_difficulty {
                            Difficulty::Hard => { button_template }
                            _ => { faint_button_template }
                        },
                        mouse_pos,
                        "Hard",
                        TextParams {
                            font,
                            font_size: 50,
                            font_scale: 0.25,
                            ..Default::default()
                        }
                    ) {
                        active_difficulty = Difficulty::Hard
                    }

                    // Expert Button
                    if element_text_template(
                        justify_rect(50.0 + 96.0, 75.0 + 120.0, 96.0 * 1.5, 30.0, vec2(0.5, 0.0)),
                        match active_difficulty {
                            Difficulty::Expert => { button_template }
                            _ => { faint_button_template }
                        },
                        mouse_pos,
                        "Expert",
                        TextParams {
                            font,
                            font_size: 50,
                            font_scale: 0.25,
                            ..Default::default()
                        }
                    ) {
                        active_difficulty = Difficulty::Expert
                    }

                    // Extreme Button
                    if element_text_template(
                        justify_rect(50.0 + 96.0, 75.0 + 160.0, 96.0 * 1.5, 30.0, vec2(0.5, 0.0)),
                        match active_difficulty {
                            Difficulty::Extreme => { button_template }
                            _ => { faint_button_template }
                        },
                        mouse_pos,
                        "Extreme",
                        TextParams {
                            font,
                            font_size: 50,
                            font_scale: 0.25,
                            ..Default::default()
                        }
                    ) {
                        active_difficulty = Difficulty::Extreme
                    }

                    if !changing_song {
                        // Song Data Panel
                        nine_slice_frame.draw(justify_rect(self.window_context.active_screen_size.x - 50.0, 50.0, 400.0, 240.0, vec2(1.0, 0.0)));

                        draw_text_justified(
                            song_database.songs[chosen_song_idx].name.as_str(),
                            vec2(self.window_context.active_screen_size.x - 50.0 - 200.0, 80.0),
                            TextParams {
                                font,
                                font_size: 125,
                                font_scale: 0.25,
                                ..Default::default()
                            },vec2(0.5, 1.0));

                        draw_text_justified(
                            &format!("High Score: {}", song_database.songs[chosen_song_idx].high_score.separate_with_commas()),
                            vec2(self.window_context.active_screen_size.x - 425.0, 175.0),
                            TextParams {
                                font,
                                font_size: 40,
                                font_scale: 0.25,
                                ..Default::default()
                            },vec2(0.0, 1.0));

                        draw_text_justified(
                            &format!("Length: {} Seconds", song_database.songs[chosen_song_idx].song_length),
                            vec2(self.window_context.active_screen_size.x - 425.0, 125.0),
                            TextParams {
                                font,
                                font_size: 40,
                                font_scale: 0.25,
                                ..Default::default()
                            },vec2(0.0, 1.0));

                        // Change Song Button
                        if element_text_template(
                            justify_rect(self.window_context.active_screen_size.x - 50.0 - 100.0, self.window_context.active_screen_size.y - 60.0, 96.0 * 1.5, 26.0 * 1.5, vec2(0.5, 1.0)),
                            button_template,
                            mouse_pos,
                            "Songs",
                            TextParams {
                                font,
                                font_size: 80,
                                font_scale: 0.25,
                                ..Default::default()
                            }
                        ) {
                            changing_song = true;
                        }

                    } else {
                        // Song Choice Panel
                        nine_slice_frame.draw(justify_rect(self.window_context.active_screen_size.x - 250.0, 50.0, 200.0, 240.0, vec2(1.0, 0.0)));

                        for song_idx in 0..song_database.songs.len() {
                            if element_text_template(
                                justify_rect(self.window_context.active_screen_size.x - 425.0, 75.0 + (50.0 * song_idx as f32), 95.0 * 1.5, 26.0 * 1.5, vec2(0.0, 0.0)),
                                {
                                    if song_idx == chosen_song_idx {
                                        button_template
                                    } else {
                                        faint_button_template
                                    }
                                },
                                mouse_pos,
                                &song_database.songs[song_idx].name,
                                TextParams {
                                    font,
                                    font_size: 50,
                                    font_scale: 0.25,
                                    ..Default::default()
                                }
                            ) {
                                chosen_song_idx = song_idx;
                                changing_song = false;
                            }
                        }
                    }

                    // Play Button
                    if element_text_template(
                        justify_rect(self.window_context.active_screen_size.x - 50.0 - 300.0, self.window_context.active_screen_size.y - 60.0, 96.0 * 1.5, 26.0 * 1.5, vec2(0.5, 1.0)),
                        button_template,
                        mouse_pos,
                        "Play",
                        TextParams {
                            font,
                            font_size: 80,
                            font_scale: 0.25,
                            ..Default::default()
                        }
                    ) {
                        state = MenuState::Loading;
                        load_scene_timer.start();
                    }

                    // Back Button
                    if element_text_template(
                        justify_rect(50.0, self.window_context.active_screen_size.y - 70.0, 96.0 * 1.3, 26.0 * 1.3, vec2(0.0, 1.0)),
                        button_template,
                        mouse_pos,
                        "Back",
                        TextParams {
                            font,
                            font_size: 60,
                            font_scale: 0.25,
                            ..Default::default()
                        }
                    ) {
                        state = MenuState::MainMenu
                    }
                }
                MenuState::Settings => {
                    if element_text_template(
                        justify_rect(708.0 / 2.0, 400.0 - 15.0, 96.0 * 2.0, 26.0 * 2.0, vec2(0.5, 1.0)),
                        button_template, mouse_pos, "Back",
                        TextParams {
                            font,
                            font_size: 80,
                            font_scale: 0.25,
                            ..Default::default()
                        }
                    ) {
                        state = MenuState::MainMenu
                    }
                }
                MenuState::Loading => {
                    let dots = (load_scene_timer.percent_done() * 5.0).round() as i32 % 4;

                    let mut text = "Loading".to_string();

                    for _ in 0..dots {
                        text.push('.');
                    }

                    draw_text_justified(&text, vec2(708.0 / 2.0, 400.0 / 2.0), TextParams {
                        font,
                        font_size: 250,
                        font_scale: 0.25,
                        ..Default::default()
                    }, vec2(0.5, 0.5));
                }
            }

            load_scene_timer.update();

            if load_scene_timer.running {
                music.set_volume((1.0 - load_scene_timer.percent_done()) as f64, Default::default()).unwrap();
            }

            if load_scene_timer.is_done() {
                return Some(Box::new(NoteGameplayScene::new(
                    self.window_context.clone(),
                    format!("assets/songs/{}/{}", active_difficulty.to_string(), song_database.songs[chosen_song_idx].json_name).as_str()))
                );
            }

            if !load_scene_timer.running {
                quit_timer.update();

                if quit_timer.running {
                    music.set_volume((1.0 - quit_timer.percent_done()) as f64, Default::default()).unwrap();
                }

                if quit_timer.is_done() {
                    return None;
                }

                if is_key_pressed(KeyCode::Escape) {
                    quit_timer.start();
                }
            }

            draw_window(&mut self.window_context);

            next_frame().await;
        }
    }
}