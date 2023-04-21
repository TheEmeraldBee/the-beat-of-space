#![windows_subsystem = "windows"]

// for nvidia cards
#[no_mangle]
pub static NvOptimusEnablement: i32 = 1;

// for amd cards
#[no_mangle]
pub static AmdPowerXpressRequestHighPerformance: i32 = 1;

use std::env;
use std::fs::{read_to_string};
use macroquad::miniquad::conf::Icon;
use macroquad::prelude::*;
use macroquad_aspect::prelude::*;
use crate::main_menu_scene::MainMenuScene;
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

mod error_scene;

mod tutorial_scene;

mod ui;
mod game_map_scene;

fn window_conf() -> Conf {
    // let mut current_exe = env::current_exe().unwrap();
    // current_exe.pop();
    // env::set_current_dir(&current_exe).unwrap();
    // println!("{:?}", env::current_dir().unwrap());

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

    let config = serde_json::from_str::<Config>(&read_to_string("assets/config.json").unwrap()).unwrap();

    let args: Vec<String> = env::args().collect();

    for i in 0..args.len() {
        let arg = args[i].clone();
        if arg.as_str() == "midi" {
            let bpm = args[i + 1].clone().parse::<f32>().unwrap();
            let midi_path = args[i + 2].clone();
            let song_path =  args[i + 3].clone();

            let midi_convert = midi_converter::MidiConverter {
                bpm,
                midi_path,
                song_path
            };
            midi_convert.load_midi().await;

            return;
        }
    }


    let mut window_context = WindowContext::new(vec![
        Aspect::new(708.0, 400.0)
    ]);
    window_context.forced = false;
    window_context.scale = config.resolution_scale;

    let mut scene: Box<dyn Scene> = Box::new(MainMenuScene {
        window_context
    });

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
