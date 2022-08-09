use crate::physics::TileBody;
use hecs::{Entity, World};
use macroquad::prelude::*;

struct Left {
    x: f32,
    y: f32,
    h: f32,
}

struct Right {
    x: f32,
    y: f32,
    h: f32,
}

struct Top {
    x: f32,
    y: f32,
    w: f32,
}

struct Bottom {
    x: f32,
    y: f32,
    w: f32,
}

enum Obscurer {
    L(Left),
    R(Right),
    T(Top),
    B(Bottom),
}

struct Obscurers {
    lefts: Vec<Left>,
    rights: Vec<Right>,
    tops: Vec<Top>,
    bottoms: Vec<Bottom>,
}

impl Obscurers {
    fn new() -> Self {
        Self {
            lefts: Vec::new(),
            rights: Vec::new(),
            tops: Vec::new(),
            bottoms: Vec::new(),
        }
    }
}

pub fn compute_obscurers(world: &mut World) {
    let mut new: Vec<(Entity, Obscurers)> = Vec::new();
    for (id, body) in world.query::<&TileBody>().iter() {
        let mut o = Obscurers::new();
        let cw = body.width;
        let ch = body.data.len() as i32 / body.width;
        for cx in 0..=body.width {
            let mut sly: Option<i32> = None;
            let mut sry: Option<i32> = None;
            for cy in 0..=ch {
                let index = (cy * body.width + cx) as usize;
                let is_left_edge =
                    cy < ch && cx < cw && body.data[index] && (cx == 0 || !body.data[index - 1]);
                let is_right_edge =
                    cy < ch && cx > 0 && body.data[index - 1] && (cx == cw || !body.data[index]);
                if is_left_edge && sly.is_none() {
                    sly = Some(cy);
                } else if !is_left_edge && sly.is_some() {
                    o.lefts.push(Left {
                        x: (cx * body.size) as f32,
                        y: (sly.unwrap() * body.size) as f32,
                        h: ((cy - sly.unwrap()) * body.size) as f32,
                    });
                    sly = None;
                }
                if is_right_edge && sry.is_none() {
                    sry = Some(cy);
                } else if !is_right_edge && sry.is_some() {
                    o.rights.push(Right {
                        x: (cx * body.size) as f32,
                        y: (sry.unwrap() * body.size) as f32,
                        h: ((cy - sry.unwrap()) * body.size) as f32,
                    });
                    sry = None;
                }
            }
        }
        for cy in 0..=ch {
            let mut stx: Option<i32> = None;
            let mut sbx: Option<i32> = None;
            for cx in 0..=cw {
                let index = (cy * body.width + cx) as usize;
                let is_top_edge = cx < cw
                    && cy < ch
                    && body.data[index]
                    && (cy == 0 || !body.data[index - body.width as usize]);
                let is_bottom_edge = cx < cw
                    && cy > 0
                    && body.data[index - body.width as usize]
                    && (cy == ch || !body.data[index]);
                if is_top_edge && stx.is_none() {
                    stx = Some(cx);
                } else if !is_top_edge && stx.is_some() {
                    o.tops.push(Top {
                        x: (stx.unwrap() * body.size) as f32,
                        y: (cy * body.size) as f32,
                        w: ((cx - stx.unwrap()) * body.size) as f32,
                    });
                    stx = None;
                }
                if is_bottom_edge && sbx.is_none() {
                    sbx = Some(cx);
                } else if !is_bottom_edge && sbx.is_some() {
                    o.bottoms.push(Bottom {
                        x: (sbx.unwrap() * body.size) as f32,
                        y: (cy * body.size) as f32,
                        w: ((cx - sbx.unwrap()) * body.size) as f32,
                    });
                    sbx = None;
                }
            }
        }
        new.push((id, o));
    }
    for (id, obs) in new.into_iter() {
        world.insert_one(id, obs).unwrap();
    }
}

pub fn draw_visibility(world: &World) {
    for (_, (body, obs)) in world.query::<(&TileBody, &Obscurers)>().iter() {
        let bx = body.x as f32;
        let by = body.y as f32;
        for l in &obs.lefts {
            draw_line(l.x + bx, l.y + by, l.x + bx, l.y + l.h + by, 2., PINK);
        }
        for r in &obs.rights {
            draw_line(r.x + bx, r.y + by, r.x + bx, r.y + r.h + by, 2., GREEN);
        }
        for t in &obs.tops {
            draw_line(t.x + bx, t.y + by, t.x + t.w + bx, t.y + by, 2., RED);
        }
        for b in &obs.bottoms {
            draw_line(b.x + bx, b.y + by, b.x + b.w + bx, b.y + by, 2., YELLOW);
        }
    }
}
