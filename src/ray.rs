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
    blockers
        .iter()
        .filter_map(|id| ray_collision_single(&*world.get::<&TileBody>(*id).unwrap(), orig, dest))
        .min_by(|(t1, _), (t2, _)| f32::total_cmp(t1, t2))
        .map(|(t, ct)| (*orig + (*dest - *orig) * t, ct))
}

// Returns the fraction of the ray (i.e. [0,1]) before the collision
fn ray_collision_single(body: &TileBody, orig: &Vec2, dest: &Vec2) -> Option<(f32, CollisionType)> {
    let bx = body.x as f32;
    let by = body.y as f32;
    let b_height = body.data.len() as i32 / body.width;
    let orig_fine_cell_x = (orig.x - bx) / body.size as f32;
    let orig_fine_cell_y = (orig.y - by) / body.size as f32;
    let dest_fine_cell_x = (dest.x - bx) / body.size as f32;
    let dest_fine_cell_y = (dest.y - by) / body.size as f32;
    let mut cells_entered = Vec::<(f32, CollisionType, usize)>::new();
    let mut entry_t = 0.0f32;
    let mut exit_t = 1.0f32;
    if dest_fine_cell_x > orig_fine_cell_x {
        // generate rightwards cell entries
        let first = (orig_fine_cell_x.ceil() as i32).max(0);
        let last = (dest_fine_cell_x.floor() as i32).min(body.width);
        for ii in first..=last {
            cells_entered.push((
                ((ii as f32) - orig_fine_cell_x) / (dest_fine_cell_x - orig_fine_cell_x),
                CollisionType::Vertical,
                ii as usize,
            ));
        }
        entry_t = entry_t.max((0.0 - orig_fine_cell_x) / (dest_fine_cell_x - orig_fine_cell_x));
        exit_t = exit_t
            .min((body.width as f32 - orig_fine_cell_x) / (dest_fine_cell_x - orig_fine_cell_x));
    } else if dest_fine_cell_x < orig_fine_cell_x {
        // generate leftwards cell entries
        let first = (orig_fine_cell_x.floor() as i32 - 1).min(body.width);
        let last = (dest_fine_cell_x.ceil() as i32 - 1).max(0);
        for ii in last..=first {
            cells_entered.push((
                (((ii + 1) as f32) - orig_fine_cell_x) / (dest_fine_cell_x - orig_fine_cell_x),
                CollisionType::Vertical,
                ii as usize,
            ));
        }
        entry_t = entry_t
            .max((body.width as f32 - orig_fine_cell_x) / (dest_fine_cell_x - orig_fine_cell_x));
        exit_t = exit_t.min((0.0 - orig_fine_cell_x) / (dest_fine_cell_x - orig_fine_cell_x));
    }
    if dest_fine_cell_y > orig_fine_cell_y {
        // generate downwards cell entries
        let first = (orig_fine_cell_y.ceil() as i32).max(0);
        let last = (dest_fine_cell_y.floor() as i32).min(b_height);
        for ii in first..=last {
            cells_entered.push((
                ((ii as f32) - orig_fine_cell_y) / (dest_fine_cell_y - orig_fine_cell_y),
                CollisionType::Horizontal,
                ii as usize,
            ));
        }
        entry_t = entry_t.max((0.0 - orig_fine_cell_y) / (dest_fine_cell_y - orig_fine_cell_y));
        exit_t = exit_t
            .min((b_height as f32 - orig_fine_cell_y) / (dest_fine_cell_y - orig_fine_cell_y));
    } else if dest_fine_cell_y < orig_fine_cell_y {
        // generate upwards cell entries
        let first = (orig_fine_cell_y.floor() as i32 - 1).min(b_height);
        let last = (dest_fine_cell_y.ceil() as i32 - 1).max(0);
        for ii in last..=first {
            cells_entered.push((
                (((ii + 1) as f32) - orig_fine_cell_y) / (dest_fine_cell_y - orig_fine_cell_y),
                CollisionType::Horizontal,
                ii as usize,
            ));
        }
        entry_t = entry_t
            .max((b_height as f32 - orig_fine_cell_y) / (dest_fine_cell_y - orig_fine_cell_y));
        exit_t = exit_t.min((0.0 - orig_fine_cell_y) / (dest_fine_cell_y - orig_fine_cell_y));
    }
    cells_entered.retain(|(t, _, _)| t >= &entry_t && t <= &exit_t);
    cells_entered.sort_by(|(t1, _, _), (t2, _, _)| f32::total_cmp(t1, t2));
    if cells_entered.len() == 0 {
        return None;
    }
    let mut cx =
        (orig_fine_cell_x + entry_t * (dest_fine_cell_x - orig_fine_cell_x)).floor() as usize;
    let mut cy =
        (orig_fine_cell_y + entry_t * (dest_fine_cell_y - orig_fine_cell_y)).floor() as usize;
    for (t, ct, n) in cells_entered {
        match ct {
            CollisionType::Horizontal => {
                cy = n;
            }
            CollisionType::Vertical => {
                cx = n;
            }
        }
        if (cx as i32) < body.width && (cy as i32) <= b_height {
            let idx = cy * body.width as usize + cx;
            if idx < body.data.len() && body.data[idx].is_blocker() {
                return Some((t, ct));
            }
        }
    }
    None
}
