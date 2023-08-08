use crate::config::config;
use crate::draw::ZapSprite;
use crate::enemy::EnemyHittable;
use crate::physics::{collide_any, find_collision_pos, IntRect};
use crate::player::Controller;
use crate::resources::SceneResources;
use crate::vfx::FireballEffect;
use crate::vfx::ZapFlash;
use crate::vfx::{Explosion, SmokeParticle};
use hecs::CommandBuffer;
use std::cmp::Ordering;

pub struct DamageEnemies {}
pub struct DamagePlayer {}
pub struct LaserImpact {}
pub struct FireballSplit {}
pub struct ProjectileGravity {}
pub struct ProjectileDrag {}
pub struct ToxicSmoke {}
pub struct Napalm {}

impl ToxicSmoke {
    fn new() -> Self {
        Self {}
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

    pub fn update(resources: &mut SceneResources, buffer: &mut CommandBuffer) {
        let world = resources.world_ref.lock().unwrap();
        for (e, (proj, rect)) in world.query::<(&mut Projectile, &mut IntRect)>().iter() {
            let ox = rect.x;
            let oy = rect.y;
            proj.prec_x += proj.vx;
            proj.prec_y += proj.vy;
            rect.x = proj.prec_x.round() as i32;
            rect.y = proj.prec_y.round() as i32;
            if collide_any(&world, &resources.body_index, rect) {
                buffer.despawn(e);
                if world.satisfies::<&LaserImpact>(e).unwrap_or(false) {
                    let (x, y) = find_collision_pos(rect, ox, oy, &world, &resources.body_index);
                    let sx = if proj.vx > 0.0 { x + rect.w - 1 } else { x };
                    buffer.spawn((ZapFlash::new_from_centre(sx, y + 2),));
                }
                if world.satisfies::<&FireballSplit>(e).unwrap_or(false) {
                    let (x, y) = find_collision_pos(rect, ox, oy, &world, &resources.body_index);
                    spawn_mini_fireballs(buffer, x + 8, y + 8);
                }
            }
        }
        for (e, (proj, rect, _)) in world
            .query::<(&mut Projectile, &mut IntRect, &DamageEnemies)>()
            .iter()
        {
            let mut live = true;
            for (_, (en, e_rect)) in world.query::<(&mut EnemyHittable, &IntRect)>().iter() {
                if live && en.hp > 0 && rect.intersects(e_rect) {
                    buffer.despawn(e);
                    if world.satisfies::<&LaserImpact>(e).unwrap_or(false) {
                        let sx = if proj.vx > 0.0 {
                            rect.x + rect.w - 1
                        } else {
                            rect.x
                        };
                        buffer.spawn((ZapFlash::new_from_centre(sx, rect.y + 2),));
                    }
                    if world.satisfies::<&FireballSplit>(e).unwrap_or(false) {
                        spawn_mini_fireballs(buffer, rect.x + 8, rect.y + 8);
                    }
                    en.hurt(1);
                    live = false;
                }
            }
        }

        if let Ok(mut q) = world.query_one::<(&mut Controller, &IntRect)>(resources.player_id) {
            if let Some((c, p_rect)) = q.get() {
                if c.can_hurt() {
                    for (id, (proj, rect, _)) in world
                        .query::<(&mut Projectile, &mut IntRect, &DamagePlayer)>()
                        .iter()
                    {
                        if rect.intersects(p_rect) {
                            c.hurt();
                            buffer.despawn(id);
                            if world.satisfies::<&LaserImpact>(id).unwrap_or(false) {
                                let sx = if proj.vx > 0.0 {
                                    rect.x + rect.w - 1
                                } else {
                                    rect.x
                                };
                                buffer.spawn((ZapFlash::new_from_centre(sx, rect.y + 2),));
                            }
                            if world.satisfies::<&FireballSplit>(id).unwrap_or(false) {
                                spawn_mini_fireballs(buffer, rect.x + 8, rect.y + 8);
                            }
                        }
                    }
                }
            };
        }

        for (_, (proj, _)) in world
            .query::<(&mut Projectile, &ProjectileGravity)>()
            .iter()
        {
            proj.vy += 0.2;
        }
        for (id, (proj, _)) in world.query::<(&mut Projectile, &ProjectileDrag)>().iter() {
            proj.vx *= 0.8;
            proj.vy *= 0.8;
            if proj.vx.abs() < 2.0 {
                buffer.despawn(id);
            }
        }
        for (_, (proj, rect, _smoke)) in world
            .query::<(&mut Projectile, &IntRect, &ToxicSmoke)>()
            .iter()
        {
            proj.vy -= 0.2;
            proj.vx = proj.vx * 0.9 + quad_rand::gen_range(-0.8, 0.8);
            buffer.spawn((SmokeParticle::new_from_centre(
                rect.x + 4,
                rect.y + 2,
                std::f32::consts::PI / -2.0 + quad_rand::gen_range(-0.3, 0.3),
                8.0,
            ),));
        }

        let g = config().flamer_g();
        for (_, (proj, _)) in world.query::<(&mut Projectile, &Napalm)>().iter() {
            proj.vy += g;
        }

        // process railgun collisions with enemies
        let damage = config().rg_damage();
        for (id, hb) in world.query::<&RailgunHitbox>().iter() {
            for (_, (en, e_rect)) in world.query::<(&mut EnemyHittable, &IntRect)>().iter() {
                if en.hp > 0 && railgun_intersects(hb, e_rect) {
                    en.hurt(damage as u16);
                }
            }
            buffer.despawn(id);
        }
    }
}

fn spawn_mini_fireballs(buffer: &mut CommandBuffer, x: i32, y: i32) {
    buffer.spawn((Explosion::new_from_centre(x, y),));
    let mut a = quad_rand::gen_range(0.0, std::f32::consts::TAU);
    a += std::f32::consts::TAU / std::f32::consts::E;
    let rect = IntRect::new(x - 4, y - 4, 8, 8);
    for _ in 0..6 {
        make_enemy_fireball(buffer, rect.clone(), a.cos() * 2.0, a.sin() * 2.0, false);
        a += std::f32::consts::TAU / std::f32::consts::E;
    }
}

pub fn make_player_laser(buffer: &mut CommandBuffer, rect: IntRect, vx: f32, vy: f32) {
    let proj = Projectile::new(&rect, vx, vy);
    buffer.spawn((
        rect,
        ZapSprite::new(),
        proj,
        DamageEnemies {},
        LaserImpact {},
    ));
}

pub fn make_enemy_laser(buffer: &mut CommandBuffer, rect: IntRect, vx: f32) {
    let proj = Projectile::new(&rect, vx, 0.0);
    buffer.spawn((
        rect,
        ZapSprite::new(),
        proj,
        DamagePlayer {},
        LaserImpact {},
    ));
}

pub fn make_player_fireball(buffer: &mut CommandBuffer, rect: IntRect, vx: f32, vy: f32) {
    let proj = Projectile::new(&rect, vx, vy);
    buffer.spawn((rect, FireballEffect::new(4.0), proj, DamageEnemies {}));
}

pub fn make_enemy_fireball(
    buffer: &mut CommandBuffer,
    rect: IntRect,
    vx: f32,
    vy: f32,
    split: bool,
) {
    let proj = Projectile::new(&rect, vx, vy);
    if split {
        buffer.spawn((
            rect,
            FireballEffect::new(8.0),
            proj,
            DamagePlayer {},
            FireballSplit {},
        ));
    } else {
        buffer.spawn((
            rect,
            FireballEffect::new(4.0),
            proj,
            DamagePlayer {},
            ProjectileGravity {},
        ));
    }
}

pub fn make_smoke(buffer: &mut CommandBuffer, rect: IntRect, vx: f32, vy: f32) {
    let proj = Projectile::new(&rect, vx, vy);
    buffer.spawn((
        rect,
        proj,
        FireballEffect::new(4.0),
        DamageEnemies {},
        ToxicSmoke::new(),
    ));
}

pub fn make_napalm(buffer: &mut CommandBuffer, rect: IntRect, vx: f32, vy: f32, real: bool) {
    let proj = Projectile::new(&rect, vx, vy);
    if real {
        buffer.spawn((
            rect,
            proj,
            FireballEffect::new(4.0),
            DamageEnemies {},
            Napalm {},
        ));
    } else {
        buffer.spawn((rect, proj, FireballEffect::new(4.0), Napalm {}));
    }
}

pub struct RailgunHitbox {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
}

impl RailgunHitbox {
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }
}

pub fn make_railgun_hitbox(buffer: &mut CommandBuffer, x1: f32, y1: f32, x2: f32, y2: f32) {
    buffer.spawn((RailgunHitbox::new(x1, y1, x2, y2),))
}

pub fn railgun_intersects(hb: &RailgunHitbox, rect: &IntRect) -> bool {
    let mut t_min = 0.0f32;
    let mut t_max = 1.0f32;
    let w = hb.x2 - hb.x1;
    let h = hb.y2 - hb.y1;
    match w.total_cmp(&0.0) {
        Ordering::Greater => {
            t_min = t_min.max((rect.x as f32 - hb.x1) / w);
            t_max = t_max.min(((rect.x + rect.w) as f32 - hb.x1) / w);
        }
        Ordering::Equal => {
            if hb.x1 < rect.x as f32 || hb.x1 > (rect.x + rect.w) as f32 {
                return false;
            }
        }
        Ordering::Less => {
            t_min = t_min.max(((rect.x + rect.w) as f32 - hb.x1) / w);
            t_max = t_max.min((rect.x as f32 - hb.x1) / w);
        }
    }
    match h.total_cmp(&0.0) {
        Ordering::Greater => {
            t_min = t_min.max((rect.y as f32 - hb.y1) / h);
            t_max = t_max.min(((rect.y + rect.h) as f32 - hb.y1) / h);
        }
        Ordering::Equal => {
            if hb.y1 < rect.y as f32 || hb.y1 > (rect.y + rect.h) as f32 {
                return false;
            }
        }
        Ordering::Less => {
            t_min = t_min.max(((rect.y + rect.h) as f32 - hb.y1) / h);
            t_max = t_max.min((rect.y as f32 - hb.y1) / h);
        }
    }
    t_min <= t_max
}
