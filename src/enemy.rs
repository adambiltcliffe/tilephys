use std::cmp::Ordering;

use crate::config::config;
use crate::draw::{DogSprite, ParrotSprite};
use crate::physics::{collide_any, Actor, IntRect};
use crate::player::Controller;
use crate::projectile::{make_enemy_fireball, make_enemy_laser, make_railgun_hitbox};
use crate::ray::ray_collision;
use crate::resources::SceneResources;
use crate::vfx::{create_explosion, make_railgun_trail};
use hecs::{CommandBuffer, Entity, World};
use macroquad::prelude::*;

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum EnemyKind {
    Dog,
    JumpyDog,
    SpiderParrot(ParrotKind),
    Drone,
}

pub fn add_enemy(world: &mut World, kind: EnemyKind, x: i32, y: i32) {
    let (w, h) = match kind {
        EnemyKind::Drone => (16, 16),
        EnemyKind::SpiderParrot(_) => (24, 24),
        EnemyKind::Dog | EnemyKind::JumpyDog => (24, 16),
    };
    let rect = IntRect::new(x - w / 2, y - h, w, h);
    let actor = Actor::new(&rect, 0.4);
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
            DroneBehaviour::new(),
            rect,
            crate::draw::ColorRect::new(macroquad::color::BEIGE),
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
    world
        .get::<&IntRect>(player_id)
        .map(|rect| rect.centre().x)
        .ok()
}

fn player_y(world: &World, player_id: Entity) -> Option<f32> {
    world
        .get::<&IntRect>(player_id)
        .map(|rect| rect.centre().y)
        .ok()
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

struct Reticule {
    x: i32,
    y: i32,
}

struct DroneBehaviour {
    thrust_x: f32,
    thrust_y: f32,
    thrust_lock: u8,
    fs: DroneFiringState,
}

impl DroneBehaviour {
    pub fn new() -> Self {
        Self {
            thrust_x: 0.0,
            thrust_y: 0.0,
            thrust_lock: 0,
            fs: DroneFiringState::Ready,
        }
    }

    pub fn update(world: &World, resources: &SceneResources, buffer: &mut CommandBuffer) {
        let antigravity;
        let airbrake;
        let thrust_mag;
        let thrust_lock;
        let x_factor;
        let floor_sensor_w;
        let floor_sensor_h;
        let da;
        {
            let conf = config();
            antigravity = conf.drone_antigravity();
            airbrake = conf.drone_airbrake();
            thrust_mag = conf.drone_thrust();
            thrust_lock = conf.drone_thrust_lock() as u8;
            x_factor = conf.drone_thrust_x_factor();
            floor_sensor_w = conf.drone_sensor_w();
            floor_sensor_h = conf.drone_sensor_h();
            da = conf.drone_thrust_max_angle();
        }
        let player_x = player_x(world, resources.player_id);
        let player_y = player_y(world, resources.player_id);
        for (_, (actor, beh, rect)) in world
            .query::<(&mut Actor, &mut DroneBehaviour, &IntRect)>()
            .iter()
        {
            let below_rect = IntRect::new(
                rect.x - (floor_sensor_w - rect.w) / 2,
                rect.y + rect.h,
                floor_sensor_w,
                floor_sensor_h,
            );
            let too_low = collide_any(world, &resources.body_index, &below_rect);
            if too_low && actor.vy > 0.0 {
                actor.vy *= 1.0 - airbrake;
            }
            if beh.thrust_lock == 0 {
                if too_low {
                    let a = -std::f32::consts::FRAC_PI_2 + quad_rand::gen_range(-da, da);
                    beh.thrust_x = a.cos() * thrust_mag * x_factor;
                    beh.thrust_y = a.sin() * thrust_mag;
                    beh.thrust_lock = thrust_lock;
                } else {
                    beh.thrust_x = 0.0;
                    beh.thrust_y = 0.0;
                    beh.thrust_lock = thrust_lock;
                }
            } else {
                beh.thrust_lock -= 1
            };
            actor.vx += beh.thrust_x;
            actor.vy += beh.thrust_y - antigravity;
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
                        let orig =
                            Vec2::new((rect.x + rect.w / 2) as f32, (rect.y + rect.h / 2) as f32);
                        let intended_dest = Vec2::new(px as f32, py as f32);
                        if ray_collision(&*world, &resources.body_index, &orig, &intended_dest)
                            .is_none()
                        {
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
                            //make_railgun_hitbox(buffer, orig.x, orig.y, dest.x, dest.y);
                            make_railgun_trail(buffer, orig.x, orig.y, dest.x, dest.y);
                            DroneFiringState::Delay(20)
                        } else {
                            DroneFiringState::Delay(5)
                        }
                    } else {
                        DroneFiringState::Ready
                    }
                }
                DroneFiringState::Seeking(_) => DroneFiringState::Ready,
            }
        }
    }
}

pub fn update_enemies(resources: &mut SceneResources, buffer: &mut CommandBuffer) {
    let world = resources.world_ref.lock().unwrap();
    DogBehaviour::update(&world, resources);
    ParrotBehaviour::update(&world, resources, buffer);
    DroneBehaviour::update(&world, resources, buffer);

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
