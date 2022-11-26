use macroquad::input::{is_key_down, is_key_pressed, KeyCode};
use std::collections::HashSet;

#[derive(PartialEq, Hash, Eq, Clone, Copy)]
pub enum VirtualKey {
    Left,
    Right,
    Jump,
    Fire,
    DebugRestart,
}

const ALL_KEYS: [(KeyCode, VirtualKey); 5] = [
    (KeyCode::Left, VirtualKey::Left),
    (KeyCode::Right, VirtualKey::Right),
    (KeyCode::Z, VirtualKey::Jump),
    (KeyCode::X, VirtualKey::Fire),
    (KeyCode::R, VirtualKey::DebugRestart),
];

pub struct Input {
    down: HashSet<VirtualKey>,
    pressed: HashSet<VirtualKey>,
}

impl Input {
    pub fn new() -> Self {
        Self {
            down: HashSet::new(),
            pressed: HashSet::new(),
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
    }

    pub fn is_down(&self, vk: VirtualKey) -> bool {
        self.down.contains(&vk)
    }

    pub fn is_pressed(&self, vk: VirtualKey) -> bool {
        self.pressed.contains(&vk)
    }

    pub fn reset(&mut self) {
        self.pressed.clear();
    }
}
