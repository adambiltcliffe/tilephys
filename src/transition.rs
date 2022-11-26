use crate::render::WALL_VISION_DEPTH;
use macroquad::prelude::*;

pub trait TransitionEffect {
    fn tick(&mut self);
    fn draw(&self, freeze_frame: &Texture2D);
    fn finished(&self) -> bool;
}

pub struct Fade {
    alpha: f32,
}

impl Fade {
    pub fn new() -> Self {
        Self { alpha: 1.0 }
    }
}

impl TransitionEffect for Fade {
    fn tick(&mut self) {
        self.alpha -= 0.03;
    }
    fn draw(&self, freeze_frame: &Texture2D) {
        let c = Color::new(1.0, 1.0, 1.0, self.alpha);
        draw_texture(
            *freeze_frame,
            WALL_VISION_DEPTH.ceil(),
            WALL_VISION_DEPTH.ceil(),
            c,
        );
    }
    fn finished(&self) -> bool {
        self.alpha <= 0.0
    }
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
        self.n > 160
    }
}
