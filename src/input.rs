use macroquad::{
    input::{is_key_down, is_key_pressed, KeyCode},
    prelude::get_char_pressed,
};
use std::collections::HashSet;

#[derive(PartialEq, Hash, Eq, Clone, Copy)]
pub enum VirtualKey {
    Left,
    Right,
    Jump,
    Fire,
    DebugRestart,
    DebugWin,
}

const ALL_KEYS: [(KeyCode, VirtualKey); 6] = [
    (KeyCode::Left, VirtualKey::Left),
    (KeyCode::Right, VirtualKey::Right),
    (KeyCode::Z, VirtualKey::Jump),
    (KeyCode::X, VirtualKey::Fire),
    (KeyCode::R, VirtualKey::DebugRestart),
    (KeyCode::W, VirtualKey::DebugWin),
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
        for (kc, vk) in ALL_KEYS.iter() {
            if is_key_down(*kc) {
                self.down.insert(*vk);
            }
            if is_key_pressed(*kc) {
                self.pressed.insert(*vk);
            }
        }
        self.any_pressed = get_char_pressed().is_some();
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
