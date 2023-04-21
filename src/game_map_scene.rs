use std::fs;
use async_trait::async_trait;
use macroquad::camera::set_camera;
use macroquad::prelude::*;
use macroquad::window::{clear_background, next_frame};
use macroquad_aspect::prelude::WindowContext;
use macroquad_aspect::window::draw_window;
use crate::error_scene::ErrorScene;
use crate::game_map_scene::world::World;
use crate::note_gameplay_scene::song::Song;
use crate::scene::Scene;

pub mod world;

pub struct GameMapScene {
    window_context: WindowContext,
    selected_world: String,
}

impl GameMapScene {
    pub fn new(window_context: WindowContext, selected_world: Option<String>) -> Self {
        Self {
            window_context,
            selected_world: match selected_world {
                Some(world) => world,
                None => "".to_string()
            },
        }
    }
}

#[async_trait]
impl Scene for GameMapScene {
    async fn run(&mut self) -> Option<Box<dyn Scene>> {
        let mut worlds = vec![];

        let mut reads = vec![];

        let files = fs::read_dir("worlds").unwrap();
        for file in files {
            reads.push(file.unwrap());
        }

        while let Some(dir) = reads.pop() {
            if dir.file_type().unwrap().is_dir() {
                println!("Found World!");

                let path = format!("{}", dir.path().as_os_str().to_str().unwrap());

                let world_json = match load_string(&format!("{}/world.json", &path)).await {
                    Ok(str) => str,
                    Err(_) => return Some(Box::new(
                        ErrorScene::new(
                            &format!("Error: World {:?} is not formatted correctly", dir.file_name()),
                            self.window_context.clone()))
                    )
                };

                let mut world = match serde_json::from_str::<World>(&world_json) {
                    Ok(world) => world,
                    Err(_) => return Some(Box::new(
                        ErrorScene::new(
                            &format!("Error: World {:?} is not formatted correctly", dir.file_name()),
                            self.window_context.clone()))
                    )
                };

                match verify_world(&mut world, &path).await {
                    Ok(()) => (),
                    Err(err) => return Some(Box::new(ErrorScene::new(&err, self.window_context.clone())))
                }

                worlds.push(world)
            }
        }

        loop {
            set_camera(&self.window_context.camera);
            clear_background(BLACK);

            draw_window(&mut self.window_context);

            next_frame().await;
        }
    }
}

pub async fn verify_world(world: &mut World, path: &str) -> Result<(), String> {
    for mut level in &mut world.levels {
        let level_json = match load_string(&format!("{}/{}.json", path, level)).await {
            Ok(s) => s,
            Err(_) => return Err(format!("{} level doesn't exist", level))
        };

        match serde_json::from_str::<Song>(&level_json) {
            Ok(_) => (),
            Err(_) => return Err(format!("{} level is formatted incorrectly", level))
        };

        level = &mut format!("{}/{}.json", path, level);
    }


    Ok(())
}