use crate::physics::{Actor, IntRect};
use crate::player::Controller;
use crate::resources::SceneResources;
use crate::weapon::{ammo_max, ammo_name, weapon_name, AmmoQuantity, AmmoType, WeaponType};
use hecs::{CommandBuffer, World};
use std::collections::HashMap;

pub enum PickupType {
    Heart,
    Ammo(AmmoType, AmmoQuantity),
}

pub struct Pickup {
    pub typ: PickupType,
    touched: bool,
}

pub fn add_heart(world: &mut World, x: i32, y: i32) {
    let rect = IntRect::new(x - 8, y - 16, 16, 16);
    let draw = crate::draw::PickupSprite::new();
    let actor = Actor::new(&rect, 0.4);
    world.spawn((
        rect,
        draw,
        actor,
        Pickup {
            typ: PickupType::Heart,
            touched: false,
        },
    ));
}

pub fn add_ammo(world: &mut World, x: i32, y: i32, typ: AmmoType, amt: AmmoQuantity) {
    let rect = IntRect::new(x - 8, y - 16, 16, 16);
    let draw = crate::draw::PickupSprite::new();
    let actor = Actor::new(&rect, 0.4);
    world.spawn((
        rect,
        draw,
        actor,
        Pickup {
            typ: PickupType::Ammo(typ, amt),
            touched: false,
        },
    ));
}

impl Pickup {
    pub fn update(resources: &mut SceneResources, buffer: &mut CommandBuffer) -> Option<()> {
        let world = resources.world_ref.lock().unwrap();
        let mut q = world
            .query_one::<(&IntRect, &mut Controller)>(resources.player_id)
            .ok()?;
        let (p_rect, c) = q.get()?;
        for (id, (rect, p)) in world.query::<(&IntRect, &mut Pickup)>().iter() {
            if rect.intersects(p_rect) {
                if !p.touched {
                    p.touched = true;
                    resources.stats.items += 1
                }
                match p.typ {
                    PickupType::Heart => {
                        if c.can_heal() {
                            buffer.despawn(id);
                            c.heal();
                            resources.messages.add("Picked up a heart.".to_owned());
                        }
                    }
                    PickupType::Ammo(typ, amt) => {
                        if resources.ammo[typ] < ammo_max(typ) {
                            buffer.despawn(id);
                            crate::weapon::add_ammo(
                                &mut resources.weapons,
                                &mut resources.ammo,
                                &mut resources.selector,
                                typ,
                                amt,
                            );
                            resources
                                .messages
                                .add(format!("Picked up {}.", ammo_name(typ, amt)));
                        }
                    }
                }
            }
        }
        Some(())
    }
}

pub struct WeaponPickup {
    touched: bool,
    pub typ: WeaponType,
}

pub fn add_weapon(world: &mut World, x: i32, y: i32, typ: WeaponType) {
    let rect = IntRect::new(x - 12, y - 16, 24, 16);
    let actor = Actor::new(&rect, 0.4);
    world.spawn((
        rect,
        actor,
        WeaponPickup {
            touched: false,
            typ,
        },
    ));
}

impl WeaponPickup {
    pub fn update(resources: &mut SceneResources) -> Option<()> {
        let mut new_touched = HashMap::new();
        let world = resources.world_ref.lock().unwrap();
        let mut q = world
            .query_one::<(&IntRect, &mut Controller)>(resources.player_id)
            .ok()?;
        let (p_rect, c) = q.get()?;
        for (id, (rect, p)) in world.query::<(&IntRect, &mut WeaponPickup)>().iter() {
            if rect.intersects(p_rect) {
                if !p.touched {
                    p.touched = true;
                    resources.stats.items += 1
                }
                new_touched.insert(p.typ, id);
            }
        }
        for typ in new_touched.keys() {
            if !c.touched_weapons.contains_key(typ) {
                if resources.weapons.iter().any(|w| w.get_type() == *typ) {
                    resources
                        .messages
                        .add(format!("Already carrying {}.", weapon_name(*typ)));
                } else if resources.weapons.len() < 3 {
                    resources
                        .messages
                        .add(format!("Press C to pick up {}.", weapon_name(*typ)));
                } else {
                    resources
                        .messages
                        .add(format!("Press C to swap for {}.", weapon_name(*typ)));
                }
            }
        }
        c.touched_weapons = new_touched;
        Some(())
    }
}
