use camera::PlayerCamera;
use input::Input;
use loader::{LoadedMap, LoadingManager};
use macroquad::prelude::*;
use physics::{Actor, ConstantMotion, Controller, IntRect, PathMotion};
use render::Renderer;
use script::ScriptEngine;
use timer::Timer;
use visibility::compute_obscurers;

mod camera;
mod draw;
mod input;
mod loader;
mod physics;
mod render;
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
    let mut loader = LoadingManager::new();
    let map = loader.load("secondmap.tmx").await.unwrap();

    let mut script_engine = ScriptEngine::new(&map);
    script_engine.load_file("secondmap.rhai").await;
    script_engine.call_entry_point("init");

    let LoadedMap { world_ref, .. } = map;
    let (player_id, mut eye, mut cam) = {
        let mut world = world_ref.borrow_mut();

        let player_rect = IntRect::new(50, 30, 10, 10);
        let player_eye = player_rect.centre();
        let player = Actor::new(&player_rect);
        let camera_pos = player_rect.centre();
        let controller = Controller::new();
        let player_id = world.spawn((player_rect, player, controller));

        world.spawn((PlayerCamera::new(camera_pos.y), camera_pos.clone()));

        let thing_rect = IntRect::new(200, 10, 6, 6);
        let thing = Actor::new(&thing_rect);
        world.spawn((thing_rect, thing));

        (player_id, player_eye, camera_pos)
    };

    compute_obscurers(&mut world_ref.borrow_mut());

    let renderer = Renderer::new(RENDER_W, RENDER_H);
    let mut clock = Timer::new();
    let mut input = Input::new();

    loop {
        input.update();

        for _ in 0..clock.get_num_updates() {
            let world = world_ref.borrow_mut();
            ConstantMotion::apply(&world);
            PathMotion::apply(&world);
            let new_triggers = Controller::update(&world, &input);
            Actor::update(&world);

            if let Ok(rect) = world.get::<&IntRect>(player_id) {
                let player_pos = rect.centre();
                *eye = *player_pos;
            }

            if let Some(camera_pos) = PlayerCamera::update_and_get(&world) {
                *cam = *camera_pos;
            }

            drop(world);

            for t in new_triggers {
                println!("entered new trigger zone {}", t);
                script_engine.call_entry_point(&format!("{}_enter", t));
            }

            input.reset();
        }

        renderer.draw(&mut world_ref.borrow_mut(), eye, cam, &map.tileset_info);

        next_frame().await;
    }
}
