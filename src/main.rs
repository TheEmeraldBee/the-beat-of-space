#![windows_subsystem = "windows"]

use std::fs::{read_to_string};
use macroquad::miniquad::conf::Icon;
use macroquad::prelude::*;
use macroquad_aspect::prelude::*;
use crate::main_menu_scene::MainMenuScene;
use crate::porpus_scene::PorpusScene;
use crate::scene::Scene;
use crate::utils::Config;

mod utils;
mod scene;

mod note_gameplay_scene;
mod porpus_scene;

mod main_menu_scene;
mod game_end_scene;

mod beatmap_editor_scene;
mod midi_converter;

mod ui;

fn window_conf() -> Conf {
    let config = serde_json::from_str::<Config>(&read_to_string("assets/config.json").unwrap()).unwrap();

    Conf {
        window_title: "The Beat Of Space".to_string(),
        window_width: 1920,
        window_height: 1080,
        window_resizable: true,
        fullscreen: config.fullscreen,
        icon: Some(Icon {
            small: <[u8; 1024]>::try_from(image::open("assets/images/icon_small.png").unwrap().as_bytes()).unwrap(),
            medium: <[u8; 4096]>::try_from(image::open("assets/images/icon_med.png").unwrap().as_bytes()).unwrap(),
            big: <[u8; 16384]>::try_from(image::open("assets/images/icon_large.png").unwrap().as_bytes()).unwrap()
        }),
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let window_context = WindowContext::new(vec![
        Aspect::new(708.0, 400.0)
    ]);

    // let mut scene: Box<dyn Scene> = Box::new(MainMenuScene {
    //     window_context
    // });

    let midi_convert = midi_converter::MidiConverter {
        song_path: "".to_string()
    };
    midi_convert.load_midi().await;

    let mut scene: Box<dyn Scene> = Box::new(PorpusScene::new(window_context, "assets/songs/extreme/goldn.json"));

    loop {

        let next_scene = scene.run().await;

        if let Some(next_scene) = next_scene {
            scene = next_scene;
        }
        else {
            break;
        }

        next_frame().await;
    }

}
