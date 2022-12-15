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

pub fn update_vfx(world: &World, buffer: &mut CommandBuffer) {
    for (id, z) in world.query::<&mut ZapFlash>().iter() {
        z.n += 1;
        if z.n > 5 {
            buffer.despawn(id);
        }
    }
}
