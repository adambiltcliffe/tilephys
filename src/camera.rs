use crate::physics::{teleport_actor_unchecked, Actor, IntRect, PhysicsCoeffs};
use crate::resources::SceneResources;
use hecs::World;
use macroquad::prelude::*;

pub enum EyeballState {
    Tracking(Vec2),
    Free(Vec2),
    Crushed,
}

pub fn add_camera(world: &mut World, player_pos: Vec2) -> Vec2 {
    let pos = vec2(player_pos.x, player_pos.y - CAMERA_FLOOR_OFFSET);
    let eyeball_rect = IntRect::new(player_pos.x as i32, player_pos.y as i32, 2, 2);
    let act = Actor::new(&eyeball_rect, PhysicsCoeffs::Static);
    world.spawn((PlayerCamera::new(pos.y), pos, act, eyeball_rect));
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

    pub fn update(resources: &mut SceneResources) {
        let mut crushed = false;
        let world = resources.world_ref.lock().unwrap();
        match &mut resources.eye {
            EyeballState::Tracking(ref mut eye_pos) => {
                let q = world.query_one::<(&Actor, &IntRect)>(resources.player_id);
                if q.is_err() {
                    return;
                }
                let mut q = q.unwrap();
                if let Some((player_pos, player_grounded)) =
                    q.get().map(|(actor, rect)| (rect.centre(), actor.grounded))
                {
                    *eye_pos = player_pos;
                    for (_, (cam, v, eyeball_actor, eyeball_rect)) in world
                        .query::<(&mut PlayerCamera, &mut Vec2, &mut Actor, &mut IntRect)>()
                        .iter()
                    {
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
                        resources.camera_pos = *v;
                        teleport_actor_unchecked(eyeball_actor, &player_pos);
                        eyeball_rect.x = player_pos.x as i32;
                        eyeball_rect.y = player_pos.y as i32;
                    }
                }
            }
            EyeballState::Free(ref mut eye_pos) => {
                // the player has died; the eyeball is now a free-floating actor
                for (_, (_, eyeball_rect, eyeball_actor)) in world
                    .query::<(&mut PlayerCamera, &mut IntRect, &Actor)>()
                    .iter()
                {
                    *eye_pos = eyeball_rect.centre();
                    if eyeball_actor.crushed {
                        crushed = true;
                    }
                }
            }
            EyeballState::Crushed => (),
        }
        if crushed {
            resources.eye = EyeballState::Crushed;
        }
    }
}
