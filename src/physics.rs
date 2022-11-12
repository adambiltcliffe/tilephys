use crate::enemy::Enemy;
use crate::loader::TileFlags;
use hecs::{CommandBuffer, Entity, World};
use macroquad::math::{vec2, Vec2};
use std::collections::HashSet;

#[derive(PartialEq, Eq)]
enum CollisionType {
    Blocker,
    TopOfBlockerOrPlatform,
}

#[derive(Clone)]
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

    pub fn centre(&self) -> Vec2 {
        vec2((self.x + self.w / 2) as f32, (self.y + self.h / 2) as f32)
    }
}

fn offset_rect(rect: &IntRect, dx: i32, dy: i32) -> IntRect {
    IntRect::new(rect.x + dx, rect.y + dy, rect.w, rect.h)
}

fn offset_rect_up(rect: &IntRect) -> IntRect {
    IntRect::new(rect.x, rect.y - 1, rect.w, 1)
}

fn offset_rect_down(rect: &IntRect) -> IntRect {
    IntRect::new(rect.x, rect.y + rect.h, rect.w, 1)
}

fn pushing_rect(rect: &IntRect) -> IntRect {
    IntRect::new(rect.x, rect.y, rect.w, rect.h + 1)
}

fn feet_rect(rect: &IntRect) -> IntRect {
    IntRect::new(rect.x, rect.y + rect.h, rect.w, 1)
}

#[derive(PartialEq, Eq)]
pub enum Secrecy {
    NotSecret,
    HiddenSecret,
    FoundSecret,
}

pub struct TriggerZone {
    pub name: String,
    pub secrecy: Secrecy,
}

impl TriggerZone {
    pub fn new(name: String, secret: bool) -> Self {
        Self {
            name,
            secrecy: if secret {
                Secrecy::HiddenSecret
            } else {
                Secrecy::NotSecret
            },
        }
    }
}

pub struct TileBody {
    pub width: i32,
    pub size: i32,
    pub data: Vec<TileFlags>,
    pub tiles: Vec<u16>,
    pub x: i32,
    pub y: i32,
    pub base_pos: Vec2,
}

impl TileBody {
    pub fn new(
        x: i32,
        y: i32,
        size: i32,
        width: i32,
        data: Vec<TileFlags>,
        tiles: Vec<u16>,
    ) -> Self {
        Self {
            x,
            y,
            size,
            width,
            data,
            tiles,
            base_pos: vec2(x as f32, y as f32),
        }
    }

    fn collide(&self, rect: &IntRect, typ: CollisionType) -> bool {
        let adjustment = match typ {
            CollisionType::Blocker => 0,
            CollisionType::TopOfBlockerOrPlatform => self.size - 1,
        };
        let min_kx = (rect.x - self.x).div_euclid(self.size);
        let max_kx = (rect.x + rect.w - 1 - self.x).div_euclid(self.size);
        let min_ky = (rect.y - self.y + adjustment).div_euclid(self.size);
        let max_ky = (rect.y + rect.h - 1 - self.y).div_euclid(self.size);
        for ky in min_ky..=max_ky {
            if ky >= 0 {
                for kx in min_kx..=max_kx {
                    if kx >= 0 && kx < self.width {
                        let index = ky * self.width + kx;
                        if index >= 0
                            && index < self.data.len() as i32
                            && (self.data[index as usize].is_blocker()
                                || self.data[index as usize].is_platform()
                                    && typ == CollisionType::TopOfBlockerOrPlatform)
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
    pub drag: f32,
}

impl Actor {
    pub fn new(rect: &IntRect, drag: f32) -> Self {
        Self {
            prec_x: rect.x as f32,
            prec_y: rect.y as f32,
            vx: 0.0,
            vy: 0.0,
            grounded: false,
            drag,
        }
    }

    pub fn update(world: &World) {
        for (_, (actor, rect)) in world.query::<(&mut Actor, &mut IntRect)>().iter() {
            actor.vy += 1.0;
            actor.vx *= actor.drag;
            actor.vy = actor.vy.min(16.0);
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

pub struct Projectile {
    prec_x: f32,
    prec_y: f32,
    pub vx: f32,
    pub vy: f32,
}

impl Projectile {
    pub fn new(rect: &IntRect, vx: f32, vy: f32) -> Self {
        Self {
            prec_x: rect.x as f32,
            prec_y: rect.y as f32,
            vx,
            vy,
        }
    }

    pub fn update(world: &World, buffer: &mut CommandBuffer) {
        for (e, (proj, rect)) in world.query::<(&mut Projectile, &mut IntRect)>().iter() {
            proj.prec_x += proj.vx;
            proj.prec_y += proj.vy;
            rect.x = proj.prec_x.round() as i32;
            rect.y = proj.prec_y.round() as i32;
            if world
                .query::<&TileBody>()
                .iter()
                .any(|(_, c)| c.collide(rect, CollisionType::Blocker))
            {
                buffer.despawn(e)
            }
            let mut live = true;
            world
                .query::<(&mut Enemy, &IntRect)>()
                .iter()
                .for_each(|(e_id, (en, e_rect))| {
                    if live && rect.intersects(&e_rect) {
                        buffer.despawn(e);
                        en.hp -= 1;
                        if en.hp <= 0 {
                            buffer.despawn(e_id)
                        }
                        live = false;
                    }
                });
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
    pub speed: f32,
    prec_x: f32,
    prec_y: f32,
    next_node: usize,
    offsets: Vec<Vec2>,
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
            let tmp = if v.length() <= pm.speed {
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
                curr + v.normalize() * pm.speed
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
            .any(|(_, c)| c.collide(&offset_rect(rect, dx, 0), CollisionType::Blocker))
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
        let clo: Box<dyn FnMut((_, &TileBody)) -> bool> = if dy == -1 {
            Box::new(|(_, c): (_, &TileBody)| {
                c.collide(&offset_rect_up(rect), CollisionType::Blocker)
            })
        } else {
            Box::new(|(_, c): (_, &TileBody)| {
                c.collide(
                    &offset_rect_down(rect),
                    CollisionType::TopOfBlockerOrPlatform,
                )
            })
        };
        if world.query::<&TileBody>().iter().any(clo) {
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
            if body.collide(&pushing_rect(rect), CollisionType::Blocker) {
                should_move.insert(e);
            }
        }
        body.x += vx.signum();
        for (e, (_, rect)) in world.query::<(&Actor, &IntRect)>().iter() {
            if body.collide(&pushing_rect(rect), CollisionType::Blocker) {
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
            if body.collide(&pushing_rect(rect), CollisionType::Blocker) {
                should_move.insert(e);
            }
        }
        body.y += vy.signum();
        for (e, (_, rect)) in world.query::<(&Actor, &IntRect)>().iter() {
            if body.collide(&pushing_rect(rect), CollisionType::Blocker) {
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
    world.query::<&TileBody>().iter().any(|(_, c)| {
        c.collide(
            &feet_rect(&player_rect),
            CollisionType::TopOfBlockerOrPlatform,
        )
    })
}
