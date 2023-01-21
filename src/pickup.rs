use crate::physics::{Actor, IntRect};
use crate::player::Controller;
use crate::resources::SceneResources;
use crate::weapon::{weapon_name, WeaponType};
use hecs::{CommandBuffer, World};
use std::collections::HashMap;

pub struct Pickup {
    touched: bool,
}

pub fn add_pickup(world: &mut World, x: i32, y: i32) {
    let rect = IntRect::new(x - 8, y - 16, 16, 16);
    let draw = crate::draw::PickupSprite::new();
    let actor = Actor::new(&rect, 0.4);
    world.spawn((rect, draw, actor, Pickup { touched: false }));
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
                if c.can_heal() {
                    buffer.despawn(id);
                    c.heal();
                    resources.messages.add("Picked up a heart.".to_owned());
                }
            }
        }
        Some(())
    }
}

pub struct WeaponPickup {
    touched: bool,
    typ: WeaponType,
}

pub fn add_weapon(world: &mut World, x: i32, y: i32, typ: WeaponType) {
    let rect = IntRect::new(x - 8, y - 16, 16, 16);
    let draw = crate::draw::ColorRect::new(macroquad::color::ORANGE);
    let actor = Actor::new(&rect, 0.4);
    world.spawn((
        rect,
        draw,
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
            if !c.touched_weapons.contains_key(&typ) {
                resources
                    .messages
                    .add(format!("Press C to pick up {}.", weapon_name(*typ)));
            }
        }
        c.touched_weapons = new_touched;
        Some(())
    }
}
