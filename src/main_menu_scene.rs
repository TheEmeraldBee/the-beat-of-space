use std::fs::File;
use std::io::Write;
use async_trait::async_trait;
use kira::manager::{AudioManager, AudioManagerSettings};
use kira::manager::backend::cpal::CpalBackend;
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use macroquad::prelude::*;
use macroquad_aspect::prelude::*;
use serde::{Deserialize, Serialize};
use thousands::Separable;
use crate::beatmap_editor_scene::BeatmapEditorScene;

use crate::note_gameplay_scene::NoteGameplayScene;
use crate::note_gameplay_scene::song::Song;
use crate::scene::Scene;
use crate::ui::*;
use crate::utils::{Config, quick_load_texture, Timer};

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
    pub difficulties: Vec<String>
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

        let faint_button_template = UITemplate::new(
            nine_slice_button,
            Color::new(1.0, 1.0, 1.0, 0.5),
            Some(Color::new(0.8, 0.8, 0.8, 0.5))
        );

        let plus_template = UITemplate::new(
            Element {
                tex: quick_load_texture("assets/images/ui/plus.png").await,
                element_type: ElementType::Texture
            },
            Color::new(1.0, 1.0, 1.0, 1.0),
            Some(Color::new(0.8, 0.8, 0.8, 1.0))
        );

        let minus_template = UITemplate::new(
            Element {
                tex: quick_load_texture("assets/images/ui/minus.png").await,
                element_type: ElementType::Texture
            },
            Color::new(1.0, 1.0, 1.0, 1.0),
            Some(Color::new(0.8, 0.8, 0.8, 1.0))
        );

        let mut sound_manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default()).unwrap();
        let sound = StaticSoundData::from_file(
            "assets/songs/music_files/ForestLullaby.wav",
            StaticSoundSettings::default(),
        ).unwrap();

        let mut music = sound_manager.play(sound).unwrap();

        let mut load_scene_timer = Timer::new(3.5, false);

        let mut active_difficulty = Difficulty::Easy;

        let song_database = serde_json::from_str::<SongDatabase>(&load_string("assets/song_data.json").await.unwrap()).unwrap();
        let mut chosen_song_idx = 0usize;

        let mut song = serde_json::from_str::<Song>(&load_string(&format!("assets/songs/{}/{}", active_difficulty.to_string(), song_database.songs[chosen_song_idx].json_name)).await.unwrap()).unwrap();

        let mut config = serde_json::from_str::<Config>(&load_string("assets/config.json").await.unwrap()).unwrap();

        music.set_volume(config.volume, Default::default()).unwrap();

        let mut changing_song = false;

        let mut play_button_pos = 0.0;
        let mut settings_button_pos = 0.0;
        let mut quit_button_pos = 0.0;

        let start_fullscreen = config.fullscreen;

        loop {
            set_camera(&self.window_context.camera);

            draw_texture_ex(background, 0.0, 0.0, WHITE, Default::default());

            let mouse_pos = self.window_context.camera.screen_to_world(mouse_position().into());

            match state {
                MenuState::MainMenu => {
                    let mut play_rect = justify_rect(50.0, 50.0, 96.0 * 1.5, 26.0 * 1.25, vec2(0.0, 0.5));
                    if hover_rect(play_rect, mouse_pos) {
                        play_button_pos += 20.0 * (play_button_pos + 1.0) * get_frame_time();
                    } else {
                        play_button_pos -= 20.0 * (play_button_pos + 1.0) * get_frame_time()
                    }
                    play_button_pos = clamp(play_button_pos, 0.0, 25.0);
                    play_rect.x += play_button_pos;

                    if element_text_template(
                        play_rect,
                        button_template, mouse_pos, "Play Songs",
                        TextParams {
                            font,
                            font_size: 50,
                            font_scale: 0.25,
                            ..Default::default()
                        }
                    ).clicked() {
                        state = MenuState::PlayMenu;
                    }

                    let mut settings_rect = justify_rect(50.0, 100.0, 96.0 * 1.25, 26.0, vec2(0.0, 0.5));
                    if hover_rect(settings_rect, mouse_pos) {
                        settings_button_pos += 20.0 * (settings_button_pos + 1.0) * get_frame_time();
                    } else {
                        settings_button_pos -= 20.0 * (settings_button_pos + 1.0) * get_frame_time()
                    }
                    settings_button_pos = clamp(settings_button_pos, 0.0, 25.0);
                    settings_rect.x += settings_button_pos;

                    if element_text_template(
                        settings_rect,
                        button_template, mouse_pos, "Settings",
                        TextParams {
                            font,
                            font_size: 50,
                            font_scale: 0.25,
                            ..Default::default()
                        }
                    ).clicked() {
                        state = MenuState::Settings;
                    }

                    let mut quit_rect = justify_rect(50.0, 145.0, 96.0 * 1.1, 26.0 * 0.85, vec2(0.0, 0.5));
                    if hover_rect(quit_rect, mouse_pos) {
                        quit_button_pos += 20.0 * (quit_button_pos + 1.0) * get_frame_time();
                    } else {
                        quit_button_pos -= 20.0 * (quit_button_pos + 1.0) * get_frame_time()
                    }
                    quit_button_pos = clamp(quit_button_pos, 0.0, 25.0);
                    quit_rect.x += quit_button_pos;

                    if element_text_template(
                        quit_rect,
                        button_template, mouse_pos, "Quit",
                        TextParams {
                            font,
                            font_size: 45,
                            font_scale: 0.25,
                            ..Default::default()
                        }
                    ).clicked() {
                        return None
                    }
                }
                MenuState::PlayMenu => {
                    // Difficulty Selection Menu
                    nine_slice_frame.draw(
                        justify_rect(50.0, 50.0, self.window_context.active_screen_size.x / 4.0, 96.0 * 2.5, vec2(0.0, 0.0)),
                        WHITE
                    );

                    // Easy Button
                    if element_text_template(
                        justify_rect(50.0 + self.window_context.active_screen_size.x / 8.0, 75.0, self.window_context.active_screen_size.x / 6.0, 30.0, vec2(0.5, 0.0)),
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
                    ).clicked() {
                        active_difficulty = Difficulty::Easy;
                        if !song_database.songs[chosen_song_idx].difficulties.contains(&active_difficulty.to_string()) {
                            for idx in 0..song_database.songs.len() {
                                if song_database.songs[idx].difficulties.contains(&active_difficulty.to_string()) {
                                    chosen_song_idx = idx;
                                    break;
                                }
                            }
                        }
                        song = serde_json::from_str::<Song>(&load_string(&format!("assets/songs/{}/{}", active_difficulty.to_string(), song_database.songs[chosen_song_idx].json_name)).await.unwrap()).unwrap();
                    }

                    // Medium Button
                    if element_text_template(
                        justify_rect(50.0 + self.window_context.active_screen_size.x / 8.0, 75.0 + 40.0, self.window_context.active_screen_size.x / 6.0, 30.0, vec2(0.5, 0.0)),
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
                    ).clicked() {
                        active_difficulty = Difficulty::Medium;
                        if !song_database.songs[chosen_song_idx].difficulties.contains(&active_difficulty.to_string()) {
                            for idx in 0..song_database.songs.len() {
                                if song_database.songs[idx].difficulties.contains(&active_difficulty.to_string()) {
                                    chosen_song_idx = idx;
                                    break;
                                }
                            }
                        }
                        song = serde_json::from_str::<Song>(&load_string(&format!("assets/songs/{}/{}", active_difficulty.to_string(), song_database.songs[chosen_song_idx].json_name)).await.unwrap()).unwrap();
                    }

                    // Hard Button
                    if element_text_template(
                        justify_rect(50.0 + self.window_context.active_screen_size.x / 8.0, 75.0 + 80.0, self.window_context.active_screen_size.x / 6.0, 30.0, vec2(0.5, 0.0)),
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
                    ).clicked() {
                        active_difficulty = Difficulty::Hard;
                        if !song_database.songs[chosen_song_idx].difficulties.contains(&active_difficulty.to_string()) {
                            for idx in 0..song_database.songs.len() {
                                if song_database.songs[idx].difficulties.contains(&active_difficulty.to_string()) {
                                    chosen_song_idx = idx;
                                    break;
                                }
                            }
                        }
                        song = serde_json::from_str::<Song>(&load_string(&format!("assets/songs/{}/{}", active_difficulty.to_string(), song_database.songs[chosen_song_idx].json_name)).await.unwrap()).unwrap();
                    }

                    // Expert Button
                    if element_text_template(
                        justify_rect(50.0 + self.window_context.active_screen_size.x / 8.0, 75.0 + 120.0, self.window_context.active_screen_size.x / 6.0, 30.0, vec2(0.5, 0.0)),
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
                    ).clicked() {
                        active_difficulty = Difficulty::Expert;
                        if !song_database.songs[chosen_song_idx].difficulties.contains(&active_difficulty.to_string()) {
                            for idx in 0..song_database.songs.len() {
                                if song_database.songs[idx].difficulties.contains(&active_difficulty.to_string()) {
                                    chosen_song_idx = idx;
                                    break;
                                }
                            }
                        }
                        song = serde_json::from_str::<Song>(&load_string(&format!("assets/songs/{}/{}", active_difficulty.to_string(), song_database.songs[chosen_song_idx].json_name)).await.unwrap()).unwrap();
                    }

                    // Extreme Button
                    if element_text_template(
                        justify_rect(50.0 + self.window_context.active_screen_size.x / 8.0, 75.0 + 160.0, self.window_context.active_screen_size.x / 6.0, 30.0, vec2(0.5, 0.0)),
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
                    ).clicked() {
                        active_difficulty = Difficulty::Extreme;
                        if !song_database.songs[chosen_song_idx].difficulties.contains(&active_difficulty.to_string()) {
                            for idx in 0..song_database.songs.len() {
                                if song_database.songs[idx].difficulties.contains(&active_difficulty.to_string()) {
                                    chosen_song_idx = idx;
                                    break;
                                }
                            }
                        }
                        song = serde_json::from_str::<Song>(&load_string(&format!("assets/songs/{}/{}", active_difficulty.to_string(), song_database.songs[chosen_song_idx].json_name)).await.unwrap()).unwrap();
                    }

                    let song_data_left = self.window_context.active_screen_size.x - 50.0 - self.window_context.active_screen_size.x * 0.56;
                    let song_data_center = self.window_context.active_screen_size.x - 50.0 - (self.window_context.active_screen_size.x * 0.56) / 2.0;

                    if !changing_song {
                        // Song Data Panel
                        nine_slice_frame.draw(justify_rect(self.window_context.active_screen_size.x - 50.0, 50.0, self.window_context.active_screen_size.x * 0.56, 240.0, vec2(1.0, 0.0)), WHITE);

                        draw_text_justified(
                            song_database.songs[chosen_song_idx].name.as_str(),
                            vec2(song_data_center, 80.0),
                            TextParams {
                                font,
                                font_size: 125,
                                font_scale: 0.25,
                                ..Default::default()
                            },vec2(0.5, 1.0));

                        draw_text_justified(
                            &format!("High Score: {}", song.high_score.separate_with_commas()),
                            vec2(song_data_left + 25.0, 175.0),
                            TextParams {
                                font,
                                font_size: 40,
                                font_scale: 0.25,
                                ..Default::default()
                            },vec2(0.0, 1.0));

                        draw_text_justified(
                            &format!("Length: {} Seconds", song.song_length),
                            vec2(song_data_left + 25.0, 125.0),
                            TextParams {
                                font,
                                font_size: 40,
                                font_scale: 0.25,
                                ..Default::default()
                            },vec2(0.0, 1.0));

                        // Change Song Button
                        if element_text_template(
                            justify_rect(song_data_center + 100.0, self.window_context.active_screen_size.y - 60.0, 96.0 * 1.5, 26.0 * 1.5, vec2(0.5, 1.0)),
                            button_template,
                            mouse_pos,
                            "Songs",
                            TextParams {
                                font,
                                font_size: 80,
                                font_scale: 0.25,
                                ..Default::default()
                            }
                        ).clicked() {
                            changing_song = true;
                        }

                    } else {
                        // Song Choice Panel
                        nine_slice_frame.draw(justify_rect(song_data_center - 100.0, 50.0, self.window_context.active_screen_size.x * 0.28, 240.0, vec2(0.5, 0.0)), WHITE);

                        let mut total_songs = 0;
                        for song_idx in 0..song_database.songs.len() {
                            if song_database.songs[song_idx].difficulties.contains(&active_difficulty.to_string()) {
                                if element_text_template(
                                    justify_rect(song_data_center - 100.0, 75.0 + (50.0 * total_songs as f32), self.window_context.active_screen_size.x * 0.22, 26.0 * 1.5, vec2(0.5, 0.0)),
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
                                ).clicked() {
                                    chosen_song_idx = song_idx;
                                    changing_song = false;
                                    song = serde_json::from_str::<Song>(&load_string(&format!("assets/songs/{}/{}", active_difficulty.to_string(), song_database.songs[chosen_song_idx].json_name)).await.unwrap()).unwrap();
                                }

                                total_songs += 1;
                            }
                        }
                    }

                    // Play Button
                    if element_text_template(
                        justify_rect(song_data_center - 100.0, self.window_context.active_screen_size.y - 60.0, 96.0 * 1.5, 26.0 * 1.5, vec2(0.5, 1.0)),
                        button_template,
                        mouse_pos,
                        "Play",
                        TextParams {
                            font,
                            font_size: 80,
                            font_scale: 0.25,
                            ..Default::default()
                        }
                    ).clicked() {
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
                    ).clicked() {
                        state = MenuState::MainMenu
                    }
                }
                MenuState::Settings => {
                    if element_text_template(
                        justify_rect(50.0, self.window_context.active_screen_size.y - 15.0, 96.0 * 1.35, 26.0 * 1.1, vec2(0.0, 1.0)),
                        button_template, mouse_pos, "Back",
                        TextParams {
                            font,
                            font_size: 45,
                            font_scale: 0.25,
                            ..Default::default()
                        }
                    ).clicked() {
                        state = MenuState::MainMenu
                    }

                    nine_slice_frame.draw(justify_rect(40.0, 50.0, 200.0, 40.0, vec2(0.0, 0.5)), WHITE);

                    draw_text_justified("Volume: ", vec2(50.0, 50.0), TextParams {
                        font,
                        font_size: 45,
                        font_scale: 0.25,
                        ..Default::default()
                    }, vec2(0.0, 0.5));

                    if element_template(justify_rect(125.0, 50.0, 18.0, 8.0, vec2(0.0, 0.5)), minus_template, mouse_pos).clicked() {
                        config.volume -= 0.05;
                        config.volume = clamp(config.volume, 0.0, 1.0);

                        let mut data = File::create("assets/config.json").unwrap();
                        data.write_all((serde_json::to_string_pretty(&config).unwrap()).as_ref()).unwrap();
                        config = serde_json::from_str::<Config>(&load_string("assets/config.json").await.unwrap()).unwrap();

                        music.set_volume(config.volume, Default::default()).unwrap();
                    }

                    draw_text_justified(&format!("{}%", (config.volume * 100.0).round()), vec2(150.0, 50.0), TextParams {
                        font,
                        font_size: 45,
                        font_scale: 0.25,
                        ..Default::default()
                    }, vec2(0.0, 0.5));

                    if element_template(justify_rect(195.0, 50.0, 18.0, 18.0, vec2(0.0, 0.5)), plus_template, mouse_pos).clicked() {
                        config.volume += 0.05;
                        config.volume = clamp(config.volume, 0.0, 1.0);

                        let mut data = File::create("assets/config.json").unwrap();
                        data.write_all((serde_json::to_string_pretty(&config).unwrap()).as_ref()).unwrap();
                        config = serde_json::from_str::<Config>(&load_string("assets/config.json").await.unwrap()).unwrap();

                        music.set_volume(config.volume, Default::default()).unwrap();
                    }

                    if element_text_template(
                        justify_rect(40.0, 100.0, 96.0 * 2.0, 18.0 * 1.8, vec2(0.0, 0.5)),
                        button_template,
                        mouse_pos,
                        &format!("Fullscreen: {}", match config.fullscreen {
                            true => { "On" }
                            false => { "Off" }
                        }),
                        TextParams {
                            font,
                            font_size: 45,
                            font_scale: 0.25,
                            ..Default::default()
                        }
                    ).clicked() {
                        config.fullscreen = !config.fullscreen;

                        let mut data = File::create("assets/config.json").unwrap();
                        data.write_all((serde_json::to_string_pretty(&config).unwrap()).as_ref()).unwrap();
                        config = serde_json::from_str::<Config>(&load_string("assets/config.json").await.unwrap()).unwrap();
                    }

                    if config.fullscreen != start_fullscreen {
                        draw_text_justified(
                            "Restart required to apply change.",
                            vec2(45.0, 125.0),
                            TextParams {
                                font,
                                font_size: 28,
                                font_scale: 0.25,
                                ..Default::default()
                            }, vec2(0.0, 1.0)
                        );
                    }
                }
                MenuState::Loading => {
                    let dots = (load_scene_timer.percent_done() * 5.0).round() as i32 % 4;

                    let mut text = "Loading".to_string();

                    for _ in 0..dots {
                        text.push('.');
                    }

                    draw_text_justified(&text, vec2(25.0, 400.0 - 25.0), TextParams {
                        font,
                        font_size: 150,
                        font_scale: 0.25,
                        ..Default::default()
                    }, vec2(0.0, 0.0));
                }
            }

            load_scene_timer.update();

            if load_scene_timer.running {
                music.set_volume(config.volume * (1.0 - load_scene_timer.percent_done()) as f64, Default::default()).unwrap();
            }

            if load_scene_timer.is_done() {
                return Some(Box::new(NoteGameplayScene::new(
                    self.window_context.clone(),
                    format!("assets/songs/{}/{}", active_difficulty.to_string(), song_database.songs[chosen_song_idx].json_name).as_str()))
                );
            }

            if is_key_pressed(KeyCode::F12) {
                return Some(Box::new(BeatmapEditorScene { window_context: self.window_context.clone() }));
            }

            if is_key_pressed(KeyCode::Escape) {
                return None;
            }

            draw_window(&mut self.window_context);

            next_frame().await;
        }
    }
}