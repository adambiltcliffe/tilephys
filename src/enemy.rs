use std::cmp::Ordering;

use crate::draw::{DogSprite, ParrotSprite};
use crate::physics::{collide_any, Actor, IntRect};
use crate::player::Controller;
use crate::projectile::make_enemy_projectile;
use crate::resources::SceneResources;
use crate::vfx::create_explosion;
use hecs::{CommandBuffer, Entity, World};
use macroquad::prelude::*;

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum EnemyKind {
    Dog,
    JumpyDog,
    SpiderParrot,
}

pub fn add_enemy(world: &mut World, kind: EnemyKind, x: i32, y: i32) {
    let h = match kind {
        EnemyKind::SpiderParrot => 24,
        _ => 16,
    };
    let rect = IntRect::new(x - 12, y - h, 24, h);
    let actor = Actor::new(&rect, 0.4);
    let hittable = EnemyHittable::new(3);
    let dmg = EnemyContactDamage::new();
    if kind == EnemyKind::SpiderParrot {
        world.spawn((
            kind,
            ParrotBehaviour::new(),
            rect,
            crate::draw::ParrotSprite::new(),
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
enum ParrotState {
    Wait,
    Move,
    Attack,
    Fall,
}

struct ParrotBehaviour {
    state: ParrotState,
    state_timer: u8,
    facing: i8,
}

impl ParrotBehaviour {
    pub fn new() -> Self {
        Self {
            state: ParrotState::Wait,
            state_timer: 0,
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
                            if is_facing_player && with_prob(0.7) {
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
                    spr.frame = 3;
                    spr.muzzle_flash = Some(beh.state_timer % 6);
                    if beh.state_timer % 6 == 1 {
                        actor.vx -= beh.facing as f32 * 10.0;
                        let new_x = rect.x + 7 + beh.facing as i32 * 6;
                        let rect = IntRect::new(new_x, rect.y + 8, 8, 5);
                        make_enemy_projectile(buffer, rect, beh.facing as f32 * 4.0);
                    }
                    if beh.state_timer % 6 == 5
                        && parrot_off_edge(world, resources, rect, beh.facing)
                    {
                        beh.set_state(ParrotState::Move);
                    } else if beh.state_timer >= 24 {
                        beh.set_state(ParrotState::Wait);
                    }
                }
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

pub fn update_enemies(resources: &mut SceneResources, buffer: &mut CommandBuffer) {
    let world = resources.world_ref.lock().unwrap();
    DogBehaviour::update(&world, resources);
    ParrotBehaviour::update(&world, resources, buffer);

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
                EnemyKind::SpiderParrot => resources
                    .messages
                    .add("Destroyed a red scuttler.".to_owned()),
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
