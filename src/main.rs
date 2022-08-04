use glam::vec2;
use hecs::{Entity, World};
use macroquad::prelude::*;
use std::collections::HashMap;

const SCR_W: i32 = 400;
const SCR_H: i32 = 400;

struct TileBody {
    width: i32,
    size: i32,
    data: Vec<bool>,
    x: i32,
    y: i32,
}

impl TileBody {
    fn new(x: i32, y: i32, size: i32, width: i32, data: Vec<bool>) -> Self {
        Self {
            x,
            y,
            size,
            width,
            data,
        }
    }

    // signature of this function should change when we have a proper rectangle struct
    fn collide(&self, x: i32, y: i32, w: i32, h: i32) -> bool {
        let min_kx = (x - self.x).div_euclid(self.size);
        let max_kx = (x + w - 1 - self.x).div_euclid(self.size);
        let min_ky = (y - self.y).div_euclid(self.size);
        let max_ky = (y + h - 1 - self.y).div_euclid(self.size);
        for ky in min_ky..=max_ky {
            if ky >= 0 {
                for kx in min_kx..=max_kx {
                    if kx >= 0 && kx < self.width {
                        let index = ky * self.width + kx;
                        if index >= 0 && index < self.data.len() as i32 && self.data[index as usize]
                        {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

struct Actor {
    x: i32,
    y: i32,
    prec_x: f32,
    prec_y: f32,
    width: i32,
    height: i32,
}

impl Actor {
    fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            prec_x: x as f32,
            prec_y: y as f32,
        }
    }
}

struct ConstantMotion {
    vx: i32,
    vy: i32,
}

fn move_actor(actor: &mut Actor, vx: f32, vy: f32, world: &World) -> (bool, bool) {
    actor.prec_x += vx;
    let targ_x = actor.prec_x.round() as i32;
    let mut collided_x = false;
    while actor.x != targ_x {
        let dx = (targ_x - actor.x).signum();
        if world
            .query::<&TileBody>()
            .iter()
            .any(|(_, c)| c.collide(actor.x + dx, actor.y, actor.width, actor.height))
        {
            actor.prec_x = actor.x as f32;
            collided_x = true;
            break;
        } else {
            actor.x += dx;
        }
    }
    actor.prec_y += vy;
    let targ_y = actor.prec_y.round() as i32;
    let mut collided_y = false;
    while actor.y != targ_y {
        let dy = (targ_y - actor.y).signum();
        if world
            .query::<&TileBody>()
            .iter()
            .any(|(_, c)| c.collide(actor.x, actor.y + dy, actor.width, actor.height))
        {
            actor.prec_y = actor.y as f32;
            collided_y = true;
            break;
        } else {
            actor.y += dy
        }
    }
    (collided_x, collided_y)
}

// this is an awful way of doing this but let's get it working for now
fn move_body(actor: &mut Actor, world: &mut World, index: Entity, vx: i32, vy: i32) {
    for _ii in 0..(vx.abs()) {
        let mut body = world.get::<&mut TileBody>(index).unwrap();
        body.x += vx.signum();
        let should_move = body.collide(actor.x, actor.y, actor.width, actor.height + 1);
        drop(body);
        if should_move {
            move_actor(actor, vx.signum() as f32, 0.0, &world);
        }
    }
    for _ii in 0..(vy.abs()) {
        let mut body = world.get::<&mut TileBody>(index).unwrap();
        body.y += vy.signum();
        let should_move = body.collide(actor.x, actor.y, actor.width, actor.height + 1);
        drop(body);
        if should_move {
            move_actor(actor, 0.0, vy.signum() as f32, &world);
        }
    }
}

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

    let mut world: World = World::new();
    let mut chunk_ids: Vec<Entity> = Vec::new();
    let mut paths: HashMap<String, Vec<(f32, f32)>> = HashMap::new();

    let mut loader = tiled::Loader::new();
    let map = loader.load_tmx_map("testmap.tmx").unwrap();
    for layer in map.layers() {
        match layer.layer_type() {
            tiled::LayerType::TileLayer(tiled::TileLayer::Infinite(data)) => {
                println!("Found an infinite tiled layer named {}", layer.name);
                let (xmin, xmax, ymin, ymax) = data.chunks().fold(
                    (i32::MAX, i32::MIN, i32::MAX, i32::MIN),
                    |(x0, x1, y0, y1), ((x, y), _)| (x0.min(x), x1.max(x), y0.min(y), y1.max(y)),
                );
                const W: i32 = tiled::Chunk::WIDTH as i32;
                const H: i32 = tiled::Chunk::HEIGHT as i32;
                let (mut x0, mut x1, mut y0, mut y1) = (i32::MAX, i32::MIN, i32::MAX, i32::MIN);
                for y in ymin * H..(ymax + 1) * H {
                    for x in xmin * W..(xmax + 1) * W {
                        if data.get_tile(x, y).is_some() {
                            x0 = x0.min(x);
                            x1 = x1.max(x);
                            y0 = y0.min(y);
                            y1 = y1.max(y);
                        }
                    }
                }
                println!("Real chunk bounds are x:{}-{}, y:{}-{}", x0, x1, y0, y1);
                let mut tiledata = Vec::new();
                for y in y0..=y1 {
                    for x in x0..=x1 {
                        tiledata.push(data.get_tile(x, y).is_some());
                    }
                }
                chunk_ids.push(world.spawn((TileBody::new(
                    x0 * map.tile_width as i32,
                    y0 * map.tile_height as i32,
                    map.tile_width as i32,
                    (x1 - x0) + 1,
                    tiledata,
                ),)))
            }
            tiled::LayerType::ObjectLayer(data) => {
                for obj in data.objects() {
                    if let tiled::ObjectData {
                        name,
                        shape: tiled::ObjectShape::Polyline { points },
                        ..
                    } = &*obj
                    {
                        paths.insert(name.clone(), points.clone());
                    }
                }
            }
            _ => println!("(Something other than an infinite tiled layer)"),
        }
    }

    world
        .insert_one(chunk_ids[1], ConstantMotion { vx: -1, vy: 0 })
        .unwrap();
    world
        .insert_one(chunk_ids[2], ConstantMotion { vx: 1, vy: 0 })
        .unwrap();
    world
        .insert_one(chunk_ids[3], ConstantMotion { vx: 0, vy: -1 })
        .unwrap();

    let mut player = Actor::new(50, 10, 10, 10);
    let mut player_vx = 0.0;
    let mut player_vy = 0.0;
    let mut player_jump_frames = 0;
    let mut player_grounded = false;

    let mut chunk4_prec_x = world.get::<&TileBody>(chunk_ids[4]).unwrap().x as f32;
    let mut chunk4_prec_y = world.get::<&TileBody>(chunk_ids[4]).unwrap().y as f32;
    let mut chunk4_next_node = 0;
    let chunk4_base_vec = glam::vec2(chunk4_prec_x, chunk4_prec_y);

    loop {
        // TODO make this use those nice ConstantMotion components we added
        move_body(&mut player, &mut world, chunk_ids[1], -1, 0);
        move_body(&mut player, &mut world, chunk_ids[2], 1, 0);
        move_body(&mut player, &mut world, chunk_ids[3], 0, -1);

        // this is fiddly
        let dest_tuple = paths["orbit"][chunk4_next_node];
        let dest = vec2(dest_tuple.0, dest_tuple.1) + chunk4_base_vec;
        let curr = vec2(chunk4_prec_x, chunk4_prec_y);
        let v = dest - curr;
        let tmp = if v.length() < 1.0 {
            chunk4_next_node = (chunk4_next_node + 1) % 4;
            (dest.x, dest.y)
        } else {
            let new = curr + v.normalize();
            (new.x, new.y)
        };
        chunk4_prec_x = tmp.0;
        chunk4_prec_y = tmp.1;

        let (dx, dy) = {
            let chunk = world.get::<&TileBody>(chunk_ids[4]).unwrap();
            (
                chunk4_prec_x.round() as i32 - chunk.x,
                chunk4_prec_y.round() as i32 - chunk.y,
            )
        };
        move_body(&mut player, &mut world, chunk_ids[4], dx, dy);

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

        let (cx, cy) = move_actor(&mut player, player_vx, player_vy, &world);
        if cx {
            player_vx = 0.0;
        }
        if cy {
            player_vy = 0.0;
        }

        player_grounded = world
            .query::<&TileBody>()
            .iter()
            .any(|(_, c)| c.collide(player.x, player.y + player.height, player.width, 1));

        draw(&mut world, &player);
        next_frame().await
    }
}

fn draw(world: &mut World, player: &Actor) {
    // we don't actually need mutable access to the world but having it lets us tell
    // hecs we can skip dynamic borrow checking by using query_mut
    clear_background(SKYBLUE);

    let _delta = get_frame_time();
    let (mx, my) = mouse_position();

    for (_, chunk) in world.query_mut::<&TileBody>() {
        let mut tx = chunk.x;
        let mut ty = chunk.y;
        for ii in 0..(chunk.data.len()) {
            if chunk.data[ii] {
                let c = if chunk.collide(mx as i32 - 5, my as i32 - 5, 10, 10) {
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
        player.x as f32,
        player.y as f32,
        player.width as f32,
        player.height as f32,
        GREEN,
    );
}
