use hecs::{CommandBuffer, World};

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
    pub fn new_from_centre(x: i32, y: i32, a: f32) -> Self {
        let d = quad_rand::gen_range(0.0, 6.0);
        Self {
            x: x as f32 + a.cos() * d,
            y: y as f32 + a.sin() * d,
            vx: a.cos() * 1.0,
            vy: a.sin() * 1.0,
            r: quad_rand::gen_range(8.0, 16.0),
        }
    }
}

pub fn create_explosion(buffer: &mut CommandBuffer, x: i32, y: i32) {
    let mut a = quad_rand::gen_range(0.0, std::f32::consts::TAU);
    buffer.spawn((Explosion::new_from_centre(x, y),));
    a += std::f32::consts::TAU / std::f32::consts::E;
    for _ in 0..6 {
        buffer.spawn((FireParticle::new_from_centre(x, y, a),));
        buffer.spawn((SmokeParticle::new_from_centre(x, y, a),));
        a += std::f32::consts::TAU / std::f32::consts::E;
    }
}

pub fn update_vfx(world: &World, buffer: &mut CommandBuffer) {
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
}
