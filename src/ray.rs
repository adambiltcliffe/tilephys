use crate::index::SpatialIndex;
use crate::physics::{IntRect, TileBody};
use hecs::World;
use macroquad::math::Vec2;

pub enum CollisionType {
    Horizontal,
    Vertical,
}

pub fn ray_collision(
    world: &World,
    body_index: &SpatialIndex,
    orig: &Vec2,
    dest: &Vec2,
) -> Option<(Vec2, CollisionType)> {
    let bounds = {
        let min_x = orig.x.min(dest.x).floor() as i32;
        let min_y = orig.y.min(dest.y).floor() as i32;
        let max_x = orig.x.max(dest.x).ceil() as i32;
        let max_y = orig.y.max(dest.y).ceil() as i32;
        IntRect::new(min_x, min_y, max_x - min_x, max_y - min_y)
    };
    let blockers = body_index.entities(&bounds);
    println!("{}", blockers.len());
    blockers
        .iter()
        .filter_map(|id| ray_collision_single(&*world.get::<&TileBody>(*id).unwrap(), orig, dest))
        .min_by(|(t1, _), (t2, _)| f32::total_cmp(t1, t2))
        .map(|(t, ct)| (*orig + (*dest - *orig) * t, ct))
}

// Returns the fraction of the ray (i.e. [0,1]) before the collision
fn ray_collision_single(body: &TileBody, orig: &Vec2, dest: &Vec2) -> Option<(f32, CollisionType)> {
    if body.get_rect().intersects(&IntRect::new(
        orig.x.round() as i32,
        orig.y.round() as i32,
        1,
        1,
    )) {
        println!("within");
        return None;
    }
    println!("not within");
    Some((0.1, CollisionType::Vertical))
}
