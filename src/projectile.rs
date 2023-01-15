use crate::draw::ZapSprite;
use crate::enemy::EnemyHittable;
use crate::physics::collide_any;
use crate::physics::IntRect;
use crate::player::Controller;
use crate::resources::SceneResources;
use crate::vfx::ZapFlash;
use hecs::{CommandBuffer, World};

struct DamageEnemies {}
struct DamagePlayer {}

pub struct Projectile {
    prec_x: f32,
    prec_y: f32,
    pub vx: f32,
    pub vy: f32,
}

impl Projectile {
    fn new(rect: &IntRect, vx: f32, vy: f32) -> Self {
        Self {
            prec_x: rect.x as f32,
            prec_y: rect.y as f32,
            vx,
            vy,
        }
    }

    pub fn update(resources: &mut SceneResources, buffer: &mut CommandBuffer) {
        let world = resources.world_ref.lock().unwrap();
        for (e, (proj, rect)) in world.query::<(&mut Projectile, &mut IntRect)>().iter() {
            let ox = rect.x;
            let oy = rect.y;
            proj.prec_x += proj.vx;
            proj.prec_y += proj.vy;
            rect.x = proj.prec_x.round() as i32;
            rect.y = proj.prec_y.round() as i32;
            if collide_any(&world, &resources.body_index, rect) {
                buffer.despawn(e);
                let (x, y) = find_collision_pos(&world, resources, ox, oy, rect);
                let sx = if proj.vx > 0.0 { x + 7 } else { x };
                buffer.spawn((ZapFlash::new_from_centre(sx, y + 2),));
            }
        }
        for (e, (proj, rect, _)) in world
            .query::<(&mut Projectile, &mut IntRect, &DamageEnemies)>()
            .iter()
        {
            let mut live = true;
            for (_, (en, e_rect)) in world.query::<(&mut EnemyHittable, &IntRect)>().iter() {
                if live && en.hp > 0 && rect.intersects(e_rect) {
                    buffer.despawn(e);
                    let sx = if proj.vx > 0.0 { rect.x + 7 } else { rect.x };
                    buffer.spawn((ZapFlash::new_from_centre(sx, rect.y + 2),));
                    en.hurt(1);
                    live = false;
                }
            }
        }

        if let Ok(mut q) = world.query_one::<(&mut Controller, &IntRect)>(resources.player_id) {
            if let Some((c, p_rect)) = q.get() {
                for (id, (proj, rect, _)) in world
                    .query::<(&mut Projectile, &mut IntRect, &DamagePlayer)>()
                    .iter()
                {
                    if rect.intersects(p_rect) {
                        c.hurt();
                        buffer.despawn(id);
                        let sx = if proj.vx > 0.0 { rect.x + 7 } else { rect.x };
                        buffer.spawn((ZapFlash::new_from_centre(sx, rect.y + 2),));
                    }
                }
            };
        };
    }
}

fn find_collision_pos(
    world: &World,
    resources: &SceneResources,
    ox: i32,
    oy: i32,
    rect: &IntRect,
) -> (i32, i32) {
    // this function can be slow as it's only called to generate the vfx when a projectile hits a wall
    // but it should be better than this because there is already code to do this more efficiently elsewhere!
    let mut r = rect.clone();
    let dx = (ox - r.x).signum();
    while r.x != ox {
        r.x += dx;
        if !collide_any(world, &resources.body_index, &r) {
            return (r.x, r.y);
        }
    }
    let dy = (oy - r.y).signum();
    while r.y != oy {
        r.y += dy;
        if !collide_any(world, &resources.body_index, &r) {
            return (r.x, r.y);
        }
    }
    (r.x, r.y)
}

pub fn make_player_projectile(buffer: &mut CommandBuffer, rect: IntRect, vx: f32) {
    let proj = Projectile::new(&rect, vx, 0.0);
    buffer.spawn((rect, ZapSprite::new(), proj, DamageEnemies {}));
}

pub fn make_enemy_projectile(buffer: &mut CommandBuffer, rect: IntRect, vx: f32) {
    let proj = Projectile::new(&rect, vx, 0.0);
    buffer.spawn((rect, ZapSprite::new(), proj, DamagePlayer {}));
}
