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

    pub fn intersects(&self, other: &IntRect) -> bool {
        self.x < other.x + other.w
            && self.y < other.y + other.h
            && self.x + self.w > other.x
            && self.y + self.h > other.y
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

pub struct TriggerZone {
    pub name: String,
}

pub struct TileBody {
    pub width: i32,
    pub size: i32,
    pub data: Vec<bool>,
    pub x: i32,
    pub y: i32,
    pub base_pos: Vec2,
}

impl TileBody {
    pub fn new(x: i32, y: i32, size: i32, width: i32, data: Vec<bool>) -> Self {
        Self {
            x,
            y,
            size,
            width,
            data,
            base_pos: vec2(x as f32, y as f32),
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

    pub fn update(world: &World) {
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
    triggers: HashSet<String>,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            jump_frames: 0,
            triggers: HashSet::new(),
        }
    }

    pub fn update(world: &World) -> HashSet<String> {
        let mut result: HashSet<String> = HashSet::new();
        use macroquad::input::{is_key_down, is_key_pressed, KeyCode};
        for (_, (player, p_rect, controller)) in world
            .query::<(&mut Actor, &IntRect, &mut Controller)>()
            .iter()
        {
            let mut new_triggers: HashSet<String> = HashSet::new();
            for (_, (trigger, t_rect)) in world.query::<(&TriggerZone, &IntRect)>().iter() {
                if p_rect.intersects(&t_rect) {
                    let name = trigger.name.clone();
                    if !controller.triggers.contains(&name) {
                        result.insert(name.clone());
                    }
                    new_triggers.insert(name);
                }
            }
            controller.triggers = new_triggers;
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
        result
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
    pub fn apply(world: &World) {
        for (e, cm) in world.query::<&ConstantMotion>().iter() {
            move_body(&world, e, cm.vx, cm.vy);
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum PathMotionType {
    Static,
    GoToNode(usize),
    ForwardOnce,
    ForwardCycle,
}

pub struct PathMotion {
    pub path_name: String,
    pub motion_type: PathMotionType,
    prec_x: f32,
    prec_y: f32,
    next_node: usize,
    offsets: Vec<Vec2>,
    speed: f32,
}

impl PathMotion {
    pub fn new(
        path_name: &str,
        x: f32,
        y: f32,
        point_list: Vec<(f32, f32)>,
        speed: f32,
        motion_type: PathMotionType,
    ) -> Self {
        Self {
            path_name: path_name.to_owned(),
            prec_x: x,
            prec_y: y,
            next_node: 0,
            offsets: point_list.iter().map(|(px, py)| vec2(*px, *py)).collect(),
            speed,
            motion_type,
        }
    }

    pub fn apply(world: &World) {
        for (e, pm) in world.query::<&mut PathMotion>().iter() {
            let dest = {
                let body = world.get::<&TileBody>(e).unwrap();
                pm.offsets[pm.next_node] + body.base_pos
            };
            let curr = vec2(pm.prec_x, pm.prec_y);
            let v = dest - curr;
            let tmp = if v.length() < pm.speed {
                // reached the current destination node
                match &pm.motion_type {
                    PathMotionType::Static => (),
                    PathMotionType::GoToNode(index) => {
                        if index < &pm.next_node {
                            pm.next_node -= 1;
                        } else if index > &pm.next_node {
                            pm.next_node += 1;
                        }
                    }
                    PathMotionType::ForwardOnce => {
                        if pm.next_node < pm.offsets.len() - 1 {
                            pm.next_node += 1;
                        }
                    }
                    PathMotionType::ForwardCycle => {
                        pm.next_node = (pm.next_node + 1) % pm.offsets.len();
                    }
                }
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
