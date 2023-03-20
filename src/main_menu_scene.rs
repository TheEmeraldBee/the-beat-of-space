use std::fmt::{Display, Formatter};
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
use crate::error_scene::ErrorScene;

use crate::note_gameplay_scene::{NoteGameplayScene, ReturnTo};
use crate::note_gameplay_scene::song::Song;
use crate::porpus_scene::PorpusScene;
use crate::scene::Scene;
use crate::ui::*;
use crate::utils::{Config, key_code_to_u32, quick_load_texture, Timer, u32_to_key_code};

pub enum MenuState {
    MainMenu,
    PlayMenu,
    Settings,
    Loading
}

#[derive(Clone)]
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

impl Display for Difficulty {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Difficulty::Easy => { "easy".to_string() }
            Difficulty::Medium => { "medium".to_string() }
            Difficulty::Hard => { "hard".to_string() }
            Difficulty::Expert => { "expert".to_string() }
            Difficulty::Extreme => { "extreme".to_string() }
        })
    }
}

pub struct MainMenuScene {
    pub window_context: WindowContext,
    pub selected_difficulty: Option<Difficulty>,
    pub selected_song_idx: Option<usize>
}

#[async_trait]
impl Scene for MainMenuScene {
    async fn run(&mut self) -> Option<Box<dyn Scene>> {

        let mut state = MenuState::MainMenu;

        let background = match quick_load_texture("assets/images/backgrounds/Space Background (15).png").await  {
            Ok(tex) => tex,
            Err(_) => return Some(Box::new(ErrorScene::new("Assets Missing (Verify Game Files or Reinstall)", self.window_context.clone())))
        };

        let font = match load_ttf_font("assets/fonts/pixel.ttf").await  {
            Ok(font) => font,
            Err(_) => return Some(Box::new(ErrorScene::new("Assets Missing (Verify Game Files or Reinstall)", self.window_context.clone())))
        };

        let frame = match quick_load_texture("assets/images/ui/frame.png").await {
            Ok(tex) => tex,
            Err(_) => return Some(Box::new(ErrorScene::new("Assets Missing (Verify Game Files or Reinstall)", self.window_context.clone())))
        };
        let nine_slice_frame = Element {
            tex: frame,
            element_type: ElementType::NineSlice(vec2(10.0, 10.0))
        };

        let nine_slice_button = Element {
            tex: match quick_load_texture("assets/images/ui/button.png").await  {
                Ok(tex) => tex,
                Err(_) => return Some(Box::new(ErrorScene::new("Assets Missing (Verify Game Files or Reinstall)", self.window_context.clone())))
            },
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
                tex: match quick_load_texture("assets/images/ui/plus.png").await  {
                    Ok(tex) => tex,
                    Err(_) => return Some(Box::new(ErrorScene::new("Assets Missing (Verify Game Files or Reinstall)", self.window_context.clone())))
                },
                element_type: ElementType::Texture
            },
            Color::new(1.0, 1.0, 1.0, 1.0),
            Some(Color::new(0.8, 0.8, 0.8, 1.0))
        );

        let minus_template = UITemplate::new(
            Element {
                tex: match quick_load_texture("assets/images/ui/minus.png").await {
                Ok(tex) => tex,
                Err(_) => return Some(Box::new(ErrorScene::new("Assets Missing (Verify Game Files or Reinstall)", self.window_context.clone())))
            },
                element_type: ElementType::Texture
            },
            Color::new(1.0, 1.0, 1.0, 1.0),
            Some(Color::new(0.8, 0.8, 0.8, 1.0))
        );

        let mut sound_manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default()).unwrap();
        let sound = match StaticSoundData::from_file(
            "assets/songs/music_files/ForestLullaby.wav",
            StaticSoundSettings::default(),
        ) {
            Ok(song) => song,
            Err(_) => return Some(Box::new(ErrorScene::new("Assets Missing (Verify Game Files or Reinstall)", self.window_context.clone())))
        };

        let mut music = sound_manager.play(sound).unwrap();

        let mut load_scene_timer = Timer::new(3.5, false);
        let mut load_watch_timer = Timer::new(3.5, false);

        let mut active_difficulty = match self.selected_difficulty.clone() {
            Some(difficulty) => difficulty,
            None => Difficulty::Easy
        };

        let song_database = match serde_json::from_str::<SongDatabase>(&match load_string("assets/song_data.json").await {
            Ok(tex) => tex,
            Err(_) => return Some(Box::new(ErrorScene::new("song_data missing, please fix or reinstall", self.window_context.clone())))
        }) {
            Ok(tex) => tex,
            Err(_) => return Some(Box::new(ErrorScene::new("song_data format is wrong", self.window_context.clone())))
        };
        let mut chosen_song_idx = self.selected_song_idx.unwrap_or(0);

        let mut song = match serde_json::from_str::<Song>(&match load_string(&format!("assets/songs/{}/{}", active_difficulty, song_database.songs[chosen_song_idx].json_name)).await {
            Ok(tex) => tex,
            Err(_) => return Some(Box::new(ErrorScene::new("Assets Missing (Verify Game Files or Reinstall)", self.window_context.clone())))
        }) {
            Ok(tex) => tex,
            Err(_) => return Some(Box::new(ErrorScene::new("Song Format Incorrect", self.window_context.clone())))
        };
        let mut config = serde_json::from_str::<Config>(&load_string("assets/config.json").await.unwrap()).unwrap();

        music.set_volume(config.volume, Default::default()).unwrap();

        let mut changing_song = false;

        let mut play_button_pos = 0.0;
        let mut settings_button_pos = 0.0;
        let mut quit_button_pos = 0.0;

        let start_fullscreen = config.fullscreen;

        let mut checking_input = 0;

        let mut fps_display = false;

        loop {
            set_camera(&self.window_context.camera);

            draw_texture_ex(background, 0.0, 0.0, WHITE, Default::default());

            let mouse_pos = self.window_context.camera.screen_to_world(mouse_position().into());

            if is_key_pressed(KeyCode::F3) {
                fps_display = !fps_display;
            }

            if fps_display {
                draw_text_justified(
                    format!("{}", get_fps()).as_str(),
                    vec2(self.window_context.active_screen_size.x - 5.0, 5.0),
                    TextParams {
                        font,
                        font_size: 40,
                        font_scale: 0.25,
                        color: WHITE,
                        ..Default::default()
                    },
                    vec2(1.0, 1.0),
                );
            }

            match state {
                MenuState::MainMenu => {
                    if is_key_pressed(KeyCode::Escape) {
                        return None;
                    }
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
                    ).clicked() || is_key_pressed(KeyCode::Space) {
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
                        song = match serde_json::from_str::<Song>(&match load_string(&format!("assets/songs/{}/{}", active_difficulty, song_database.songs[chosen_song_idx].json_name)).await {
                            Ok(tex) => tex,
                            Err(_) => return Some(Box::new(ErrorScene::new("Assets Missing (Verify Game Files or Reinstall)", self.window_context.clone())))
                        }) {
                            Ok(tex) => tex,
                            Err(_) => return Some(Box::new(ErrorScene::new("Song Format Incorrect", self.window_context.clone())))
                        };
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
                        song = match serde_json::from_str::<Song>(&match load_string(&format!("assets/songs/{}/{}", active_difficulty, song_database.songs[chosen_song_idx].json_name)).await {
                            Ok(tex) => tex,
                            Err(_) => return Some(Box::new(ErrorScene::new("Assets Missing (Verify Game Files or Reinstall)", self.window_context.clone())))
                        }) {
                            Ok(tex) => tex,
                            Err(_) => return Some(Box::new(ErrorScene::new("Song Format Incorrect", self.window_context.clone())))
                        };
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
                        song = match serde_json::from_str::<Song>(&match load_string(&format!("assets/songs/{}/{}", active_difficulty, song_database.songs[chosen_song_idx].json_name)).await {
                            Ok(tex) => tex,
                            Err(_) => return Some(Box::new(ErrorScene::new("Assets Missing (Verify Game Files or Reinstall)", self.window_context.clone())))
                        }) {
                            Ok(tex) => tex,
                            Err(_) => return Some(Box::new(ErrorScene::new("Song Format Incorrect", self.window_context.clone())))
                        };
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
                        song = match serde_json::from_str::<Song>(&match load_string(&format!("assets/songs/{}/{}", active_difficulty, song_database.songs[chosen_song_idx].json_name)).await {
                            Ok(tex) => tex,
                            Err(_) => return Some(Box::new(ErrorScene::new("Assets Missing (Verify Game Files or Reinstall)", self.window_context.clone())))
                        }) {
                            Ok(tex) => tex,
                            Err(_) => return Some(Box::new(ErrorScene::new("Song Format Incorrect", self.window_context.clone())))
                        };
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
                        song = match serde_json::from_str::<Song>(&match load_string(&format!("assets/songs/{}/{}", active_difficulty, song_database.songs[chosen_song_idx].json_name)).await {
                            Ok(tex) => tex,
                            Err(_) => return Some(Box::new(ErrorScene::new("Assets Missing (Verify Game Files or Reinstall)", self.window_context.clone())))
                        }) {
                            Ok(tex) => tex,
                            Err(_) => return Some(Box::new(ErrorScene::new("Song Format Incorrect", self.window_context.clone())))
                        };
                    }

                    let song_data_left = self.window_context.active_screen_size.x - 50.0 - self.window_context.active_screen_size.x * 0.56;
                    let song_data_center = self.window_context.active_screen_size.x - 50.0 - (self.window_context.active_screen_size.x * 0.56) / 2.0;

                    if !changing_song {
                        if is_key_pressed(KeyCode::Down) {
                            match active_difficulty {
                                Difficulty::Easy => {active_difficulty = Difficulty::Medium}
                                Difficulty::Medium => {active_difficulty = Difficulty::Hard}
                                Difficulty::Hard => {active_difficulty = Difficulty::Expert}
                                Difficulty::Expert => {active_difficulty = Difficulty::Extreme}
                                Difficulty::Extreme => {  }
                            }
                        }
                        if is_key_pressed(KeyCode::Up) {
                            match active_difficulty {
                                Difficulty::Easy => {  }
                                Difficulty::Medium => {active_difficulty = Difficulty::Easy}
                                Difficulty::Hard => {active_difficulty = Difficulty::Medium}
                                Difficulty::Expert => {active_difficulty = Difficulty::Hard}
                                Difficulty::Extreme => {active_difficulty = Difficulty::Expert}
                            }
                        }

                        if is_key_pressed(KeyCode::S) {
                            changing_song = true;
                        }
                        if is_key_pressed(KeyCode::Escape) {
                            state = MenuState::MainMenu;
                        }
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
                        if is_key_pressed(KeyCode::S) {
                            changing_song = false;
                        }

                        if is_key_pressed(KeyCode::Down) {
                            chosen_song_idx += 1;
                            if chosen_song_idx >= song_database.songs.len() {
                                chosen_song_idx = song_database.songs.len() - 1
                            }
                        }
                        if is_key_pressed(KeyCode::Up) && chosen_song_idx > 0 {
                            chosen_song_idx -= 1;
                        }

                        if is_key_pressed(KeyCode::Escape) {
                            changing_song = false;
                        }
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
                                    song = match serde_json::from_str::<Song>(&match load_string(&format!("assets/songs/{}/{}", active_difficulty, song_database.songs[chosen_song_idx].json_name)).await {
                                        Ok(tex) => tex,
                                        Err(_) => return Some(Box::new(ErrorScene::new("Assets Missing (Verify Game Files or Reinstall)", self.window_context.clone())))
                                    }) {
                                        Ok(tex) => tex,
                                        Err(_) => return Some(Box::new(ErrorScene::new("Song Format Incorrect", self.window_context.clone())))
                                    };
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
                    ).clicked() || is_key_pressed(KeyCode::Space) {
                        state = MenuState::Loading;
                        load_scene_timer.start();
                    }

                    // Watch Button
                    if element_text_template(
                        justify_rect(song_data_center - 100.0, self.window_context.active_screen_size.y - 20.0, 96.0 * 1.5, 26.0 * 1.25, vec2(0.5, 1.0)),
                        button_template,
                        mouse_pos,
                        "Watch",
                        TextParams {
                            font,
                            font_size: 60,
                            font_scale: 0.25,
                            ..Default::default()
                        }
                    ).clicked() || is_key_pressed(KeyCode::W) {
                        state = MenuState::Loading;
                        load_watch_timer.start();
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
                        if changing_song {
                            changing_song = false;
                        } else {
                            state = MenuState::MainMenu
                        }
                    }
                }
                MenuState::Settings => {
                    if is_key_pressed(KeyCode::Escape) {
                        state = MenuState::MainMenu;
                        checking_input = 0;
                    }

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
                        state = MenuState::MainMenu;
                        checking_input = 0;
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
                        justify_rect(self.window_context.active_screen_size.x - 50.0, 100.0, 96.0 * 2.0, 18.0 * 1.8, vec2(1.0, 0.5)),
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
                            vec2( self.window_context.active_screen_size.x - 50.0, 125.0),
                            TextParams {
                                font,
                                font_size: 28,
                                font_scale: 0.25,
                                ..Default::default()
                            }, vec2(1.0, 1.0)
                        );
                    }

                    nine_slice_frame.draw(justify_rect(self.window_context.active_screen_size.x - 250.0, 50.0, 200.0, 40.0, vec2(0.0, 0.5)), WHITE);

                    draw_text_justified("Scaling: ", vec2(self.window_context.active_screen_size.x - 240.0, 50.0), TextParams {
                        font,
                        font_size: 45,
                        font_scale: 0.25,
                        ..Default::default()
                    }, vec2(0.0, 0.5));

                    if element_template(justify_rect(self.window_context.active_screen_size.x - 250.0 + 100.0, 50.0, 18.0, 8.0, vec2(0.0, 0.5)), minus_template, mouse_pos).clicked() {
                        config.resolution_scale -= 1;
                        config.resolution_scale = config.resolution_scale.clamp(1, 16);

                        self.window_context.scale = config.resolution_scale;
                        self.window_context.dirty = true;

                        let mut data = File::create("assets/config.json").unwrap();
                        data.write_all((serde_json::to_string_pretty(&config).unwrap()).as_ref()).unwrap();
                        config = serde_json::from_str::<Config>(&load_string("assets/config.json").await.unwrap()).unwrap();
                    }

                    draw_text_justified(&format!("{}", config.resolution_scale), vec2(self.window_context.active_screen_size.x - 110.0, 50.0), TextParams {
                        font,
                        font_size: 45,
                        font_scale: 0.25,
                        ..Default::default()
                    }, vec2(0.5, 0.5));

                    if element_template(justify_rect(self.window_context.active_screen_size.x - 90.0, 50.0, 18.0, 18.0, vec2(0.0, 0.5)), plus_template, mouse_pos).clicked() {
                        config.resolution_scale += 1;
                        config.resolution_scale = config.resolution_scale.clamp(1, 16);

                        self.window_context.scale = config.resolution_scale;
                        self.window_context.dirty = true;

                        let mut data = File::create("assets/config.json").unwrap();
                        data.write_all((serde_json::to_string_pretty(&config).unwrap()).as_ref()).unwrap();
                        config = serde_json::from_str::<Config>(&load_string("assets/config.json").await.unwrap()).unwrap();
                    }

                    nine_slice_frame.draw(
                        justify_rect(45.0, 150.0,
                                     192.0, 175.0,
                                     vec2(0.0, 0.0)), WHITE);

                    draw_text_justified("Left Arrow: ",
                                        vec2(45.0 + 10.0, 175.0),
                                        TextParams {
                        font,
                        font_size: 40,
                        font_scale: 0.25,
                        ..Default::default()
                    },
                                        vec2(0.0, 0.5));
                    if element_text_template(
                        justify_rect(227.0, 175.0, 70.0, 20.0, vec2(1.0, 0.5)),
                        match checking_input {
                            1 => faint_button_template,
                            _ => button_template
                        },
                        mouse_pos,
                        &format!("{:?}", u32_to_key_code(config.controls.left_arrow)),
                        TextParams {
                            font,
                            font_size: 35,
                            font_scale: 0.25,
                            ..Default::default()
                        }
                    ).clicked() {
                        checking_input = 1
                    }

                    draw_text_justified("Up Arrow: ",
                                        vec2(45.0 + 10.0, 200.0),
                                        TextParams {
                                            font,
                                            font_size: 40,
                                            font_scale: 0.25,
                                            ..Default::default()
                                        },
                                        vec2(0.0, 0.5));
                    if element_text_template(
                        justify_rect(227.0, 200.0, 70.0, 20.0, vec2(1.0, 0.5)),
                        match checking_input {
                            2 => faint_button_template,
                            _ => button_template
                        },
                        mouse_pos,
                        &format!("{:?}", u32_to_key_code(config.controls.up_arrow)),
                        TextParams {
                            font,
                            font_size: 35,
                            font_scale: 0.25,
                            ..Default::default()
                        }
                    ).clicked() {
                        checking_input = 2
                    }

                    draw_text_justified("Right Arrow: ",
                                        vec2(45.0 + 10.0, 225.0),
                                        TextParams {
                                            font,
                                            font_size: 40,
                                            font_scale: 0.25,
                                            ..Default::default()
                                        },
                                        vec2(0.0, 0.5));
                    if element_text_template(
                        justify_rect(227.0, 225.0, 70.0, 20.0, vec2(1.0, 0.5)),
                        match checking_input {
                            3 => faint_button_template,
                            _ => button_template
                        },
                        mouse_pos,
                        &format!("{:?}", u32_to_key_code(config.controls.right_arrow)),
                        TextParams {
                            font,
                            font_size: 35,
                            font_scale: 0.25,
                            ..Default::default()
                        }
                    ).clicked() {
                        checking_input = 3
                    }

                    draw_text_justified("Down Arrow: ",
                                        vec2(45.0 + 10.0, 250.0),
                                        TextParams {
                                            font,
                                            font_size: 40,
                                            font_scale: 0.25,
                                            ..Default::default()
                                        },
                                        vec2(0.0, 0.5));
                    if element_text_template(
                        justify_rect(227.0, 250.0, 70.0, 20.0, vec2(1.0, 0.5)),
                        match checking_input {
                            4 => faint_button_template,
                            _ => button_template
                        },
                        mouse_pos,
                        &format!("{:?}", u32_to_key_code(config.controls.down_arrow)),
                        TextParams {
                            font,
                            font_size: 35,
                            font_scale: 0.25,
                            ..Default::default()
                        }
                    ).clicked() {
                        checking_input = 4
                    }

                    draw_text_justified("Ship Up: ",
                                        vec2(45.0 + 10.0, 275.0),
                                        TextParams {
                                            font,
                                            font_size: 40,
                                            font_scale: 0.25,
                                            ..Default::default()
                                        },
                                        vec2(0.0, 0.5));
                    if element_text_template(
                        justify_rect(227.0, 275.0, 70.0, 20.0, vec2(1.0, 0.5)),
                        match checking_input {
                            5 => faint_button_template,
                            _ => button_template
                        },
                        mouse_pos,
                        &format!("{:?}", u32_to_key_code(config.controls.ship_up)),
                        TextParams {
                            font,
                            font_size: 35,
                            font_scale: 0.25,
                            ..Default::default()
                        }
                    ).clicked() {
                        checking_input = 5
                    }

                    draw_text_justified("Ship Down: ",
                                        vec2(45.0 + 10.0, 300.0),
                                        TextParams {
                                            font,
                                            font_size: 40,
                                            font_scale: 0.25,
                                            ..Default::default()
                                        },
                                        vec2(0.0, 0.5));
                    if element_text_template(
                        justify_rect(227.0, 300.0, 70.0, 20.0, vec2(1.0, 0.5)),
                        match checking_input {
                            6 => faint_button_template,
                            _ => button_template
                        },
                        mouse_pos,
                        &format!("{:?}", u32_to_key_code(config.controls.ship_down)),
                        TextParams {
                            font,
                            font_size: 35,
                            font_scale: 0.25,
                            ..Default::default()
                        }
                    ).clicked() {
                        checking_input = 6
                    }


                    let mut save = false;
                    if checking_input != 0 {
                        if let Some(pressed_key) = get_last_key_pressed() {
                            let code = key_code_to_u32(pressed_key);
                            if code != 120 { // An Unknown Key
                                match checking_input {
                                    1 => { config.controls.left_arrow = code; }
                                    2 => { config.controls.up_arrow = code; }
                                    3 => { config.controls.right_arrow = code; }
                                    4 => { config.controls.down_arrow = code; }
                                    5 => { config.controls.ship_up = code; }
                                    6 => { config.controls.ship_down = code; }
                                    _ => {}
                                };
                                checking_input = 0;
                                save = true;
                            }
                        }
                    }

                    if save {
                        let mut data = File::create("assets/config.json").unwrap();
                        data.write_all((serde_json::to_string_pretty(&config).unwrap()).as_ref()).unwrap();
                        config = serde_json::from_str::<Config>(&load_string("assets/config.json").await.unwrap()).unwrap();
                    }
                }
                MenuState::Loading => {
                    let dots = if load_scene_timer.running {
                        (load_scene_timer.percent_done() * 5.0).round() as i32 % 4
                    } else {
                        (load_watch_timer.percent_done() * 5.0).round() as i32 % 4
                    };

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
            load_watch_timer.update();

            if load_scene_timer.running {
                music.set_volume(config.volume * (1.0 - load_scene_timer.percent_done()) as f64, Default::default()).unwrap();
            }

            if load_watch_timer.running {
                music.set_volume(config.volume * (1.0 - load_watch_timer.percent_done()) as f64, Default::default()).unwrap();
            }

            if load_scene_timer.is_done() {
                return Some(Box::new(NoteGameplayScene::new(
                    self.window_context.clone(),
                    format!("assets/songs/{}/{}", active_difficulty, song_database.songs[chosen_song_idx].json_name).as_str(),
                    ReturnTo::MainMenu(active_difficulty.clone(), chosen_song_idx)
                )));
            }

            if load_watch_timer.is_done() {
                return Some(Box::new(PorpusScene::new(
                    self.window_context.clone(),
                    format!("assets/songs/{}/{}", active_difficulty, song_database.songs[chosen_song_idx].json_name).as_str(),
                    ReturnTo::MainMenu(active_difficulty.clone(), chosen_song_idx)
                )));
            }

            if is_key_pressed(KeyCode::F12) {
                return Some(Box::new(BeatmapEditorScene { window_context: self.window_context.clone(), song_path: Default::default() }));
            }

            draw_window(&mut self.window_context);

            next_frame().await;
        }
    }
}