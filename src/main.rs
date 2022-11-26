use camera::PlayerCamera;
use enemy::Enemy;
use hecs::{CommandBuffer, World};
use input::Input;
use loader::LoadingManager;
use macroquad::prelude::*;
use physics::{Actor, ConstantMotion, PathMotion, Projectile};
use pickup::Pickup;
use player::Controller;
use render::Renderer;
use resources::Resources;
use scene::SceneTransition;
use std::cell::RefCell;
use std::rc::Rc;
use timer::Timer;

mod camera;
mod draw;
mod enemy;
mod input;
mod loader;
mod messages;
mod physics;
mod pickup;
mod player;
mod render;
mod resources;
mod scene;
mod script;
mod timer;
mod visibility;

const RENDER_W: u32 = 320;
const RENDER_H: u32 = 200;

fn window_conf() -> Conf {
    Conf {
        window_title: "Platform tile physics test".to_owned(),
        fullscreen: false,
        window_width: RENDER_W as i32 * 2,
        window_height: RENDER_H as i32 * 2,
        ..Default::default()
    }
}

#[macroquad::main(window_conf())]
async fn main() {
    set_pc_assets_folder("assets");
    let mut loader = LoadingManager::new();
    let (mut world_ref, mut resources): (Rc<RefCell<World>>, Resources) =
        loader.load_level("intro.tmx").await.unwrap();

    let renderer = Renderer::new(RENDER_W, RENDER_H);
    let mut clock = Timer::new();
    let mut input = Input::new();

    loop {
        if resources.transition == SceneTransition::Restart {
            (world_ref, resources) = loader.load_level("intro.tmx").await.unwrap();
            clock = Timer::new();
            input = Input::new();
        }

        input.update();

        for _ in 0..clock.get_num_updates() {
            let mut world = world_ref.borrow_mut();
            let mut buffer = CommandBuffer::new();
            ConstantMotion::apply(&world);
            PathMotion::apply(&world);
            let (new_triggers, new_secrets) =
                Controller::update(&world, &mut resources, &mut buffer, &input);
            Enemy::update(&world, &resources);
            Actor::update(&world);
            Projectile::update(&world, &mut resources, &mut buffer);
            Pickup::update(&world, &mut resources, &mut buffer);
            buffer.run_on(&mut world);

            PlayerCamera::update(&world, &mut resources);

            drop(world);

            if new_secrets > 0 {
                resources.messages.add("Found a secret area!".to_owned());
            }

            for t in new_triggers {
                println!("entered new trigger zone {}", t);
                resources
                    .script_engine
                    .call_entry_point(&format!("{}_enter", t));
            }

            input.reset();
            resources.messages.update();
        }

        renderer.draw(&mut world_ref.borrow_mut(), &resources, clock.get_fps());
        next_frame().await;
    }
}
