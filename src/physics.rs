use hecs::{Entity, World};
use macroquad::math::{vec2, Vec2};
use std::collections::HashSet;

pub struct IntRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl IntRect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
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

pub struct TileBody {
    pub width: i32,
    pub size: i32,
    pub data: Vec<bool>,
    pub x: i32,
    pub y: i32,
}

impl TileBody {
    pub fn new(x: i32, y: i32, size: i32, width: i32, data: Vec<bool>) -> Self {
        Self {
            x,
            y,
            size,
            width,
            data,
        }
    }

    // this is only pub at the moment because of checking if the mouse intersects it for debugging
    pub fn collide(&self, rect: &IntRect) -> bool {
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

pub struct Actor {
    prec_x: f32,
    prec_y: f32,
    pub vx: f32,
    pub vy: f32,
    pub grounded: bool,
}

impl Actor {
    pub fn new(rect: &IntRect) -> Self {
        Self {
            prec_x: rect.x as f32,
            prec_y: rect.y as f32,
            vx: 0.0,
            vy: 0.0,
            grounded: false,
        }
    }

    pub fn update(world: &mut World) {
        for (_, (actor, rect)) in world.query::<(&mut Actor, &mut IntRect)>().iter() {
            actor.vy += 1.0;
            actor.vx *= 0.6;
            let vx = actor.vx;
            let vy = actor.vy;
            let (cx, cy) = move_actor(actor, rect, vx, vy, &world);
            if cx {
                actor.vx = 0.0;
            }
            if cy {
                actor.vy = 0.0;
            }
            actor.grounded = check_player_grounded(&rect, &world);
        }
    }
}

pub struct Controller {
    jump_frames: u32,
}

impl Controller {
    pub fn new() -> Self {
        Self { jump_frames: 0 }
    }

    pub fn update(world: &mut World) {
        use macroquad::input::{is_key_down, is_key_pressed, KeyCode};
        for (_, (player, controller)) in world.query::<(&mut Actor, &mut Controller)>().iter() {
            if is_key_down(KeyCode::Left) {
                player.vx -= 3.0;
            }
            if is_key_down(KeyCode::Right) {
                player.vx += 3.0;
            }
            if player.grounded && is_key_pressed(KeyCode::X) {
                player.vy = -6.0;
                controller.jump_frames = 5;
            } else if controller.jump_frames > 0 && is_key_down(KeyCode::X) {
                player.vy = -6.0;
                controller.jump_frames -= 1;
            } else {
                controller.jump_frames = 0;
            }
        }
    }
}

pub struct ConstantMotion {
    vx: i32,
    vy: i32,
}

impl ConstantMotion {
    pub fn new(vx: i32, vy: i32) -> Self {
        Self { vx, vy }
    }
    pub fn apply(world: &mut World) {
        for (e, cm) in world.query::<&ConstantMotion>().iter() {
            move_body(&world, e, cm.vx, cm.vy);
        }
    }
}

pub struct PathMotion {
    prec_x: f32,
    prec_y: f32,
    next_node: usize,
    base_pos: Vec2,
    offsets: Vec<Vec2>,
    speed: f32,
}

impl PathMotion {
    pub fn new(x: f32, y: f32, point_list: Vec<(f32, f32)>, speed: f32) -> Self {
        Self {
            prec_x: x,
            prec_y: y,
            next_node: 0,
            base_pos: vec2(x, y),
            offsets: point_list.iter().map(|(px, py)| vec2(*px, *py)).collect(),
            speed,
        }
    }

    pub fn apply(world: &mut World) {
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
            move_body(&world, e, dx, dy);
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

fn move_body(world: &World, index: Entity, vx: i32, vy: i32) {
    // this is a fiddly mess of borrows and drops but we should be able to skip
    // this in many cases if there are no actors in position to be pushed
    for _ii in 0..(vx.abs()) {
        let mut body = world.get::<&mut TileBody>(index).unwrap();
        let mut should_move = HashSet::new();
        for (e, (_, rect)) in world.query::<(&Actor, &IntRect)>().iter() {
            if body.collide(&pushing_rect(rect)) {
                should_move.insert(e);
            }
        }
        body.x += vx.signum();
        for (e, (_, rect)) in world.query::<(&Actor, &IntRect)>().iter() {
            if body.collide(&pushing_rect(rect)) {
                should_move.insert(e);
            }
        }
        drop(body);
        for e in should_move {
            let mut actor = world.get::<&mut Actor>(e).unwrap();
            let mut rect = world.get::<&mut IntRect>(e).unwrap();
            move_actor(&mut *actor, &mut *rect, vx.signum() as f32, 0.0, &world);
        }
    }
    for _ii in 0..(vy.abs()) {
        let mut body = world.get::<&mut TileBody>(index).unwrap();
        let mut should_move = HashSet::new();
        for (e, (_, rect)) in world.query::<(&Actor, &IntRect)>().iter() {
            if body.collide(&pushing_rect(rect)) {
                should_move.insert(e);
            }
        }
        body.y += vy.signum();
        for (e, (_, rect)) in world.query::<(&Actor, &IntRect)>().iter() {
            if body.collide(&pushing_rect(rect)) {
                should_move.insert(e);
            }
        }
        drop(body);
        for e in should_move {
            let mut actor = world.get::<&mut Actor>(e).unwrap();
            let mut rect = world.get::<&mut IntRect>(e).unwrap();
            move_actor(&mut *actor, &mut *rect, 0.0, vy.signum() as f32, &world);
        }
    }
}

fn check_player_grounded(player_rect: &IntRect, world: &World) -> bool {
    world
        .query::<&TileBody>()
        .iter()
        .any(|(_, c)| c.collide(&feet_rect(&player_rect)))
}
