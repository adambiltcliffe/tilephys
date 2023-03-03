use crate::{config::config, physics::IntRect, resources::SceneResources};
use hecs::{CommandBuffer, World};
use macroquad::prelude::*;
use std::cmp::Ordering;

const EXPLOSION_OUTER_COLOR: Color = Color {
    r: 0.1333,
    g: 1.0,
    b: 0.0,
    a: 1.0,
};

const EXPLOSION_INNER_COLOR: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 0.3382,
    a: 1.0,
};

const EXPLOSION_SMOKE_COLOR: Color = Color {
    r: 0.2275,
    g: 0.2275,
    b: 0.2275,
    a: 1.0,
};

pub struct ZapFlash {
    pub x: i32,
    pub y: i32,
    pub n: u32,
}

impl ZapFlash {
    pub fn new_from_centre(x: i32, y: i32) -> Self {
        Self {
            x: x - 4,
            y: y - 4,
            n: 0,
        }
    }
}

pub struct Explosion {
    pub x: i32,
    pub y: i32,
    pub n: i32,
}

impl Explosion {
    pub fn new_from_centre(x: i32, y: i32) -> Self {
        Self {
            x: x - 12,
            y: y - 12,
            n: 0,
        }
    }
}

pub struct FireParticle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub r: f32,
}

impl FireParticle {
    pub fn new_from_centre(x: i32, y: i32, a: f32) -> Self {
        let d = quad_rand::gen_range(0.0, 4.0);
        Self {
            x: x as f32 + a.cos() * d,
            y: y as f32 + a.sin() * d,
            vx: a.cos() * 4.0,
            vy: a.sin() * 4.0,
            r: quad_rand::gen_range(8.0, 16.0),
        }
    }
}

pub struct SmokeParticle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub r: f32,
}

impl SmokeParticle {
    pub fn new_from_centre(x: i32, y: i32, a: f32, r: f32) -> Self {
        let d = quad_rand::gen_range(0.0, 6.0);
        Self {
            x: x as f32 + a.cos() * d,
            y: y as f32 + a.sin() * d,
            vx: a.cos() * 1.0,
            vy: a.sin() * 1.0,
            r: quad_rand::gen_range(r / 2.0, r),
        }
    }
}

pub fn create_explosion(buffer: &mut CommandBuffer, x: i32, y: i32) {
    let mut a = quad_rand::gen_range(0.0, std::f32::consts::TAU);
    buffer.spawn((Explosion::new_from_centre(x, y),));
    a += std::f32::consts::TAU / std::f32::consts::E;
    for _ in 0..6 {
        buffer.spawn((FireParticle::new_from_centre(x, y, a),));
        buffer.spawn((SmokeParticle::new_from_centre(x, y, a, 16.0),));
        a += std::f32::consts::TAU / std::f32::consts::E;
    }
}

pub struct FireballEffect {
    pub r: f32,
    pub t: f32,
}

impl FireballEffect {
    pub fn new(r: f32) -> Self {
        Self { r, t: 0.0 }
    }
}

pub struct RailgunTrail {
    x1: f32,
    x2: f32,
    y: f32,
    n: u8,
}

impl RailgunTrail {
    pub fn new(x1: f32, x2: f32, y: f32) -> Self {
        Self { x1, x2, y, n: 0 }
    }
}

pub fn make_railgun_trail(buffer: &mut CommandBuffer, x1: i32, x2: i32, y: i32) {
    let (sp, da, r) = {
        let cfg = config();
        (cfg.rg_smoke_sp(), cfg.rg_smoke_da(), cfg.rg_smoke_r())
    };
    buffer.spawn((RailgunTrail::new(x1 as f32, x2 as f32, y as f32),));
    let n = (x2 - x1).abs() / sp;
    let d = (x2 - x1).signum() * sp;
    let mut a = -std::f32::consts::PI / 2.0;
    for i in 0..n {
        // check if hecs has a spawn_multi for this or something
        buffer.spawn((SmokeParticle::new_from_centre(x1 + i * d, y, a, r),));
        a += da;
    }
}

pub fn update_vfx(resources: &SceneResources, buffer: &mut CommandBuffer) {
    let world = resources.world_ref.lock().unwrap();
    for (id, z) in world.query::<&mut ZapFlash>().iter() {
        z.n += 1;
        if z.n > 5 {
            buffer.despawn(id);
        }
    }
    for (id, z) in world.query::<&mut Explosion>().iter() {
        z.n += 1;
        if z.n > 6 {
            buffer.despawn(id);
        }
    }
    for (id, f) in world.query::<&mut FireParticle>().iter() {
        f.x += f.vx;
        f.y += f.vy;
        f.r -= 2.0;
        if f.r <= 0.0 {
            buffer.despawn(id);
        }
    }
    for (id, f) in world.query::<&mut SmokeParticle>().iter() {
        f.x += f.vx;
        f.y += f.vy;
        f.vx += quad_rand::gen_range(-0.1, 0.1);
        f.vy += quad_rand::gen_range(-0.15, 0.05);
        f.r *= 0.875;
        if f.r < 1.0 {
            buffer.despawn(id);
        }
    }
    for (_id, (rect, f)) in world.query::<(&IntRect, &mut FireballEffect)>().iter() {
        f.t += 0.25;
        let c = rect.centre();
        let a = quad_rand::gen_range(0.0, std::f32::consts::TAU);
        buffer.spawn((SmokeParticle::new_from_centre(
            c.x as i32,
            c.y as i32,
            a,
            f.r * 0.75,
        ),));
    }
    let rgf = config().rg_frames();
    for (id, t) in world.query::<&mut RailgunTrail>().iter() {
        t.n += 1;
        if t.n as i32 > rgf {
            buffer.despawn(id);
        }
    }
}

pub fn draw_vfx(world: &World) {
    let thick = config().rg_thickness();
    for (_, t) in world.query::<&RailgunTrail>().iter() {
        draw_line(t.x1, t.y, t.x2, t.y, thick, EXPLOSION_INNER_COLOR);
    }
    for (_, fp) in world.query::<&SmokeParticle>().iter() {
        draw_circle(fp.x, fp.y, fp.r, EXPLOSION_SMOKE_COLOR);
    }
    for (_, (rect, fb)) in world.query::<(&IntRect, &FireballEffect)>().iter() {
        let c = rect.centre();
        let r = fb.r * (1.0 - 0.5 * fb.t.cos().powi(5).abs());
        draw_circle(c.x, c.y, r, EXPLOSION_OUTER_COLOR);
        draw_circle(c.x, c.y, r * 0.75, EXPLOSION_INNER_COLOR);
    }
    for (_, fp) in world.query::<&FireParticle>().iter() {
        draw_circle(fp.x, fp.y, fp.r, EXPLOSION_OUTER_COLOR);
    }
    for (_, fp) in world.query::<&FireParticle>().iter() {
        draw_circle(fp.x, fp.y, fp.r * 0.75, EXPLOSION_INNER_COLOR);
    }
    for (_, ex) in world.query::<&Explosion>().iter() {
        match ex.n.cmp(&0) {
            Ordering::Less => (),
            Ordering::Equal => {
                draw_circle(ex.x as f32 + 12.0, ex.y as f32 + 12.0, 24.0, WHITE);
            }
            Ordering::Greater => {
                draw_circle_lines(
                    ex.x as f32 + 12.0,
                    ex.y as f32 + 12.0,
                    4.0 * ex.n as f32,
                    2.0,
                    WHITE,
                );
            }
        }
    }
}
