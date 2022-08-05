use hecs::{Entity, World};
use macroquad::math::{vec2, Vec2};

pub struct IntRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl IntRect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }
}

fn offset_rect(rect: &IntRect, dx: i32, dy: i32) -> IntRect {
    IntRect::new(rect.x + dx, rect.y + dy, rect.w, rect.h)
}

fn pushing_rect(rect: &IntRect) -> IntRect {
    IntRect::new(rect.x, rect.y, rect.w, rect.h + 1)
}

fn feet_rect(rect: &IntRect) -> IntRect {
    IntRect::new(rect.x, rect.y + rect.h, rect.w, 1)
}

pub struct TileBody {
    pub width: i32,
    pub size: i32,
    pub data: Vec<bool>,
    pub x: i32,
    pub y: i32,
}

impl TileBody {
    pub fn new(x: i32, y: i32, size: i32, width: i32, data: Vec<bool>) -> Self {
        Self {
            x,
            y,
            size,
            width,
            data,
        }
    }

    // this is only pub at the moment because of checking if the mouse intersects it for debugging
    pub fn collide(&self, rect: &IntRect) -> bool {
        let min_kx = (rect.x - self.x).div_euclid(self.size);
        let max_kx = (rect.x + rect.w - 1 - self.x).div_euclid(self.size);
        let min_ky = (rect.y - self.y).div_euclid(self.size);
        let max_ky = (rect.y + rect.h - 1 - self.y).div_euclid(self.size);
        for ky in min_ky..=max_ky {
            if ky >= 0 {
                for kx in min_kx..=max_kx {
                    if kx >= 0 && kx < self.width {
                        let index = ky * self.width + kx;
                        if index >= 0 && index < self.data.len() as i32 && self.data[index as usize]
                        {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

pub struct Actor {
    prec_x: f32,
    prec_y: f32,
}

impl Actor {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            prec_x: x as f32,
            prec_y: y as f32,
        }
    }
}

pub struct ConstantMotion {
    vx: i32,
    vy: i32,
}

impl ConstantMotion {
    pub fn new(vx: i32, vy: i32) -> Self {
        Self { vx, vy }
    }
    pub fn apply(player: &mut Actor, player_rect: &mut IntRect, world: &mut World) {
        for (e, cm) in world.query::<&ConstantMotion>().iter() {
            move_body(player, player_rect, &world, e, cm.vx, cm.vy);
        }
    }
}

pub struct PathMotion {
    prec_x: f32,
    prec_y: f32,
    next_node: usize,
    base_pos: Vec2,
    offsets: Vec<Vec2>,
    speed: f32,
}

impl PathMotion {
    pub fn new(x: f32, y: f32, point_list: Vec<(f32, f32)>, speed: f32) -> Self {
        Self {
            prec_x: x,
            prec_y: y,
            next_node: 0,
            base_pos: vec2(x, y),
            offsets: point_list.iter().map(|(px, py)| vec2(*px, *py)).collect(),
            speed,
        }
    }

    pub fn apply(player: &mut Actor, player_rect: &mut IntRect, world: &mut World) {
        for (e, pm) in world.query::<&mut PathMotion>().iter() {
            let dest = pm.offsets[pm.next_node] + pm.base_pos;
            let curr = vec2(pm.prec_x, pm.prec_y);
            let v = dest - curr;
            let tmp = if v.length() < pm.speed {
                pm.next_node = (pm.next_node + 1) % pm.offsets.len();
                dest
            } else {
                let new = curr + v.normalize() * pm.speed;
                new
            };
            pm.prec_x = tmp.x;
            pm.prec_y = tmp.y;

            let (dx, dy) = {
                let body = world.get::<&TileBody>(e).unwrap();
                (
                    pm.prec_x.round() as i32 - body.x,
                    pm.prec_y.round() as i32 - body.y,
                )
            };
            move_body(player, player_rect, &world, e, dx, dy);
        }
    }
}

// TODO: try to make this not pub
pub fn move_actor(
    actor: &mut Actor,
    rect: &mut IntRect,
    vx: f32,
    vy: f32,
    world: &World,
) -> (bool, bool) {
    actor.prec_x += vx;
    let targ_x = actor.prec_x.round() as i32;
    let mut collided_x = false;
    while rect.x != targ_x {
        let dx = (targ_x - rect.x).signum();
        if world
            .query::<&TileBody>()
            .iter()
            .any(|(_, c)| c.collide(&offset_rect(rect, dx, 0)))
        {
            actor.prec_x = rect.x as f32;
            collided_x = true;
            break;
        } else {
            rect.x += dx;
        }
    }
    actor.prec_y += vy;
    let targ_y = actor.prec_y.round() as i32;
    let mut collided_y = false;
    while rect.y != targ_y {
        let dy = (targ_y - rect.y).signum();
        if world
            .query::<&TileBody>()
            .iter()
            .any(|(_, c)| c.collide(&offset_rect(rect, 0, dy)))
        {
            actor.prec_y = rect.y as f32;
            collided_y = true;
            break;
        } else {
            rect.y += dy
        }
    }
    (collided_x, collided_y)
}

fn move_body(
    actor: &mut Actor,
    actor_rect: &mut IntRect,
    world: &World,
    index: Entity,
    vx: i32,
    vy: i32,
) {
    // this is a fiddly mess of borrows and drops but we should be able to skip
    // this in many cases if there are no actors in position to be pushed
    for _ii in 0..(vx.abs()) {
        let mut body = world.get::<&mut TileBody>(index).unwrap();
        let mut should_move = body.collide(&pushing_rect(&actor_rect));
        body.x += vx.signum();
        should_move |= body.collide(&pushing_rect(&actor_rect));
        drop(body);
        if should_move {
            move_actor(actor, actor_rect, vx.signum() as f32, 0.0, &world);
        }
    }
    for _ii in 0..(vy.abs()) {
        let mut body = world.get::<&mut TileBody>(index).unwrap();
        let mut should_move = body.collide(&pushing_rect(&actor_rect));
        body.y += vy.signum();
        should_move |= body.collide(&pushing_rect(&actor_rect));
        drop(body);
        if should_move {
            move_actor(actor, actor_rect, 0.0, vy.signum() as f32, &world);
        }
    }
}

// TODO this probably shouldn't even be public eventually
pub fn check_player_grounded(player_rect: &IntRect, world: &World) -> bool {
    world
        .query::<&TileBody>()
        .iter()
        .any(|(_, c)| c.collide(&feet_rect(&player_rect)))
}
