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
    pub fn new_from_centre(x: i32, y: i32, n: i32) -> Self {
        Self {
            x: x - 12,
            y: y - 12,
            n,
        }
    }
}

pub fn create_explosions(buffer: &mut CommandBuffer, x: i32, y: i32, n: i32) {
    let mut a = quad_rand::gen_range(0.0, std::f32::consts::TAU);
    for i in 0_..n {
        let r = quad_rand::gen_range(0.0, 5.0 * i as f32);
        buffer.spawn((Explosion::new_from_centre(
            x + (r * a.cos()) as i32,
            y + (r * a.sin()) as i32,
            -i * 3,
        ),));
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
        if z.n > 5 {
            buffer.despawn(id);
        }
    }
}
