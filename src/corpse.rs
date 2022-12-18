use crate::draw::CorpseSprite;
use hecs::World;

pub struct Corpse {
    n: i32,
}

impl Corpse {
    pub fn new() -> Self {
        Self { n: 0 }
    }

    pub fn update(world: &World) {
        for (_, (corpse, spr)) in world.query::<(&mut Corpse, &mut CorpseSprite)>().iter() {
            corpse.n += 1;
            spr.frame = (corpse.n / 2).min(4) as u8;
        }
    }
}
