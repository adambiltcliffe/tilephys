use crate::{
    physics::IntRect,
    render::{WALL_VISION_DEPTH, Renderer},
    RENDER_H, RENDER_W,
};
use macroquad::{
    input::{
        is_key_down, is_key_pressed, is_mouse_button_down, is_mouse_button_pressed, mouse_position,
        KeyCode, MouseButton,
    },
    prelude::get_char_pressed,
};
use std::collections::HashSet;

#[derive(PartialEq, Hash, Eq, Clone, Copy)]
pub enum VirtualKey {
    Left,
    Right,
    Jump,
    Fire,
    Interact,
    DebugRestart,
    DebugWin,
    DebugKill,
}

const ALL_KEYS: [(KeyCode, VirtualKey); 8] = [
    (KeyCode::Left, VirtualKey::Left),
    (KeyCode::Right, VirtualKey::Right),
    (KeyCode::Z, VirtualKey::Jump),
    (KeyCode::X, VirtualKey::Fire),
    (KeyCode::C, VirtualKey::Interact),
    (KeyCode::R, VirtualKey::DebugRestart),
    (KeyCode::W, VirtualKey::DebugWin),
    (KeyCode::K, VirtualKey::DebugKill),
];

// I need a to do ceil by myself (works on the principle that 8.7 as i32 == 8) to be able to use it in a constant

const WVDC: i32 = WALL_VISION_DEPTH as i32
    + (((WALL_VISION_DEPTH as i32) as f32 - WALL_VISION_DEPTH)
        / ((WALL_VISION_DEPTH as i32) as f32 - WALL_VISION_DEPTH)) as i32;

const CLICK_AREAS: [(IntRect, VirtualKey); 3] = [
    (
        IntRect {
            x: WVDC + 64,
            y: RENDER_H as i32 - WVDC - 16,
            w: 16,
            h: 16,
        },
        VirtualKey::Left,
    ),
    (
        IntRect {
            x: WVDC + 80,
            y: RENDER_H as i32 - WVDC - 16,
            w: 16,
            h: 16,
        },
        VirtualKey::Right,
    ),
    (
        IntRect {
            x: WVDC + 72,
            y: RENDER_H as i32 - WVDC - 28,
            w: 16,
            h: 16,
        },
        VirtualKey::Jump,
    ),
];

pub struct Input {
    down: HashSet<VirtualKey>,
    pressed: HashSet<VirtualKey>,
    any_pressed: bool,
}

impl Input {
    pub fn new() -> Self {
        Self {
            down: HashSet::new(),
            pressed: HashSet::new(),
            any_pressed: false,
        }
    }

    pub fn update(&mut self, renderer: &Renderer) {
        self.down.clear();
        self.any_pressed = false;
        // creates an IntRect to represent the mouse in order to check collisions with CLICK_AREAS
        let mouse_pos = renderer.format_abs_pos(mouse_position());
        let mouse_rect: IntRect = IntRect {
            x: mouse_pos.0.clamp(0.0, RENDER_W as f32).round() as i32,
            y: mouse_pos.1.clamp(0.0, RENDER_H as f32).round() as i32,
            w: 1,
            h: 1,
        };
        for (cl, vk) in CLICK_AREAS.iter() {
            println!("X:{} Y:{}", cl.x, cl.y);
            println!("X:{} Y:{}", mouse_rect.x, mouse_rect.y);
            if cl.intersects(&mouse_rect) {
                if is_mouse_button_down(MouseButton::Left) {
                    self.down.insert(*vk);
                }
                if is_mouse_button_pressed(MouseButton::Left) {
                    self.pressed.insert(*vk);
                }
                self.any_pressed = true;
            }
        }
        for (kc, vk) in ALL_KEYS.iter() {
            //println!("updating key inputs");
            if is_key_down(*kc) {
                self.down.insert(*vk);
            }
            if is_key_pressed(*kc) {
                self.pressed.insert(*vk);
            }
        }
        self.any_pressed = self.any_pressed || get_char_pressed().is_some();
    }

    pub fn is_down(&self, vk: VirtualKey) -> bool {
        self.down.contains(&vk)
    }

    pub fn is_pressed(&self, vk: VirtualKey) -> bool {
        self.pressed.contains(&vk)
    }

    pub fn is_any_pressed(&self) -> bool {
        self.any_pressed
    }

    pub fn reset(&mut self) {
        self.pressed.clear();
        while get_char_pressed().is_some() {}
    }
}
