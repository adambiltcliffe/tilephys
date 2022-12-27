use crate::draw::{DogSprite, ParrotSprite};
use crate::physics::{Actor, IntRect};
use crate::player::Controller;
use crate::resources::Resources;
use hecs::{Entity, World};
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
}

impl EnemyHittable {
    pub fn new(hp: u16) -> Self {
        Self { hp }
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

    pub fn update(world: &World, resources: &Resources) {
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
                if player_x.is_some() && with_prob(0.7) {
                    enemy.dir = (player_x.unwrap() - rect.centre().x).signum() * 5.0;
                } else {
                    enemy.dir = 5.0 * rand_sign();
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

    pub fn update(world: &World, resources: &Resources) {
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

            beh.state_timer += 1;
            let change = quad_rand::gen_range(10, 20) < beh.state_timer;

            match beh.state {
                ParrotState::Fall => {
                    spr.frame = 0;
                    if actor.grounded {
                        beh.set_state(ParrotState::Wait);
                    }
                }
                ParrotState::Wait => {
                    spr.frame = 0;
                    if change {
                        if with_prob(0.5) {
                            beh.facing = -beh.facing;
                            beh.state_timer = 0;
                        } else {
                            beh.set_state(ParrotState::Move);
                        }
                    }
                }
                ParrotState::Move => {
                    spr.frame = (beh.state_timer / 2) % 2;
                    if change {
                        beh.set_state(ParrotState::Wait);
                    } else {
                        actor.vx += 5.0 * beh.facing as f32;
                    }
                }
                ParrotState::Attack => (),
            }

            if beh.facing < 0 {
                spr.flipped = false
            } else {
                spr.flipped = true
            }
        }
    }
}

pub fn update_enemies(world: &World, resources: &Resources) {
    DogBehaviour::update(world, resources);
    ParrotBehaviour::update(world, resources);

    for (_, (_, rect)) in world.query::<(&EnemyContactDamage, &IntRect)>().iter() {
        if let Ok(mut q) = world.query_one::<(&mut Controller, &IntRect)>(resources.player_id) {
            if let Some((c, p_rect)) = q.get() {
                if rect.intersects(p_rect) {
                    c.hurt();
                }
            }
        }
    }
}
