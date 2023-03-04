use crate::index::SpatialIndex;
use crate::physics::{IntRect, TileBody};
use hecs::World;
use macroquad::math::Vec2;

pub enum CollisionType {
    Horizontal,
    Vertical,
}

// this function may be ultimately redundant
fn body_contains_vec2(body: &TileBody, v: Vec2) -> bool {
    let r = body.get_rect();
    v.x as i32 >= r.x
        && v.y as i32 >= r.y
        && (v.x.ceil() as i32) < r.x + r.w
        && (v.y.ceil() as i32) < r.y + r.h
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
    if body_contains_vec2(body, *orig) {
        return None;
    }
    let rect = body.get_rect();
    let l = rect.x as f32;
    if orig.x < l && dest.x > l {
        return Some(((l - orig.x) / (dest.x - orig.x), CollisionType::Vertical));
    }
    let r = (rect.x + rect.w) as f32;
    if orig.x > r && dest.x < r {
        return Some(((orig.x - r) / (orig.x - dest.x), CollisionType::Vertical));
    }
    let t = rect.y as f32;
    if orig.y < t && dest.y > t {
        return Some(((t - orig.y) / (dest.y - orig.y), CollisionType::Horizontal));
    }
    let b = (rect.y + rect.h) as f32;
    if orig.y > b && dest.y < b {
        return Some(((orig.y - b) / (orig.y - dest.y), CollisionType::Horizontal));
    }
    return None;
}
