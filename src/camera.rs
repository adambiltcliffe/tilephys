use crate::physics::{Actor, IntRect};
use crate::resources::Resources;
use hecs::World;
use macroquad::prelude::*;

pub fn add_camera(world: &mut World, player_pos: Vec2) -> Vec2 {
    let pos = vec2(player_pos.x, player_pos.y - CAMERA_FLOOR_OFFSET);
    world.spawn((PlayerCamera::new(pos.y), pos.clone()));
    pos
}

pub struct PlayerCamera {
    floor: f32,
}

const CAMERA_BUFFER_X: f32 = 16.0;
const CAMERA_BUFFER_ABOVE: f32 = 80.0;
const CAMERA_BUFFER_BELOW: f32 = 48.0;
const CAMERA_FLOOR_OFFSET: f32 = 32.0;
const CAMERA_V_SPEED: f32 = 4.0;

impl PlayerCamera {
    fn new(floor: f32) -> Self {
        Self { floor }
    }

    pub fn update_and_get(world: &World, resources: &Resources) -> Option<Vec2> {
        let mut q = world
            .query_one::<(&Actor, &IntRect)>(resources.player_id)
            .ok()?;
        let (player_pos, player_grounded) = q
            .get()
            .map(|(actor, rect)| (rect.centre(), actor.grounded))?;
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
