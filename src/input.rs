use crate::physics::IntRect;
use crate::render::WALL_VISION_DEPTH;
use macroquad::{
    input::{is_key_down, is_key_pressed, mouse_position, is_mouse_button_down, is_mouse_button_pressed, KeyCode, MouseButton},
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

const CLICK_AREAS: [(IntRect, VirtualKey); 3] = [
    (IntRect { x: 0, y: 0, w: 16, h: 16}, VirtualKey::Left),
    (IntRect { x: 16, y: 0, w: 16, h: 16}, VirtualKey::Right),
    (IntRect { x: 32, y: 0, w: 16, h: 16}, VirtualKey::Jump)
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

    pub fn update(&mut self) {
        self.down.clear();
        // creates an IntRect to represent the mouse in order to check collisions with CLICK_AREAS
        let mouse_rect: IntRect = IntRect {x: mouse_position().0.round() as i32, y: mouse_position().1.round() as i32, w: 0, h: 0};
        self.any_pressed = false;
        for (cl, vk) in CLICK_AREAS.iter() {
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
