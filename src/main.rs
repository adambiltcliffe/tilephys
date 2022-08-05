use hecs::World;
use loader::{load_map, LoadedMap};
use macroquad::prelude::*;
use physics::{Actor, ConstantMotion, Controller, IntRect, PathMotion, TileBody};

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
        mut world,
        body_ids,
        paths,
    } = load_map();

    world
        .insert_one(body_ids["layer1"], ConstantMotion::new(-1, 0))
        .unwrap();
    world
        .insert_one(body_ids["layer2"], ConstantMotion::new(1, 0))
        .unwrap();
    world
        .insert_one(body_ids["layer3"], ConstantMotion::new(0, -1))
        .unwrap();

    let pm = PathMotion::new(
        world.get::<&TileBody>(body_ids["layer4"]).unwrap().x as f32,
        world.get::<&TileBody>(body_ids["layer4"]).unwrap().y as f32,
        paths["orbit"].clone(),
        1.0,
    );
    world.insert_one(body_ids["layer4"], pm).unwrap();

    let pm = PathMotion::new(
        world.get::<&TileBody>(body_ids["cross"]).unwrap().x as f32,
        world.get::<&TileBody>(body_ids["cross"]).unwrap().y as f32,
        paths["diamondpath"].clone(),
        4.0,
    );
    world.insert_one(body_ids["cross"], pm).unwrap();

    let player_rect = IntRect::new(50, 10, 10, 10);
    let player = Actor::new(&player_rect);
    let controller = Controller::new();
    world.spawn((player_rect, player, controller));

    loop {
        ConstantMotion::apply(&mut world);
        PathMotion::apply(&mut world);

        Controller::update(&mut world);
        Actor::update(&mut world);

        draw(&mut world);
        next_frame().await
    }
}

fn draw(world: &mut World) {
    // we don't actually need mutable access to the world but having it lets us tell
    // hecs we can skip dynamic borrow checking by using query_mut
    clear_background(SKYBLUE);

    let _delta = get_frame_time();
    let (mx, my) = mouse_position();
    let mouse_rect = IntRect::new(mx as i32 - 5, my as i32 - 5, 10, 10);

    for (_, chunk) in world.query_mut::<&TileBody>() {
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

    for (_, (_, rect)) in world.query_mut::<(&Actor, &IntRect)>() {
        draw_rectangle(
            rect.x as f32,
            rect.y as f32,
            rect.w as f32,
            rect.h as f32,
            GREEN,
        );
    }
}
