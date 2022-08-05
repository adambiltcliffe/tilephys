use hecs::World;
use loader::{load_map, LoadedMap};
use macroquad::prelude::*;
use physics::{Actor, ConstantMotion, IntRect, PathMotion, TileBody};

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

    let mut player_rect = IntRect::new(50, 10, 10, 10);
    let mut player = Actor::new(player_rect.x, player_rect.y);
    let mut player_vx = 0.0;
    let mut player_vy = 0.0;
    let mut player_jump_frames = 0;
    let mut player_grounded = false;

    loop {
        ConstantMotion::apply(&mut player, &mut player_rect, &mut world);
        PathMotion::apply(&mut player, &mut player_rect, &mut world);

        player_vy += 1.0;
        if is_key_down(KeyCode::Left) {
            player_vx -= 3.0;
        }
        if is_key_down(KeyCode::Right) {
            player_vx += 3.0;
        }
        player_vx *= 0.6;

        if player_grounded && is_key_pressed(KeyCode::X) {
            player_vy = -5.0;
            player_jump_frames = 5;
        } else if player_jump_frames > 0 && is_key_down(KeyCode::X) {
            player_vy = -5.0;
            player_jump_frames -= 1;
        } else {
            player_jump_frames = 0;
        }

        let (cx, cy) =
            physics::move_actor(&mut player, &mut player_rect, player_vx, player_vy, &world);
        if cx {
            player_vx = 0.0;
        }
        if cy {
            player_vy = 0.0;
        }

        player_grounded = physics::check_player_grounded(&player_rect, &world);

        draw(&mut world, &player_rect);
        next_frame().await
    }
}

fn draw(world: &mut World, player_rect: &IntRect) {
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

    draw_rectangle(
        player_rect.x as f32,
        player_rect.y as f32,
        player_rect.w as f32,
        player_rect.h as f32,
        GREEN,
    );
}
