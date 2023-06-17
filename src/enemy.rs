use std::cmp::Ordering;
use std::convert::TryInto;
use std::num::NonZeroU8;

use crate::config::config;
use crate::draw::{DogSprite, DroneSprite, ParrotSprite};
use crate::physics::{collide_any, find_collision_pos, Actor, IntRect, PhysicsCoeffs};
use crate::player::Controller;
use crate::projectile::{make_enemy_fireball, make_enemy_laser, railgun_intersects, RailgunHitbox};
use crate::ray::ray_collision;
use crate::resources::SceneResources;
use crate::vfx::{create_explosion, create_smoke_cloud, make_railgun_trail};
use hecs::{CommandBuffer, Entity, World};
use macroquad::prelude::*;

const FRAC_2_SQRT_3: f32 = 1.1547005177; // one over sin(60)

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum EnemyKind {
    Dog,
    JumpyDog,
    SpiderParrot(ParrotKind),
    Drone,
}

pub fn add_enemy(world: &mut World, kind: EnemyKind, x: i32, y: i32) {
    let (w, h, c) = match kind {
        EnemyKind::Drone => (16, 16, PhysicsCoeffs::Flyer),
        EnemyKind::SpiderParrot(_) => (24, 24, PhysicsCoeffs::Actor),
        EnemyKind::Dog | EnemyKind::JumpyDog => (24, 16, PhysicsCoeffs::Actor),
    };
    let rect = IntRect::new(x - w / 2, y - h, w, h);
    let actor = Actor::new(&rect, c);
    let hp = match kind {
        EnemyKind::SpiderParrot(_) => 7,
        _ => 3,
    };
    let hittable = EnemyHittable::new(hp);
    let dmg = EnemyContactDamage::new();
    if let EnemyKind::SpiderParrot(pk) = kind {
        world.spawn((
            kind,
            ParrotBehaviour::new(pk),
            rect,
            crate::draw::ParrotSprite::new(pk),
            actor,
            hittable,
            dmg,
        ));
    } else if matches!(kind, EnemyKind::Drone) {
        world.spawn((
            kind,
            DroneBehaviour::new(rect.centre()),
            rect,
            crate::draw::DroneSprite::new(),
            actor,
            hittable,
            dmg,
        ));
    } else {
        world.spawn((
            kind,
            DogBehaviour::new(),
            rect,
            crate::draw::DogSprite::new(),
            actor,
            hittable,
            dmg,
        ));
    }
}

fn with_prob(p: f32) -> bool {
    quad_rand::gen_range(0.0, 1.0) < p
}

fn rand_sign() -> f32 {
    quad_rand::gen_range(0, 2) as f32 * 2.0 - 1.0
}

fn player_x(world: &World, player_id: Entity) -> Option<f32> {
    let mut q = world.query_one::<(&Controller, &IntRect)>(player_id).ok()?;
    let (_, rect) = q.get()?;
    Some(rect.centre().x)
}

fn player_y(world: &World, player_id: Entity) -> Option<f32> {
    let mut q = world.query_one::<(&Controller, &IntRect)>(player_id).ok()?;
    let (_, rect) = q.get()?;
    Some(rect.centre().y)
}

pub struct EnemyHittable {
    pub hp: u16,
    pub was_hit: bool,
}

impl EnemyHittable {
    pub fn new(hp: u16) -> Self {
        Self { hp, was_hit: false }
    }

    pub fn hurt(&mut self, amount: u16) {
        self.hp -= amount.min(self.hp);
        self.was_hit = true;
    }
}

struct EnemyContactDamage {}

impl EnemyContactDamage {
    pub fn new() -> Self {
        Self {}
    }
}

struct DogBehaviour {
    dir: f32,
    jump_y: Option<i32>,
}

impl DogBehaviour {
    pub fn new() -> Self {
        Self {
            dir: 0.0,
            jump_y: None,
        }
    }

    pub fn update(world: &World, resources: &SceneResources) {
        let player_x = player_x(world, resources.player_id);
        for (_, (kind, actor, enemy, rect, spr)) in world
            .query::<(
                &EnemyKind,
                &mut Actor,
                &mut DogBehaviour,
                &IntRect,
                &mut DogSprite,
            )>()
            .iter()
        {
            if (actor.grounded || enemy.jump_y.is_some()) && with_prob(0.1) {
                match player_x {
                    Some(x) if with_prob(0.7) => {
                        enemy.dir = (x - rect.centre().x).signum() * 5.0;
                    }
                    _ => {
                        enemy.dir = 5.0 * rand_sign();
                    }
                }
            }
            if actor.grounded {
                let (jump_prob, jump_vel) = match kind {
                    EnemyKind::Dog => (0.45, -6.0),
                    EnemyKind::JumpyDog => (0.2, -8.0),
                    _ => unreachable!(),
                };
                if with_prob(jump_prob) {
                    actor.vy = jump_vel;
                    enemy.jump_y = Some(rect.y);
                } else {
                    enemy.jump_y = None;
                }
            } else {
                // stop moving horizontally if ground has fallen out from under us
                if match enemy.jump_y {
                    None => true,
                    Some(y) => y < rect.y,
                } {
                    enemy.dir = 0.0;
                }
            }
            actor.vx += enemy.dir;
            if actor.vx < 0.0 {
                spr.flipped = false
            }
            if actor.vx > 0.0 {
                spr.flipped = true
            }
            spr.n += 1;
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ParrotKind {
    Laser,
    Cannon,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ParrotState {
    Wait,
    Move,
    Attack,
    Fall,
}

struct ParrotBehaviour {
    kind: ParrotKind,
    state: ParrotState,
    state_timer: u8,
    attack_timer: u8,
    facing: i8,
}

impl ParrotBehaviour {
    pub fn new(kind: ParrotKind) -> Self {
        Self {
            kind,
            state: ParrotState::Wait,
            state_timer: 0,
            attack_timer: 0,
            facing: -1,
        }
    }

    fn set_state(&mut self, state: ParrotState) {
        self.state = state;
        self.state_timer = 0;
    }

    pub fn update(world: &World, resources: &SceneResources, buffer: &mut CommandBuffer) {
        let player_x = player_x(world, resources.player_id);
        for (_, (actor, beh, rect, spr)) in world
            .query::<(
                &mut Actor,
                &mut ParrotBehaviour,
                &IntRect,
                &mut ParrotSprite,
            )>()
            .iter()
        {
            if !actor.grounded {
                beh.set_state(ParrotState::Fall);
            }

            spr.frame = 0;
            spr.muzzle_flash = None;
            let new_vx = actor.vx + 5.0 * beh.facing as f32;
            match beh.state {
                ParrotState::Fall => {
                    if actor.grounded {
                        beh.set_state(ParrotState::Wait);
                    }
                }
                ParrotState::Wait => {
                    if quad_rand::gen_range(10, 20) < beh.state_timer {
                        let is_facing_player = player_x.map_or(true, |x| {
                            (x - rect.centre().x).signum() == beh.facing as f32
                        });
                        if with_prob(0.5) {
                            let will_attack = match beh.kind {
                                ParrotKind::Laser => {
                                    is_facing_player && beh.attack_timer == 0 && with_prob(0.85)
                                }
                                ParrotKind::Cannon => {
                                    is_facing_player
                                        && beh.attack_timer == 0
                                        && player_y(world, resources.player_id)
                                            .map_or(false, |y| (y - rect.centre().y).abs() < 48.0)
                                        && player_x.map_or(false, |x| {
                                            let min_x = (x + 16.0).min(rect.centre().x);
                                            let max_x = (x - 16.0).max(rect.centre().x);
                                            let w = max_x - min_x;
                                            let r = IntRect::new(
                                                min_x as i32,
                                                rect.y,
                                                w as i32,
                                                rect.h,
                                            );
                                            !collide_any(world, &resources.body_index, &r)
                                        })
                                }
                            };
                            if will_attack {
                                beh.set_state(ParrotState::Attack);
                            } else {
                                beh.facing = -beh.facing;
                                beh.state_timer = 0;
                            }
                        } else if !parrot_should_stop(world, resources, rect, new_vx) {
                            beh.set_state(ParrotState::Move);
                        }
                    }
                }
                ParrotState::Move => {
                    spr.frame = (beh.state_timer / 2) % 2;
                    if beh.state_timer > 10 && with_prob(0.05)
                        || parrot_should_stop(world, resources, rect, new_vx)
                    {
                        beh.set_state(ParrotState::Wait);
                    } else {
                        actor.vx = new_vx;
                    }
                }
                ParrotState::Attack => {
                    let (freq, limit, delay) = match beh.kind {
                        ParrotKind::Laser => (6, 24, 30),
                        ParrotKind::Cannon => (24, 12, 120),
                    };
                    spr.frame = 3;
                    let mf = beh.state_timer % freq;
                    spr.muzzle_flash = if mf > 0 && mf < 5 { Some(mf) } else { None };
                    if beh.state_timer % freq == 1 {
                        actor.vx -= beh.facing as f32 * 10.0;
                        let new_x = rect.x + 7 + beh.facing as i32 * 6;
                        match beh.kind {
                            ParrotKind::Laser => {
                                let rect = IntRect::new(new_x, rect.y + 8, 8, 5);
                                make_enemy_laser(buffer, rect, beh.facing as f32 * 4.0);
                            }
                            ParrotKind::Cannon => {
                                let rect = IntRect::new(new_x - 6, rect.y + 4, 12, 12);
                                make_enemy_fireball(
                                    buffer,
                                    rect,
                                    beh.facing as f32 * 2.0,
                                    0.0,
                                    true,
                                );
                            }
                        }
                    }
                    if beh.state_timer % freq == freq - 1
                        && parrot_off_edge(world, resources, rect, beh.facing)
                    {
                        beh.set_state(ParrotState::Move);
                    } else if beh.state_timer >= limit {
                        beh.attack_timer = delay;
                        beh.set_state(ParrotState::Wait);
                    }
                }
            }

            if beh.attack_timer > 0 {
                beh.attack_timer -= 1;
            }
            beh.state_timer += 1;
            if beh.facing < 0 {
                spr.flipped = false
            } else {
                spr.flipped = true
            }
        }
    }
}

fn parrot_should_stop(world: &World, resources: &SceneResources, rect: &IntRect, vx: f32) -> bool {
    let d = vx.abs().ceil() as i32;
    let (wall_rect_x, floor_rect_x) = match vx.total_cmp(&0.0) {
        Ordering::Equal => return false,
        Ordering::Less => (rect.x - d, rect.x - d),
        Ordering::Greater => (rect.x + rect.w, rect.x + rect.w + d - 1),
    };
    let wall_rect = IntRect::new(wall_rect_x, rect.y, d, rect.h);
    let floor_rect = IntRect::new(floor_rect_x, rect.y + rect.h, 1, 1);
    collide_any(world, &resources.body_index, &wall_rect)
        || !collide_any(world, &resources.body_index, &floor_rect)
}

// detect whether the enemy's rear foot is sliding off a cliff as a result of firing recoil
fn parrot_off_edge(world: &World, resources: &SceneResources, rect: &IntRect, facing: i8) -> bool {
    let x = if facing > 0 {
        rect.x
    } else {
        rect.x + rect.w - 1
    };
    !collide_any(
        world,
        &resources.body_index,
        &IntRect::new(x, rect.y + rect.h, 1, 1),
    )
}

enum DroneFiringState {
    Delay(u8),
    Ready,
    Seeking(Entity),
}

pub struct Reticule {
    parent: Entity,
    pub pos: Vec2,
    pub lock_timer: Option<NonZeroU8>,
}

impl Reticule {
    pub fn new(parent: Entity, pos: Vec2) -> Self {
        Self {
            parent,
            pos,
            lock_timer: None,
        }
    }
}

struct DroneBehaviour {
    phase: f32,
    dest_x: i32,
    fs: DroneFiringState,
}

impl DroneBehaviour {
    pub fn new(pos: Vec2) -> Self {
        Self {
            phase: pos.x,
            dest_x: pos.x as i32,
            fs: DroneFiringState::Ready,
        }
    }

    pub fn update(world: &World, resources: &SceneResources, buffer: &mut CommandBuffer) {
        let floor_sensor_w;
        let floor_sensor_h;
        let bob_h;
        let bob_a;
        let lock_frames: u8;
        {
            let conf = config();
            floor_sensor_w = conf.drone_sensor_w();
            floor_sensor_h = conf.drone_sensor_h();
            bob_h = conf.drone_bob_h();
            bob_a = conf.drone_bob_a();
            lock_frames = conf.drone_lock_frames().try_into().unwrap();
        }
        let player_x = player_x(world, resources.player_id);
        let player_y = player_y(world, resources.player_id);
        for (id, (actor, beh, rect, spr)) in world
            .query::<(&mut Actor, &mut DroneBehaviour, &IntRect, &mut DroneSprite)>()
            .iter()
        {
            // this is a mess because our pub physics API doesn't expose what we want.
            // probably it should
            let left_lim = find_collision_pos(
                &IntRect {
                    x: rect.x - floor_sensor_w,
                    ..*rect
                },
                rect.x,
                rect.y,
                world,
                &resources.body_index,
            )
            .0;
            let right_lim = find_collision_pos(
                &IntRect {
                    x: rect.x + floor_sensor_w,
                    ..*rect
                },
                rect.x,
                rect.y,
                world,
                &resources.body_index,
            )
            .0;
            let targ_x = beh.dest_x.clamp(left_lim, right_lim) + rect.w / 2;
            if with_prob(0.01) {
                beh.dest_x = player_x.map(|x| x as i32).unwrap_or(beh.dest_x)
                    + quad_rand::gen_range(-100, 100);
            }
            let hover_h = (floor_sensor_h as f32 + beh.phase.cos() * bob_h).round() as i32;
            beh.phase += bob_a;
            let below_lim = find_collision_pos(
                &IntRect {
                    y: rect.y + hover_h + 16,
                    ..*rect
                },
                rect.x,
                rect.y,
                world,
                &resources.body_index,
            )
            .1;
            let targ_y = below_lim + rect.h / 2 - hover_h;
            let pos = rect.centre();
            actor.vx += (targ_x as f32 - pos.x - actor.vx).clamp(-10.0, 10.0) / 100.0;
            actor.vy += (targ_y as f32 - pos.y - actor.vy).clamp(-10.0, 10.0) / 100.0;
            beh.fs = match beh.fs {
                DroneFiringState::Delay(n) => {
                    if n == 0 {
                        DroneFiringState::Ready
                    } else {
                        DroneFiringState::Delay(n - 1)
                    }
                }
                DroneFiringState::Ready => {
                    if let (Some(px), Some(py)) = (player_x, player_y) {
                        spr.flipped_h = px < pos.x;
                        spr.flipped_v = py < pos.y;
                        let orig = rect.centre();
                        let dest = Vec2::new(px as f32, py as f32);
                        if ray_collision(&*world, &resources.body_index, &orig, &dest).is_none() {
                            let ret_id = world.reserve_entity();
                            let ret = Reticule::new(id, rect.centre());
                            buffer.insert_one(ret_id, ret);
                            DroneFiringState::Seeking(ret_id)
                        } else {
                            DroneFiringState::Delay(5)
                        }
                    } else {
                        DroneFiringState::Ready
                    }
                }
                DroneFiringState::Seeking(id) => {
                    if let (Some(px), Some(py)) = (player_x, player_y) {
                        let mut ret = world.get::<&mut Reticule>(id).unwrap();
                        match &ret.lock_timer {
                            Some(t) => {
                                if t.get() > lock_frames {
                                    let orig = rect.centre();
                                    let intended_dest = ret.pos;
                                    let max_d = (intended_dest - orig).length() + 300.0;
                                    let furthest_dest =
                                        orig + (intended_dest - orig).normalize_or_zero() * max_d;
                                    let dest = match ray_collision(
                                        &*world,
                                        &resources.body_index,
                                        &orig,
                                        &furthest_dest,
                                    ) {
                                        None => furthest_dest,
                                        Some((v, _)) => v,
                                    };
                                    if let Ok(mut q) = world
                                        .query_one::<(&mut Controller, &IntRect)>(
                                            resources.player_id,
                                        )
                                    {
                                        if let Some((player, p_rect)) = q.get() {
                                            if railgun_intersects(
                                                &RailgunHitbox::new(orig.x, orig.y, dest.x, dest.y),
                                                p_rect,
                                            ) {
                                                player.hurt()
                                            }
                                        }
                                    }
                                    create_smoke_cloud(buffer, orig.x as i32, orig.y as i32, 8.0);
                                    make_railgun_trail(buffer, orig.x, orig.y, dest.x, dest.y);
                                    buffer.despawn(id);
                                    DroneFiringState::Delay(20)
                                } else {
                                    ret.lock_timer = t.checked_add(1);
                                    DroneFiringState::Seeking(id)
                                }
                            }
                            None => {
                                let p = Vec2::new(px as f32, py as f32);
                                let d = p - ret.pos;
                                if d.length() < 8.0 {
                                    ret.lock_timer = NonZeroU8::new(1);
                                } else {
                                    ret.pos = ret.pos + d.normalize_or_zero() * 3.0;
                                }
                                DroneFiringState::Seeking(id)
                            }
                        }
                    } else {
                        buffer.despawn(id);
                        DroneFiringState::Ready
                    }
                }
            };
            spr.frame = (spr.frame + 1) % 4;
        }

        for (id, ret) in world.query::<&Reticule>().iter() {
            if !world.contains(ret.parent) {
                buffer.despawn(id)
            }
        }
    }
}

pub struct ParrotBossBehaviour {
    pub heads: [Entity; 6], // pub so we can draw the necks
    pub feet: [Entity; 4],  // pub so we can find the feet to draw the legs
    foot_timer: u8,
    current_foot: i8,
    cycle_dir: i8,
    foot_dx: f32,
    foot_dy: f32,
    angle: f32,
    angle_v: f32,
    spin_timer: u8,
}

fn head_offset(head_index: i8, angle: f32) -> (i32, i32) {
    let a = angle + std::f32::consts::TAU / 6.0 * (head_index as f32);
    (
        (a.cos() * 16.0).round() as i32 + 24,
        ((a.sin() * 16.0).round() * FRAC_2_SQRT_3) as i32 - 16,
    )
}

fn foot_offset(foot_index: i8) -> i32 {
    foot_index as i32 * 32 - 48
}

pub fn add_boss(world: &mut World, x: i32, y: i32) {
    let mut heads_vec = Vec::new();
    for i in 0i8..6 {
        let (hx, hy) = head_offset(i, 0.0);
        let rect = IntRect::new(hx + x, hy + y - 12, 16, 12);
        let spr = crate::draw::ParrotHeadSprite::new();
        let hp = 7;
        let hittable = EnemyHittable::new(hp);
        let dmg = EnemyContactDamage::new();
        heads_vec.push(world.spawn((rect, spr, hittable, dmg)));
    }
    let mut feet_vec = Vec::new();
    for i in 0i8..4 {
        let fx = x + foot_offset(i);
        let rect = IntRect::new(fx - 4, y - 16, 8, 16);
        let actor = Actor::new(&rect, PhysicsCoeffs::Actor);
        let hp = 20;
        let hittable = EnemyHittable::new(hp);
        let dmg = EnemyContactDamage::new();
        feet_vec.push(world.spawn((
            rect, //crate::draw::ColorRect::new(GREEN),
            actor, hittable, dmg,
        )));
    }

    let boss = ParrotBossBehaviour {
        heads: [
            heads_vec[0],
            heads_vec[1],
            heads_vec[2],
            heads_vec[3],
            heads_vec[4],
            heads_vec[5],
        ],
        feet: [feet_vec[0], feet_vec[1], feet_vec[2], feet_vec[3]],
        foot_timer: 0,
        current_foot: 0,
        cycle_dir: 1,
        foot_dx: 0.0,
        foot_dy: 0.0,
        angle: 0.0,
        angle_v: 0.0,
        spin_timer: 0,
    };
    let rect = IntRect::new(x - 32, y - 20, 64, 20);
    let actor = Actor::new(&rect, PhysicsCoeffs::Flyer);
    let hp = 20;
    let hittable = EnemyHittable::new(hp);
    let dmg = EnemyContactDamage::new();
    world.spawn((boss, rect, actor, hittable, dmg));
}

impl ParrotBossBehaviour {
    fn update(world: &World, resources: &SceneResources) {
        for (_, (actor, beh, rect)) in world
            .query::<(&mut Actor, &mut ParrotBossBehaviour, &IntRect)>()
            .iter()
        {
            let c = rect.centre();
            let target_x = match player_x(world, resources.player_id) {
                Some(x) => x.min(c.x + 10.0).max(c.x - 10.0),
                None => c.x,
            };
            beh.cycle_dir = if c.x > target_x { -1 } else { 1 };
            // make sure we don't change cycle_dir in mid spin if it will mess stuff up
            beh.spin_timer += 1;
            if beh.spin_timer < 10 {
                beh.angle_v += 0.1 * beh.cycle_dir as f32;
            } else if beh.spin_timer < 40 {
                beh.angle_v *= 0.9;
            } else if beh.spin_timer < 120 {
                let pi_by_3 = std::f32::consts::PI / 3.0;
                let snap = (beh.angle / pi_by_3).round() * pi_by_3;
                beh.angle += (snap - beh.angle) * 0.2;
                beh.angle_v *= 0.9;
            } else {
                beh.spin_timer = 0;
            }
            beh.angle += beh.angle_v;
            for (idx, h_id) in beh.heads.iter().enumerate() {
                let mut f_rect = world.get::<&mut IntRect>(*h_id).unwrap();
                let (hx, hy) = head_offset(idx as i8, beh.angle);
                f_rect.x = rect.x + hx;
                f_rect.y = rect.y + hy;
            }
            let mut fx = 0.0;
            let mut fy = 0.0;
            beh.foot_timer = beh.foot_timer.saturating_add(1);
            if beh.foot_timer > 30 {
                let next_foot = (beh.current_foot + beh.cycle_dir).rem_euclid(4);
                let ent = world.entity(beh.feet[next_foot as usize]).unwrap();
                let mut nfa = ent.get::<&mut Actor>().unwrap();
                let nfar = ent.get::<&IntRect>().unwrap();
                if nfa.grounded {
                    nfa.coeffs = PhysicsCoeffs::Static;
                    let target_pos = target_x + foot_offset(next_foot) as f32;
                    beh.foot_dx = (target_pos - nfar.centre().x) / std::f32::consts::PI / 4.0;
                    beh.foot_dy = -2.0;
                    if beh.foot_dx.abs() < 0.2 {
                        beh.foot_dx = 0.0;
                        beh.foot_dy = 0.0;
                    }
                    drop(nfa);
                    drop(nfar);
                    let mut cfa = world
                        .get::<&mut Actor>(beh.feet[beh.current_foot as usize])
                        .unwrap();
                    cfa.coeffs = PhysicsCoeffs::Actor;
                    beh.foot_timer = 0;
                    beh.current_foot = next_foot;
                }
            }
            {
                let mut f_actor = world
                    .get::<&mut Actor>(beh.feet[beh.current_foot as usize])
                    .unwrap();
                let a = beh.foot_timer as f32 / 30.0 * std::f32::consts::PI;
                f_actor.vx = a.sin() * beh.foot_dx;
                f_actor.vy = a.cos() * beh.foot_dy;
            }
            for f_id in beh.feet {
                let f_rect = world.get::<&IntRect>(f_id).unwrap();
                let c = f_rect.centre();
                fx += c.x;
                fy += c.y;
            }
            let targ_x = fx / 4.0;
            let targ_y = fy / 4.0 - 84.0;
            let rc = rect.centre();
            actor.vx = actor.vx + ((targ_x - rc.x) / 3.0).max(-4.0).min(4.0);
            actor.vy = actor.vy + ((targ_y - rc.y) / 3.0).max(-4.0).min(4.0);
            actor.vx = if actor.vx.abs() < 0.5 {
                0.0
            } else {
                actor.vx * 0.9
            };
            actor.vy = if actor.vy.abs() < 1.0 {
                0.0
            } else {
                actor.vy * 0.9
            };
        }
    }
}

pub fn update_enemies(resources: &mut SceneResources, buffer: &mut CommandBuffer) {
    let world = resources.world_ref.lock().unwrap();
    DogBehaviour::update(&world, resources);
    ParrotBehaviour::update(&world, resources, buffer);
    DroneBehaviour::update(&world, resources, buffer);
    ParrotBossBehaviour::update(&world, resources);

    for (id, (actor, rect, kind, hittable)) in world
        .query::<(&Actor, &IntRect, &EnemyKind, &mut EnemyHittable)>()
        .iter()
    {
        hittable.was_hit = false;
        if hittable.hp == 0 || actor.crushed {
            match kind {
                EnemyKind::Dog | EnemyKind::JumpyDog => {
                    resources.messages.add("Destroyed a hound.".to_owned())
                }
                EnemyKind::SpiderParrot(ParrotKind::Laser) => resources
                    .messages
                    .add("Destroyed a red scuttler.".to_owned()),
                EnemyKind::SpiderParrot(ParrotKind::Cannon) => resources
                    .messages
                    .add("Destroyed a green scuttler.".to_owned()),
                EnemyKind::Drone => resources.messages.add("Destroyed a drone.".to_owned()),
            }
            buffer.despawn(id);
            let (ex, ey) = rect.centre_int();
            create_explosion(buffer, ex, ey);
            resources.stats.kills += 1
        }
    }

    if let Ok(mut q) = world.query_one::<(&mut Controller, &IntRect)>(resources.player_id) {
        if let Some((c, p_rect)) = q.get() {
            for (_, (_, rect)) in world.query::<(&EnemyContactDamage, &IntRect)>().iter() {
                if rect.intersects(p_rect) {
                    c.hurt();
                    break; // player will get damage invulnerability so might as well stop
                }
            }
        }
    };
}
