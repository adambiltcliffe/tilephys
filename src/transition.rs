use crate::render::WALL_VISION_DEPTH;
use macroquad::prelude::*;
use quad_rand::gen_range;

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum TransitionEffectType {
    Open,
    Shatter,
}

pub fn new_transition(typ: TransitionEffectType) -> Box<dyn TransitionEffect> {
    match typ {
        TransitionEffectType::Open => Box::new(Open::new()),
        TransitionEffectType::Shatter => Box::new(Shatter::new()),
    }
}

pub trait TransitionEffect {
    fn tick(&mut self);
    fn draw(&self, freeze_frame: &Texture2D);
    fn finished(&self) -> bool;
}

pub struct Open {
    n: i32,
}

impl Open {
    pub fn new() -> Self {
        Self { n: 0 }
    }
}

impl TransitionEffect for Open {
    fn tick(&mut self) {
        self.n += 1;
    }
    fn draw(&self, freeze_frame: &Texture2D) {
        let d = (self.n.pow(2)).min(self.n * 10) as f32;
        draw_texture_ex(
            *freeze_frame,
            WALL_VISION_DEPTH.ceil() - d,
            WALL_VISION_DEPTH.ceil(),
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(0.0, 0.0, 160.0, 240.0)),
                ..Default::default()
            },
        );
        draw_texture_ex(
            *freeze_frame,
            WALL_VISION_DEPTH.ceil() + d + 160.0,
            WALL_VISION_DEPTH.ceil(),
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(160.0, 0.0, 160.0, 240.0)),
                ..Default::default()
            },
        );
    }
    fn finished(&self) -> bool {
        self.n > 16
    }
}

pub struct Shatter {
    data: Vec<Vec<(f32, f32, f32, f32)>>,
    finished: bool,
}

impl Shatter {
    pub fn new() -> Self {
        let mut data = Vec::new();
        for y in 0..15 {
            let mut v = Vec::new();
            for x in 0..20 {
                let a = gen_range(0.0, std::f32::consts::PI * 2.0);
                v.push((
                    x as f32 * 16.0,
                    y as f32 * 16.0,
                    a.cos() * 5.0,
                    a.sin() * 5.0,
                ));
            }
            data.push(v);
        }
        Self {
            data,
            finished: false,
        }
    }
}

impl TransitionEffect for Shatter {
    fn tick(&mut self) {
        let mut finished = true;
        for y in 0..15 {
            for x in 0..20 {
                let (px, py, vx, vy) = self.data[y][x];
                self.data[y][x] = (px + vx, py + vy, vx, vy + 1.0);
                if py < 240.0 {
                    finished = false;
                }
            }
        }
        self.finished = finished;
    }
    fn draw(&self, freeze_frame: &Texture2D) {
        for y in 0..15 {
            for x in 0..20 {
                let (px, py, _, _) = self.data[y][x];
                let sx = x as f32 * 16.0;
                let sy = y as f32 * 16.0;
                draw_texture_ex(
                    *freeze_frame,
                    WALL_VISION_DEPTH.ceil() + px,
                    WALL_VISION_DEPTH.ceil() + py,
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect::new(sx, sy, 16.0, 16.0)),
                        ..Default::default()
                    },
                );
            }
        }
    }
    fn finished(&self) -> bool {
        self.finished
    }
}
