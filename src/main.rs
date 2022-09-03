use loader::{load_map, LoadedMap};
use macroquad::prelude::*;
use physics::{Actor, ConstantMotion, Controller, IntRect, PathMotion};
use render::Renderer;
use script::ScriptEngine;
use visibility::compute_obscurers;

mod draw;
mod loader;
mod physics;
mod render;
mod script;
mod visibility;

const RENDER_W: u32 = 400;
const RENDER_H: u32 = 400;

fn window_conf() -> Conf {
    Conf {
        window_title: "Platform tile physics test".to_owned(),
        fullscreen: false,
        window_width: RENDER_W as i32,
        window_height: RENDER_H as i32,
        ..Default::default()
    }
}

#[macroquad::main(window_conf())]
async fn main() {
    let map = load_map("secondmap.tmx").await.unwrap();

    let mut script_engine = ScriptEngine::new(&map);
    script_engine.load_file("secondmap.rhai");
    script_engine.call_entry_point("init");

    let LoadedMap { world_ref, .. } = map;
    let (player_id, mut eye) = {
        let mut world = world_ref.borrow_mut();

        let player_rect = IntRect::new(50, 30, 10, 10);
        let player_eye = player_rect.centre();
        let player = Actor::new(&player_rect);
        let controller = Controller::new();
        let player_id = world.spawn((player_rect, player, controller));

        let thing_rect = IntRect::new(200, 10, 6, 6);
        let thing = Actor::new(&thing_rect);
        world.spawn((thing_rect, thing));

        (player_id, player_eye)
    };

    compute_obscurers(&mut world_ref.borrow_mut());

    let renderer = Renderer::new(RENDER_W, RENDER_H);

    let camera_pos = vec2(140., 220.);

    loop {
        let mut world = world_ref.borrow_mut();
        ConstantMotion::apply(&world);
        PathMotion::apply(&world);
        let new_triggers = Controller::update(&world);
        Actor::update(&world);

        if let Ok(rect) = world.get::<&IntRect>(player_id) {
            *eye = *rect.centre();
        }

        renderer.draw(&mut world, eye, camera_pos, &map.tileset_info);
        drop(world);

        for t in new_triggers {
            println!("entered new trigger zone {}", t);
            script_engine.call_entry_point(&format!("{}_enter", t));
        }

        next_frame().await;
    }
}
