use glam::vec2;
use hecs::{Entity, World};
use macroquad::prelude::*;
use std::collections::HashMap;

const SCR_W: i32 = 400;
const SCR_H: i32 = 400;

struct IntRect {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl IntRect {
    fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }
}

fn offset_rect(rect: &IntRect, dx: i32, dy: i32) -> IntRect {
    IntRect::new(rect.x + dx, rect.y + dy, rect.w, rect.h)
}

fn pushing_rect(rect: &IntRect) -> IntRect {
    IntRect::new(rect.x, rect.y, rect.w, rect.h + 1)
}

fn feet_rect(rect: &IntRect) -> IntRect {
    IntRect::new(rect.x, rect.y + rect.h, rect.w, 1)
}

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

    fn collide(&self, rect: &IntRect) -> bool {
        let min_kx = (rect.x - self.x).div_euclid(self.size);
        let max_kx = (rect.x + rect.w - 1 - self.x).div_euclid(self.size);
        let min_ky = (rect.y - self.y).div_euclid(self.size);
        let max_ky = (rect.y + rect.h - 1 - self.y).div_euclid(self.size);
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
    prec_x: f32,
    prec_y: f32,
}

impl Actor {
    fn new(x: i32, y: i32) -> Self {
        Self {
            prec_x: x as f32,
            prec_y: y as f32,
        }
    }
}

struct ConstantMotion {
    vx: i32,
    vy: i32,
}

struct PathMotion {
    prec_x: f32,
    prec_y: f32,
    next_node: usize,
    base_pos: Vec2,
    offsets: Vec<Vec2>,
    speed: f32,
}

impl PathMotion {
    fn new(x: f32, y: f32, point_list: Vec<(f32, f32)>, speed: f32) -> Self {
        Self {
            prec_x: x,
            prec_y: y,
            next_node: 0,
            base_pos: vec2(x, y),
            offsets: point_list.iter().map(|(px, py)| vec2(*px, *py)).collect(),
            speed,
        }
    }
}

fn move_actor(
    actor: &mut Actor,
    rect: &mut IntRect,
    vx: f32,
    vy: f32,
    world: &World,
) -> (bool, bool) {
    actor.prec_x += vx;
    let targ_x = actor.prec_x.round() as i32;
    let mut collided_x = false;
    while rect.x != targ_x {
        let dx = (targ_x - rect.x).signum();
        if world
            .query::<&TileBody>()
            .iter()
            .any(|(_, c)| c.collide(&offset_rect(rect, dx, 0)))
        {
            actor.prec_x = rect.x as f32;
            collided_x = true;
            break;
        } else {
            rect.x += dx;
        }
    }
    actor.prec_y += vy;
    let targ_y = actor.prec_y.round() as i32;
    let mut collided_y = false;
    while rect.y != targ_y {
        let dy = (targ_y - rect.y).signum();
        if world
            .query::<&TileBody>()
            .iter()
            .any(|(_, c)| c.collide(&offset_rect(rect, 0, dy)))
        {
            actor.prec_y = rect.y as f32;
            collided_y = true;
            break;
        } else {
            rect.y += dy
        }
    }
    (collided_x, collided_y)
}

fn move_body(
    actor: &mut Actor,
    actor_rect: &mut IntRect,
    world: &World,
    index: Entity,
    vx: i32,
    vy: i32,
) {
    // this is a fiddly mess of borrows and drops but we should be able to skip
    // this in many cases if there are no actors in position to be pushed
    for _ii in 0..(vx.abs()) {
        let mut body = world.get::<&mut TileBody>(index).unwrap();
        let mut should_move = body.collide(&pushing_rect(&actor_rect));
        body.x += vx.signum();
        should_move |= body.collide(&pushing_rect(&actor_rect));
        drop(body);
        if should_move {
            move_actor(actor, actor_rect, vx.signum() as f32, 0.0, &world);
        }
    }
    for _ii in 0..(vy.abs()) {
        let mut body = world.get::<&mut TileBody>(index).unwrap();
        let mut should_move = body.collide(&pushing_rect(&actor_rect));
        body.y += vy.signum();
        should_move |= body.collide(&pushing_rect(&actor_rect));
        drop(body);
        if should_move {
            move_actor(actor, actor_rect, 0.0, vy.signum() as f32, &world);
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
    let mut body_ids: HashMap<String, Entity> = HashMap::new();
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
                body_ids.insert(
                    layer.name.clone(),
                    world.spawn((TileBody::new(
                        x0 * map.tile_width as i32,
                        y0 * map.tile_height as i32,
                        map.tile_width as i32,
                        (x1 - x0) + 1,
                        tiledata,
                    ),)),
                );
            }
            tiled::LayerType::ObjectLayer(data) => {
                for obj in data.objects() {
                    match &*obj {
                        tiled::ObjectData {
                            name,
                            shape: tiled::ObjectShape::Polyline { points },
                            ..
                        }
                        | tiled::ObjectData {
                            name,
                            shape: tiled::ObjectShape::Polygon { points },
                            ..
                        } => {
                            println!("found a path named {}", name);
                            paths.insert(name.clone(), points.clone());
                        }
                        _ => (),
                    }
                }
            }
            _ => println!("(Something other than an infinite tiled layer)"),
        }
    }

    world
        .insert_one(body_ids["layer1"], ConstantMotion { vx: -1, vy: 0 })
        .unwrap();
    world
        .insert_one(body_ids["layer2"], ConstantMotion { vx: 1, vy: 0 })
        .unwrap();
    world
        .insert_one(body_ids["layer3"], ConstantMotion { vx: 0, vy: -1 })
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
        for (e, cm) in world.query::<&ConstantMotion>().iter() {
            move_body(&mut player, &mut player_rect, &world, e, cm.vx, cm.vy);
        }

        for (e, pm) in world.query::<&mut PathMotion>().iter() {
            let dest = pm.offsets[pm.next_node] + pm.base_pos;
            let curr = vec2(pm.prec_x, pm.prec_y);
            let v = dest - curr;
            let tmp = if v.length() < pm.speed {
                pm.next_node = (pm.next_node + 1) % pm.offsets.len();
                dest
            } else {
                let new = curr + v.normalize() * pm.speed;
                new
            };
            pm.prec_x = tmp.x;
            pm.prec_y = tmp.y;

            let (dx, dy) = {
                let body = world.get::<&TileBody>(e).unwrap();
                (
                    pm.prec_x.round() as i32 - body.x,
                    pm.prec_y.round() as i32 - body.y,
                )
            };
            move_body(&mut player, &mut player_rect, &world, e, dx, dy);
        }

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

        let (cx, cy) = move_actor(&mut player, &mut player_rect, player_vx, player_vy, &world);
        if cx {
            player_vx = 0.0;
        }
        if cy {
            player_vy = 0.0;
        }

        player_grounded = world
            .query::<&TileBody>()
            .iter()
            .any(|(_, c)| c.collide(&feet_rect(&player_rect)));

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
