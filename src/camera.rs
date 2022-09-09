use crate::physics::{Actor, Controller, IntRect};
use hecs::World;
use macroquad::prelude::*;

pub struct PlayerCamera {
    floor: f32,
}

const CAMERA_BUFFER_X: f32 = 16.0;
const CAMERA_BUFFER_ABOVE: f32 = 80.0;
const CAMERA_BUFFER_BELOW: f32 = 48.0;
const CAMERA_FLOOR_OFFSET: f32 = 32.0;
const CAMERA_V_SPEED: f32 = 4.0;

impl PlayerCamera {
    pub fn new(floor: f32) -> Self {
        Self { floor }
    }

    pub fn update_and_get(world: &World) -> Option<Vec2> {
        let (player_pos, player_grounded) = match world
            .query::<(&Actor, &Controller, &IntRect)>()
            .iter()
            .next()
        {
            None => return None,
            Some((_, (actor, _, rect))) => (rect.centre(), actor.grounded),
        };
        for (_, (cam, v)) in world.query::<(&mut PlayerCamera, &mut Vec2)>().iter() {
            v.x =
                v.x.max(player_pos.x - CAMERA_BUFFER_X)
                    .min(player_pos.x + CAMERA_BUFFER_X);
            v.y =
                v.y.max(player_pos.y - CAMERA_BUFFER_BELOW)
                    .min(player_pos.y + CAMERA_BUFFER_ABOVE);
            if player_grounded {
                cam.floor = player_pos.y - CAMERA_FLOOR_OFFSET;
            }
            v.y = cam
                .floor
                .max(v.y - CAMERA_V_SPEED)
                .min(v.y + CAMERA_V_SPEED);
            return Some(*v);
        }
        return None;
    }
}
