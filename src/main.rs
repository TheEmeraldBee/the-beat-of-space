use macroquad::prelude::*;
use macroquad_aspect::aspect::Aspect;
use macroquad_aspect::window::WindowContext;
use crate::note_gameplay_scene::NoteGameplayScene;
use crate::scene::Scene;

mod utils;
mod scene;

mod note_gameplay_scene;

fn window_conf() -> Conf {
    Conf {
        window_title: "The Beat Of Space".to_string(),
        window_width: 1920,
        window_height: 1080,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let window_context = WindowContext::new(vec![
        Aspect::new(708.0, 400.0)
    ]);

    let mut scene: Box<dyn Scene> = Box::new(NoteGameplayScene::new(window_context, "assets/songs/easy/goldn.json"));

    loop {
        let next_scene = scene.run().await;

        if let Some(next_scene) = next_scene {
            scene = next_scene;
        }
        else {
            break;
        }
    }

}
