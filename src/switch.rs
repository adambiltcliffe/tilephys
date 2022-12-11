use crate::physics::{Actor, IntRect};
use hecs::World;

pub fn add_switch(world: &mut World, name: String, x: i32, y: i32) {
    let rect = IntRect::new(x - 8, y - 16, 16, 16);
    let draw = crate::draw::SwitchSprite::new();
    let actor = Actor::new(&rect, 0.4);
    world.spawn((rect, draw, actor, Switch { name }));
}

pub struct Switch {
    pub name: String,
}
