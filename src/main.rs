use camera::{add_camera, PlayerCamera};
use draw::PlayerSprite;
use enemy::Enemy;
use hecs::CommandBuffer;
use input::Input;
use loader::{LoadedMap, LoadingManager};
use macroquad::prelude::*;
use physics::{Actor, ConstantMotion, IntRect, PathMotion, Projectile};
use player::Controller;
use render::Renderer;
use resources::Resources;
use script::ScriptEngine;
use std::rc::Rc;
use timer::Timer;
use visibility::compute_obscurers;

mod camera;
mod draw;
mod enemy;
mod input;
mod loader;
mod physics;
mod player;
mod render;
mod resources;
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
    let map = loader.load("intro.tmx").await.unwrap();

    let mut script_engine = ScriptEngine::new(&map);
    script_engine.load_file("intro.rhai").await;
    script_engine.call_entry_point("init");

    let LoadedMap {
        player_start,
        secret_count,
        ..
    } = map;
    let world_ref = Rc::clone(&map.world_ref);

    println!("map has {} secret areas", secret_count);

    let (player_id, mut eye, mut cam) = {
        let mut world = world_ref.borrow_mut();

        let player_rect = IntRect::new(player_start.0 - 8, player_start.1 - 24, 14, 24);
        let player_eye = player_rect.centre();
        let camera_pos = add_camera(&mut world, player_rect.centre());
        let player = Actor::new(&player_rect, 0.6);
        let controller = Controller::new();
        let sprite = PlayerSprite::new();
        let player_id = world.spawn((player_rect, player, controller, sprite));

        (player_id, player_eye, camera_pos)
    };

    compute_obscurers(&mut world_ref.borrow_mut());

    let renderer = Renderer::new(RENDER_W, RENDER_H);
    let mut clock = Timer::new();
    let mut input = Input::new();

    let resources = Resources::new(&map).await;

    loop {
        input.update();

        for _ in 0..clock.get_num_updates() {
            let mut world = world_ref.borrow_mut();
            let mut buffer = CommandBuffer::new();
            ConstantMotion::apply(&world);
            PathMotion::apply(&world);
            let (new_triggers, new_secrets) = Controller::update(&world, &mut buffer, &input);
            Enemy::update(&world, player_id);
            Actor::update(&world);
            Projectile::update(&world, &mut buffer);
            buffer.run_on(&mut world);

            if let Ok(rect) = world.get::<&IntRect>(player_id) {
                let player_pos = rect.centre();
                *eye = *player_pos;
            }

            if let Some(camera_pos) = PlayerCamera::update_and_get(&world) {
                *cam = *camera_pos;
            }

            drop(world);

            if new_secrets > 0 {
                println!("found a secret area!");
            }

            for t in new_triggers {
                println!("entered new trigger zone {}", t);
                script_engine.call_entry_point(&format!("{}_enter", t));
            }

            input.reset();
        }

        // this interface is getting busy
        renderer.draw(
            &mut world_ref.borrow_mut(),
            eye,
            cam,
            &map.tileset_info,
            &resources,
            clock.get_fps(),
        );

        next_frame().await;
    }
}
