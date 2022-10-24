use crate::physics::{Actor, IntRect};
use hecs::{Entity, World};

pub(crate) struct Enemy {
    dir: f32,
    pub hp: i32,
}

impl Enemy {
    pub fn new() -> Self {
        Self { dir: 0.0, hp: 3 }
    }

    pub fn update(world: &World, player_id: Entity) {
        let player_x: Option<f32> = world
            .get::<&IntRect>(player_id)
            .map(|rect| rect.centre().x)
            .ok();
        for (_, (actor, enemy, rect)) in world.query::<(&mut Actor, &mut Enemy, &IntRect)>().iter()
        {
            if quad_rand::rand() < (u32::MAX / 10) {
                if player_x.is_some() && quad_rand::rand() < (u32::MAX / 2) {
                    enemy.dir = (player_x.unwrap() - rect.centre().x).signum() * 5.0;
                } else {
                    enemy.dir = quad_rand::gen_range(-6.0, 6.0);
                }
            }
            if actor.grounded && quad_rand::rand() < (u32::MAX / 5) {
                actor.vy = -8.0;
            }
            actor.vx += enemy.dir;
        }
    }
}
