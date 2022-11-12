use crate::draw::PlayerSprite;
use crate::input::{Input, VirtualKey};
use crate::physics::{Actor, IntRect, Projectile, Secrecy, TriggerZone};
use hecs::{CommandBuffer, World};
use macroquad::prelude::Color;
use std::collections::HashSet;

pub struct Controller {
    jump_frames: u32,
    triggers: HashSet<String>,
    facing: i8,
    fire_timer: u32,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            jump_frames: 0,
            triggers: HashSet::new(),
            facing: 1,
            fire_timer: 100000,
        }
    }

    pub fn update(
        world: &World,
        buffer: &mut CommandBuffer,
        input: &Input,
    ) -> (HashSet<String>, u32) {
        let mut result: HashSet<String> = HashSet::new();
        let mut secret_count = 0;
        let mut q = world.query::<(&mut Actor, &IntRect, &mut PlayerSprite, &mut Controller)>();
        for (_, (player, p_rect, sprite, controller)) in q.iter() {
            let mut new_triggers: HashSet<String> = HashSet::new();
            for (_, (trigger, t_rect)) in world.query::<(&mut TriggerZone, &IntRect)>().iter() {
                if p_rect.intersects(&t_rect) {
                    let name = trigger.name.clone();
                    if !controller.triggers.contains(&name) {
                        result.insert(name.clone());
                        if trigger.secrecy == Secrecy::HiddenSecret {
                            trigger.secrecy = Secrecy::FoundSecret;
                            secret_count += 1;
                        }
                    }
                    new_triggers.insert(name);
                }
            }
            controller.triggers = new_triggers;
            if input.is_down(VirtualKey::Left) {
                player.vx -= 3.0;
                controller.facing = -1;
                sprite.flipped = false;
            }
            if input.is_down(VirtualKey::Right) {
                player.vx += 3.0;
                controller.facing = 1;
                sprite.flipped = true;
            }
            if input.is_pressed(VirtualKey::Fire) {
                let color = crate::draw::ColorRect::new(Color::new(0.58, 1.0, 0.25, 1.0));
                let rect = IntRect::new(
                    p_rect.x + 3 + controller.facing as i32 * 9,
                    p_rect.y + 11,
                    8,
                    5,
                );
                let proj = Projectile::new(&rect, controller.facing as f32 * 10.0, 0.0);
                buffer.spawn((rect, color, proj));
                player.vx -= controller.facing as f32 * 10.0;
                controller.fire_timer = 0;
                sprite.firing = true;
            }
            if player.grounded && input.is_pressed(VirtualKey::Jump) {
                player.vy = -6.0;
                controller.jump_frames = 5;
            } else if controller.jump_frames > 0
                && input.is_down(VirtualKey::Jump)
                && player.vy < 0.0
            {
                player.vy = -10.0;
                controller.jump_frames -= 1;
            } else {
                controller.jump_frames = 0;
            }
            if player.grounded {
                sprite.n += player.vx.abs() as i32;
            }
            controller.fire_timer += 1;
            if controller.fire_timer > 5 {
                sprite.firing = false;
            }
        }
        (result, secret_count)
    }
}
