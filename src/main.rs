use hecs::{Satisfies, World};
use loader::{load_map, LoadedMap};
use macroquad::prelude::*;
use physics::{Actor, ConstantMotion, Controller, IntRect, PathMotion, TileBody};
use rhai::{Engine, Scope};
use std::cell::RefCell;
use std::rc::Rc;

mod loader;
mod physics;

const SCR_W: i32 = 400;
const SCR_H: i32 = 400;

fn window_conf() -> Conf {
    Conf {
        window_title: "Platform tile physics test".to_owned(),
        fullscreen: false,
        window_width: SCR_W,
        window_height: SCR_H,
        ..Default::default()
    }
}

#[macroquad::main(window_conf())]
async fn main() {
    /*set_camera(&Camera2D {
        zoom: (vec2(1.0, 1.0)),
        target: vec2(SCR_W / 2., SCR_H / 2.),
        ..Default::default()
    });*/

    let LoadedMap {
        world,
        body_ids,
        paths,
    } = load_map();

    let world_ref = Rc::new(RefCell::new(world));
    let body_ids_ref = Rc::new(body_ids);

    let mut engine = Engine::new();
    let mut scope = Scope::new();

    let cloned_world = Rc::clone(&world_ref);
    let cloned_body_ids = Rc::clone(&body_ids_ref);
    engine.register_fn(
        "set_constant_motion",
        move |name: &str, vx: i32, vy: i32| {
            cloned_world
                .borrow_mut()
                .insert_one(cloned_body_ids[name], ConstantMotion::new(vx, vy))
                .unwrap();
        },
    );

    let cloned_world = Rc::clone(&world_ref);
    let cloned_body_ids = Rc::clone(&body_ids_ref);
    engine.register_fn(
        "set_path_motion",
        move |body_name: &str, path_name: &str, speed: f32| {
            let id = cloned_body_ids[body_name];
            let mut world = cloned_world.borrow_mut();
            let (x, y) = {
                let body = world.get::<&TileBody>(id).unwrap();
                (body.x as f32, body.y as f32)
            };
            world
                .insert_one(id, PathMotion::new(x, y, paths[path_name].clone(), speed))
                .unwrap();
        },
    );

    let ast = engine.compile_file("testmap.rhai".into()).unwrap();
    engine.call_fn::<()>(&mut scope, &ast, "init", ()).unwrap();

    let mut world = world_ref.borrow_mut();

    let player_rect = IntRect::new(50, 10, 10, 10);
    let player = Actor::new(&player_rect);
    let controller = Controller::new();
    world.spawn((player_rect, player, controller));

    let thing_rect = IntRect::new(200, 10, 6, 6);
    let thing = Actor::new(&thing_rect);
    world.spawn((thing_rect, thing));

    loop {
        ConstantMotion::apply(&world);
        PathMotion::apply(&world);

        Controller::update(&world);
        Actor::update(&world);

        draw(&world);
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
