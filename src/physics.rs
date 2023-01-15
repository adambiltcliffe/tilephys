use crate::index::SpatialIndex;
use crate::loader::TileFlags;
use crate::resources::SceneResources;
use hecs::{Entity, World};
use macroquad::math::{vec2, Vec2};
use std::{cmp::Ordering, collections::HashSet};

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

    pub fn centre_int(&self) -> (i32, i32) {
        (self.x + self.w / 2, self.y + self.h / 2)
    }
}

fn pushing_rect(rect: &IntRect) -> IntRect {
    IntRect::new(rect.x, rect.y, rect.w, rect.h + 1)
}

fn feet_rect(rect: &IntRect) -> IntRect {
    IntRect::new(rect.x, rect.y + rect.h, rect.w, 1)
}

fn motion_rect(rect: &IntRect, targ_x: i32, targ_y: i32) -> IntRect {
    IntRect::new(
        rect.x.min(targ_x),
        rect.y.min(targ_y),
        rect.w + (rect.x - targ_x).abs(),
        rect.h + (rect.y - targ_y).abs(),
    )
}

#[derive(PartialEq, Eq)]
pub enum Secrecy {
    NotSecret,
    Hidden,
    Found,
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
                Secrecy::Hidden
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
    pub door: bool,
}

impl TileBody {
    pub fn new(
        x: i32,
        y: i32,
        size: i32,
        width: i32,
        data: Vec<TileFlags>,
        tiles: Vec<u16>,
        door: bool,
    ) -> Self {
        Self {
            x,
            y,
            size,
            width,
            data,
            tiles,
            base_pos: vec2(x as f32, y as f32),
            door,
        }
    }

    pub fn get_rect(&self) -> IntRect {
        IntRect {
            x: self.x,
            y: self.y,
            w: self.width * self.size,
            h: (self.data.len() as i32 / self.width) * self.size,
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

    fn collide_dist_up(&self, rect: &IntRect, max_dist: i32) -> i32 {
        let mut possible_dist = (rect.y - self.y).rem_euclid(self.size);
        let mut ky = (rect.y - self.y).div_euclid(self.size) - 1;
        let min_kx = (rect.x - self.x).div_euclid(self.size).max(0);
        let max_kx = (rect.x + rect.w - 1 - self.x)
            .div_euclid(self.size)
            .min(self.width - 1);
        if ky >= 0 {
            while possible_dist < max_dist {
                for kx in min_kx..=max_kx {
                    let index = ky * self.width + kx;
                    if index < self.data.len() as i32 && self.data[index as usize].is_blocker() {
                        return possible_dist;
                    }
                }
                possible_dist += self.size;
                ky -= 1;
            }
        }
        max_dist
    }

    fn collide_dist_down(&self, rect: &IntRect, max_dist: i32) -> i32 {
        let mut possible_dist = (self.y - (rect.y + rect.h)).rem_euclid(self.size);
        let mut ky = (rect.y + rect.h - self.y + possible_dist).div_euclid(self.size);
        let min_kx = (rect.x - self.x).div_euclid(self.size).max(0);
        let max_kx = (rect.x + rect.w - 1 - self.x)
            .div_euclid(self.size)
            .min(self.width - 1);
        if ky >= 0 {
            while possible_dist < max_dist {
                for kx in min_kx..=max_kx {
                    let index = ky * self.width + kx;
                    if index < self.data.len() as i32
                        && (self.data[index as usize].is_blocker()
                            || self.data[index as usize].is_platform())
                    {
                        return possible_dist;
                    }
                }
                possible_dist += self.size;
                ky += 1;
            }
        }
        max_dist
    }

    fn collide_dist_left(&self, rect: &IntRect, max_dist: i32) -> i32 {
        let mut possible_dist = (rect.x - self.x).rem_euclid(self.size);
        let mut kx = (rect.x - self.x).div_euclid(self.size) - 1;
        let min_ky = (rect.y - self.y).div_euclid(self.size).max(0);
        let max_ky = (rect.y + rect.h - 1 - self.y).div_euclid(self.size);
        while possible_dist < max_dist {
            if kx >= 0 && kx < self.width {
                for ky in min_ky..=max_ky {
                    let index = ky * self.width + kx;
                    if index < self.data.len() as i32 && self.data[index as usize].is_blocker() {
                        return possible_dist;
                    }
                }
            }
            possible_dist += self.size;
            kx -= 1;
        }
        max_dist
    }

    fn collide_dist_right(&self, rect: &IntRect, max_dist: i32) -> i32 {
        let mut possible_dist = (self.x - (rect.x + rect.w)).rem_euclid(self.size);
        let mut kx = (rect.x + rect.w - self.x + possible_dist).div_euclid(self.size);
        let min_ky = (rect.y - self.y).div_euclid(self.size).max(0);
        let max_ky = (rect.y + rect.h - 1 - self.y).div_euclid(self.size);
        while possible_dist < max_dist {
            if kx >= 0 && kx < self.width {
                for ky in min_ky..=max_ky {
                    let index = ky * self.width + kx;
                    if index < self.data.len() as i32 && self.data[index as usize].is_blocker() {
                        return possible_dist;
                    }
                }
            }
            possible_dist += self.size;
            kx += 1;
        }
        max_dist
    }
}

pub struct Actor {
    prec_x: f32,
    prec_y: f32,
    pub vx: f32,
    pub vy: f32,
    pub grounded: bool,
    pub crushed: bool,
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
            crushed: false,
            drag,
        }
    }

    pub fn update(resources: &SceneResources) {
        let world = resources.world_ref.lock().unwrap();
        for (_, (actor, rect)) in world.query::<(&mut Actor, &mut IntRect)>().iter() {
            actor.vy += 1.0;
            actor.vx *= actor.drag;
            actor.vy = actor.vy.min(16.0);
            let vx = actor.vx;
            let vy = actor.vy;
            let (cx, cy) = move_actor(actor, rect, vx, vy, &world, &resources.body_index);
            if cx {
                actor.vx = 0.0;
            }
            if cy {
                actor.vy = 0.0;
            }
            actor.grounded = check_player_grounded(rect, &world);
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum PathMotionType {
    Static,
    GoToNodeForward(usize),
    GoToNodeBackward(usize),
    ForwardOnce,
    ForwardCycle,
}

pub struct PathMotion {
    pub motion_type: PathMotionType,
    pub speed: f32,
    prec_x: f32,
    prec_y: f32,
    next_node: usize,
    offsets: Vec<Vec2>,
}

impl PathMotion {
    pub fn new(
        x: f32,
        y: f32,
        point_list: &[(f32, f32)],
        speed: f32,
        motion_type: PathMotionType,
    ) -> Self {
        Self {
            prec_x: x,
            prec_y: y,
            next_node: 0,
            offsets: point_list.iter().map(|(px, py)| vec2(*px, *py)).collect(),
            speed,
            motion_type,
        }
    }

    pub fn set_dest_node(&mut self, index: usize) {
        let prev_node = match self.motion_type {
            PathMotionType::GoToNodeBackward(_) => self.next_node + 1,
            _ => {
                // all other types are forwards
                if self.next_node == 0 {
                    0 // avoid negative index
                } else {
                    self.next_node - 1
                }
            }
        };
        match index.cmp(&self.next_node) {
            Ordering::Greater => {
                self.motion_type = PathMotionType::GoToNodeForward(index);
                if index >= prev_node {
                    self.next_node = prev_node
                }
            }
            Ordering::Less => {
                self.motion_type = PathMotionType::GoToNodeBackward(index);
                if index <= prev_node {
                    self.next_node = prev_node
                }
            }
            Ordering::Equal => {
                // already going to the right node, but could be wrong motion type
                self.motion_type = match self.motion_type {
                    PathMotionType::GoToNodeBackward(_) => PathMotionType::GoToNodeBackward(index),
                    _ => PathMotionType::GoToNodeForward(index),
                };
            }
        }
    }

    pub fn apply(resources: &mut SceneResources) {
        let world = resources.world_ref.lock().unwrap();
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
                    PathMotionType::GoToNodeForward(index) => {
                        if index > &pm.next_node {
                            pm.next_node += 1;
                        }
                    }
                    PathMotionType::GoToNodeBackward(index) => {
                        if index < &pm.next_node {
                            pm.next_node -= 1;
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
            if dx != 0 || dy != 0 {
                // try to move it
                if !move_body(&world, &mut resources.body_index, e, dx, dy) {
                    // body was a door that was stopped by a collision
                    let body = world.get::<&TileBody>(e).unwrap();
                    pm.prec_x = body.x as f32;
                    pm.prec_y = body.y as f32;
                }
            }
        }
    }
}

pub fn collide_any(world: &World, body_index: &SpatialIndex, rect: &IntRect) -> bool {
    let blockers = body_index.entities(rect);
    blockers.iter().any(|id| {
        world
            .get::<&TileBody>(*id)
            .unwrap()
            .collide(rect, CollisionType::Blocker)
    })
}

fn move_actor(
    actor: &mut Actor,
    rect: &mut IntRect,
    vx: f32,
    vy: f32,
    world: &World,
    body_index: &SpatialIndex,
) -> (bool, bool) {
    actor.prec_x += vx;
    let targ_x = actor.prec_x.round() as i32;
    let mut collided_x = false;
    actor.prec_y += vy;
    let targ_y = actor.prec_y.round() as i32;
    let mut collided_y = false;
    let blockers = body_index.entities(&motion_rect(rect, targ_x, targ_y));
    match targ_x.cmp(&rect.x) {
        Ordering::Less => {
            // handle moving left
            let mut d = rect.x - targ_x;
            for id in blockers.iter() {
                d = d.min(
                    world
                        .get::<&TileBody>(*id)
                        .unwrap()
                        .collide_dist_left(rect, d),
                );
            }
            rect.x -= d;
        }
        Ordering::Equal => (),
        Ordering::Greater => {
            // handle moving right
            let mut d = targ_x - rect.x;
            for id in blockers.iter() {
                d = d.min(
                    world
                        .get::<&TileBody>(*id)
                        .unwrap()
                        .collide_dist_right(rect, d),
                );
            }
            rect.x += d;
        }
    }
    if rect.x != targ_x {
        actor.prec_x = rect.x as f32;
        collided_x = true;
    }
    match targ_y.cmp(&rect.y) {
        Ordering::Less => {
            // handle moving up
            let mut d = rect.y - targ_y;
            for id in blockers.iter() {
                d = d.min(
                    world
                        .get::<&TileBody>(*id)
                        .unwrap()
                        .collide_dist_up(rect, d),
                );
            }
            rect.y -= d;
        }
        Ordering::Equal => (),
        Ordering::Greater => {
            // handle moving down
            let mut d = targ_y - rect.y;
            for id in blockers.iter() {
                d = d.min(
                    world
                        .get::<&TileBody>(*id)
                        .unwrap()
                        .collide_dist_down(rect, d),
                );
            }
            rect.y += d;
        }
    }
    if rect.y != targ_y {
        actor.prec_y = rect.y as f32;
        collided_y = true;
    }
    (collided_x, collided_y)
}

fn move_body(
    world: &World,
    spatial_index: &mut SpatialIndex,
    index: Entity,
    vx: i32,
    vy: i32,
) -> bool {
    let mut stopped = false;
    let body = world.get::<&mut TileBody>(index).unwrap();
    spatial_index.remove_at(index, &body.get_rect());
    drop(body);
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
        if body.door && !should_move.is_empty() {
            body.x -= vx.signum();
            stopped = true;
            break;
        }
        drop(body);
        for e in &should_move {
            let mut actor = world.get::<&mut Actor>(*e).unwrap();
            let mut rect = world.get::<&mut IntRect>(*e).unwrap();
            move_actor(
                &mut actor,
                &mut rect,
                vx.signum() as f32,
                0.0,
                world,
                spatial_index,
            );
        }
        // this is ridiculous, now we have moved the actor we have to borrow the body
        // again to check if it crushed the actor
        let body = world.get::<&mut TileBody>(index).unwrap();
        for e in should_move {
            let mut actor = world.get::<&mut Actor>(e).unwrap();
            let rect = world.get::<&mut IntRect>(e).unwrap();
            if body.collide(&rect, CollisionType::Blocker) {
                actor.crushed = true;
            }
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
        if body.door && !should_move.is_empty() {
            body.y -= vy.signum();
            stopped = true;
            break;
        }
        drop(body);
        for e in &should_move {
            let mut actor = world.get::<&mut Actor>(*e).unwrap();
            let mut rect = world.get::<&mut IntRect>(*e).unwrap();
            move_actor(
                &mut actor,
                &mut rect,
                0.0,
                vy.signum() as f32,
                world,
                spatial_index,
            );
        }
        // and again
        let body = world.get::<&mut TileBody>(index).unwrap();
        for e in should_move {
            let mut actor = world.get::<&mut Actor>(e).unwrap();
            let rect = world.get::<&mut IntRect>(e).unwrap();
            if body.collide(&rect, CollisionType::Blocker) {
                actor.crushed = true;
            }
        }
    }
    let body = world.get::<&mut TileBody>(index).unwrap();
    spatial_index.insert_at(index, &body.get_rect());
    drop(body);
    !stopped
}

fn check_player_grounded(player_rect: &IntRect, world: &World) -> bool {
    world.query::<&TileBody>().iter().any(|(_, c)| {
        c.collide(
            &feet_rect(player_rect),
            CollisionType::TopOfBlockerOrPlatform,
        )
    })
}
