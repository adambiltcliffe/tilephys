use crate::draw::PlayerSprite;
use crate::input::{Input, KeyState, VirtualKey};
use crate::physics::{Actor, IntRect, Secrecy, TriggerZone};
use crate::pickup::WeaponPickup;
use crate::resources::SceneResources;
use crate::switch::Switch;
use crate::vfx::create_explosion;
use crate::weapon::{new_weapon, weapon_name_indef, WeaponType};
use hecs::{CommandBuffer, Entity};
use std::collections::{HashMap, HashSet};

pub struct Controller {
    jump_frames: u32,
    zones: HashSet<String>,
    pub touched_weapons: HashMap<WeaponType, Entity>,
    facing: i8,
    fire_timer: u32,
    hurt_timer: u8,
    pub hp: u8,
    god_mode: bool,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            jump_frames: 0,
            zones: HashSet::new(),
            touched_weapons: HashMap::new(),
            facing: 1,
            fire_timer: 100000,
            hurt_timer: 0,
            hp: 3,
            god_mode: false,
        }
    }

    pub fn update(resources: &mut SceneResources, buffer: &mut CommandBuffer, input: &Input) {
        let world = resources.world_ref.lock().unwrap();
        let mut q = world.query::<(&mut Actor, &IntRect, &mut PlayerSprite, &mut Controller)>();
        for (id, (player, p_rect, sprite, controller)) in q.iter() {
            let mut new_zones: HashSet<String> = HashSet::new();
            for (_, (trigger, t_rect)) in world.query::<(&mut TriggerZone, &IntRect)>().iter() {
                if p_rect.intersects(t_rect) {
                    if !controller.zones.contains(&trigger.name) {
                        resources
                            .triggers
                            .insert(format!("{}_enter", trigger.name).to_owned());
                        if trigger.secrecy == Secrecy::Hidden {
                            trigger.secrecy = Secrecy::Found;
                            resources.stats.secrets += 1;
                            resources.messages.add("Found a secret area!".to_owned());
                        }
                    }
                    new_zones.insert(trigger.name.clone());
                }
            }
            for z in &controller.zones {
                if !new_zones.contains(z) {
                    resources.triggers.insert(format!("{}_exit", z).to_owned());
                }
            }
            controller.zones = new_zones;
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
            if input.is_pressed(VirtualKey::Interact) {
                let mut interacted = false;
                let mut q = world.query::<(&Actor, &IntRect, &mut Switch)>();
                for (_, (_, s_rect, s)) in q.iter() {
                    if p_rect.intersects(s_rect) && s.enabled {
                        resources
                            .triggers
                            .insert(format!("{}_interact", s.name).to_owned());
                        s.enabled = false;
                        interacted = true;
                    }
                }
                if !interacted {
                    let tw = controller
                        .touched_weapons
                        .iter()
                        .next()
                        // copy the references now so that the borrow can be dropped
                        .map(|(&typ, &id)| (typ, id));
                    match tw {
                        Some((typ, id))
                            if !resources.weapons.iter().any(|w| w.get_type() == typ) =>
                        {
                            // if the backup laser is in inventory anywhere, remove it
                            // player can always get it back again if they still have no ammo
                            if let Some(n) = resources
                                .weapons
                                .iter()
                                .position(|w| w.get_type() == WeaponType::BackupLaser)
                            {
                                resources.weapons.remove(n);
                            }
                            // now we can't have the backup laser so we can just use the len()
                            // to work out if the inventory is full
                            if resources.weapons.len() < 3 {
                                buffer.despawn(id);
                                resources.weapons.push_front(new_weapon(typ));
                            } else {
                                let mut w = world.get::<&mut WeaponPickup>(id).unwrap();
                                w.typ = resources.weapons[0].get_type();
                                resources.weapons.pop_front();
                                resources.weapons.push_front(new_weapon(typ));
                                // mark it as touched to suppress the message next frame
                                controller.touched_weapons.insert(typ, id);
                            }
                            resources
                                .messages
                                .add(format!("Picked up {}.", weapon_name_indef(typ)));
                            resources.selector.change(0.0);
                        }
                        // either not touching a weapon pickup or it's one we already have
                        _ => (),
                    }
                }
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
            if input.is_pressed(VirtualKey::PrevWeapon) {
                if resources.weapons.len() > 1 {
                    resources.weapons.rotate_left(1);
                    resources.selector.change(-1.0);
                } else {
                    resources.selector.change(0.1);
                }
            }
            if input.is_pressed(VirtualKey::NextWeapon) {
                if resources.weapons.len() > 1 {
                    resources.weapons.rotate_right(1);
                    resources.selector.change(1.0);
                } else {
                    resources.selector.change(-0.1);
                }
            }
            let fks = input.state(VirtualKey::Fire);
            let w = &mut resources.weapons[0];
            let t = w.get_ammo_type();
            let n = w.get_ammo_use();
            if resources.ammo[t] >= n {
                // can fire current weapon, up to the weapon to say if we should
                if w.update(buffer, player, p_rect, controller.facing, fks) {
                    controller.fire_timer = 0;
                    sprite.firing = true;
                    resources.ammo[t] -= n;
                }
            } else {
                // can't fire current weapon
                if fks == KeyState::Pressed {
                    'changed: {
                        // so change to one that can if possible
                        for idx in 1..resources.weapons.len() {
                            let (t, u) = {
                                let w = &resources.weapons[idx];
                                (w.get_ammo_type(), w.get_ammo_use())
                            };
                            if resources.ammo[t] >= u {
                                resources.weapons.rotate_left(idx);
                                resources.selector.change(-(idx as f32));
                                break 'changed;
                            }
                        }
                        // if we couldn't, add a backup laser to inventory
                        resources
                            .weapons
                            .push_front(new_weapon(WeaponType::BackupLaser));
                        resources.selector.change(-1.0);
                    }
                }
            }
            controller.fire_timer += 1;
            sprite.muzzle_flash = controller.fire_timer.min(100) as u8;
            if controller.fire_timer > 5 {
                sprite.firing = false;
            }
            if controller.hurt_timer > 0 {
                controller.hurt_timer -= 1;
                sprite.blink = (controller.hurt_timer / 3) % 2 == 0;
            } else {
                sprite.blink = false;
            }
            if controller.hp == 0 || (player.crushed && !controller.god_mode) {
                buffer.remove_one::<PlayerSprite>(id);
                buffer.remove_one::<Controller>(id);
                let (px, py) = p_rect.centre_int();
                create_explosion(buffer, px, py);
                resources.messages.add("You have died.".to_owned());
            }
            #[cfg(debug_assertions)]
            {
                use macroquad::prelude::{is_key_down, KeyCode};
                if is_key_down(KeyCode::Q) && is_key_down(KeyCode::D) && !controller.god_mode {
                    controller.god_mode = true;
                    resources.messages.add("God mode enabled!".to_owned());
                }
            }
        }
    }

    pub fn can_hurt(&self) -> bool {
        self.hurt_timer == 0 && self.hp > 0
    }

    pub fn hurt(&mut self) {
        if self.hurt_timer == 0 && self.hp > 0 && !self.god_mode {
            self.hp -= 1;
            self.hurt_timer = 24;
        }
    }

    pub fn was_hurt(&self) -> bool {
        self.hurt_timer >= 23
    }

    pub fn can_heal(&self) -> bool {
        self.hp < 3
    }

    pub fn heal(&mut self) {
        self.hp += 1;
    }
}
