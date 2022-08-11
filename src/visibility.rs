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

fn extend(v: Vec2, radius: f32) -> Vec2 {
    if v.x.abs() < v.y.abs() && v.x.abs() != 0. || v.y.abs() == 0. {
        v * (radius / v.x.abs())
    } else {
        v * (radius / v.y.abs())
    }
}

fn draw_obscurer(x1: f32, y1: f32, x2: f32, y2: f32, eye: Vec2, radius: f32) {
    let p1 = vec2(x1, y1);
    let p2 = vec2(x2, y2);
    let p1s = p1 + extend(p1 - eye, radius);
    let p2s = p2 + extend(p2 - eye, radius);
    draw_triangle(p1, p2, p1s, BLACK);
    draw_triangle(p2, p1s, p2s, BLACK);
}

pub fn draw_visibility(world: &World, eye: Vec2, radius: f32) {
    clear_background(WHITE);
    for (_, (body, obs)) in world.query::<(&TileBody, &Obscurers)>().iter() {
        let bx = body.x as f32;
        let bw = (body.width * body.size) as f32;
        if bx > eye.x + radius || bx + bw < eye.x - radius {
            continue;
        }
        let by = body.y as f32;
        let bh = (body.data.len() as f32 / body.width as f32) * body.size as f32;
        if by > eye.y + radius || by + bh < eye.y - radius {
            continue;
        }
        for l in &obs.lefts {
            if bx + l.x >= eye.x {
                draw_obscurer(l.x + bx, l.y + by, l.x + bx, l.y + l.h + by, eye, radius);
            }
        }
        for r in &obs.rights {
            if bx + r.x <= eye.x {
                draw_obscurer(r.x + bx, r.y + by, r.x + bx, r.y + r.h + by, eye, radius);
            }
        }
        for t in &obs.tops {
            if by + t.y >= eye.y {
                draw_obscurer(t.x + bx, t.y + by, t.x + t.w + bx, t.y + by, eye, radius);
            }
        }
        for b in &obs.bottoms {
            if by + b.y <= eye.y {
                draw_obscurer(b.x + bx, b.y + by, b.x + b.w + bx, b.y + by, eye, radius);
            }
        }
    }
    draw_circle(eye.x, eye.y, 3.0, PINK);
    draw_rectangle_lines(
        eye.x - radius,
        eye.y - radius,
        radius * 2.,
        radius * 2.,
        2.,
        PINK,
    );
}
