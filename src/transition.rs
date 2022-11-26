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
