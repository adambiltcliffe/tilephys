use crate::physics::{Actor, IntRect};
use crate::player::Controller;
use crate::resources::Resources;
use hecs::{CommandBuffer, World};

pub fn add_pickup(world: &mut World, x: i32, y: i32) {
    let rect = IntRect::new(x - 8, y - 16, 16, 16);
    let draw = crate::draw::PickupSprite::new();
    let actor = Actor::new(&rect, 0.4);
    world.spawn((rect, draw, actor, Pickup { touched: false }));
}

pub struct Pickup {
    touched: bool,
}

impl Pickup {
    pub fn update(
        world: &World,
        resources: &mut Resources,
        buffer: &mut CommandBuffer,
    ) -> Option<()> {
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
