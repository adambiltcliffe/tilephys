use hecs::{Satisfies, World};
use loader::{load_map, LoadedMap};
use macroquad::prelude::*;
use physics::{Actor, ConstantMotion, Controller, IntRect, PathMotion, TileBody};
use script::ScriptEngine;
use visibility::{compute_obscurers, draw_visibility};

mod loader;
mod physics;
mod script;
mod visibility;

const RENDER_W: i32 = 400;
const RENDER_H: i32 = 400;

fn window_conf() -> Conf {
    Conf {
        window_title: "Platform tile physics test".to_owned(),
        fullscreen: false,
        window_width: RENDER_W,
        window_height: RENDER_H,
        ..Default::default()
    }
}

#[macroquad::main(window_conf())]
async fn main() {
    let map = load_map();

    let mut script_engine = ScriptEngine::new(&map);
    script_engine.load_file("testmap.rhai");
    script_engine.call_entry_point("init");

    let LoadedMap { world_ref, .. } = map;
    let (player_id, mut eye) = {
        let mut world = world_ref.borrow_mut();

        let player_rect = IntRect::new(50, 10, 10, 10);
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

    set_camera(&Camera2D {
        zoom: (vec2(2. / RENDER_W as f32, -2. / RENDER_H as f32)),
        target: vec2(RENDER_W as f32 / 2., RENDER_H as f32 / 2.),
        ..Default::default()
    });

    loop {
        let world = world_ref.borrow_mut();
        ConstantMotion::apply(&world);
        PathMotion::apply(&world);
        let new_triggers = Controller::update(&world);
        Actor::update(&world);

        if let Ok(rect) = world.get::<&IntRect>(player_id) {
            *eye = *rect.centre();
            /* set_camera(&Camera2D {
                zoom: (vec2(0.02, -0.02)),
                target: vec2((rect.x + rect.w / 2) as f32, (rect.y + rect.h / 2) as f32),
                ..Default::default()
            }); */
        }

        draw(&world);
        let r = eye
            .x
            .max(RENDER_W as f32 - eye.x)
            .max(eye.y)
            .max(RENDER_H as f32 - eye.y)
            + 1.;
        draw_visibility(&world, eye, r);
        drop(world);

        for t in new_triggers {
            println!("entered new trigger zone {}", t);
            script_engine.call_entry_point(&format!("{}_enter", t));
        }

        next_frame().await
    }
}

fn draw(world: &World) {
    // we don't actually need mutable access to the world but having it lets us tell
    // hecs we can skip dynamic borrow checking by using query_mut
    clear_background(SKYBLUE);

    let _delta = get_frame_time();
    let (mx, my) = mouse_position();
    let mouse_rect = IntRect::new(mx as i32 - 5, my as i32 - 5, 10, 10);

    for (_, chunk) in world.query::<&TileBody>().iter() {
        let mut tx = chunk.x;
        let mut ty = chunk.y;
        for ii in 0..(chunk.data.len()) {
            if chunk.data[ii] {
                let c = if chunk.collide(&mouse_rect) {
                    RED
                } else {
                    BLUE
                };
                draw_rectangle(
                    tx as f32,
                    ty as f32,
                    chunk.size as f32,
                    chunk.size as f32,
                    c,
                );
            }
            tx += chunk.size as i32;
            if ((ii + 1) % chunk.width as usize) == 0 {
                tx = chunk.x;
                ty += chunk.size as i32;
            }
        }
    }

    draw_rectangle(mx - 5., my - 5., 10., 10., ORANGE);

    for (_, (_, rect, ctrl)) in world
        .query::<(&Actor, &IntRect, Satisfies<&Controller>)>()
        .iter()
    {
        draw_rectangle(
            rect.x as f32,
            rect.y as f32,
            rect.w as f32,
            rect.h as f32,
            if ctrl { GREEN } else { GRAY },
        );
    }
}
