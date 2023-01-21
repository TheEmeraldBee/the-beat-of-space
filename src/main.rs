#![windows_subsystem = "windows"]

use macroquad::miniquad::conf::Icon;
use macroquad::prelude::*;
use macroquad_aspect::prelude::*;
use crate::main_menu_scene::MainMenuScene;
use crate::scene::Scene;

mod utils;
mod scene;

mod note_gameplay_scene;
mod main_menu_scene;

mod ui;

fn window_conf() -> Conf {
    Conf {
        window_title: "The Beat Of Space".to_string(),
        window_width: 1920,
        window_height: 1080,
        window_resizable: true,
        fullscreen: true,
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
