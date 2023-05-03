use macroquad::{
    input::{is_key_down, is_key_pressed, KeyCode},
    prelude::get_char_pressed,
};
use std::collections::HashSet;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    Pressed,
    Held,
    NotPressed,
}

#[derive(PartialEq, Hash, Eq, Clone, Copy)]
pub enum VirtualKey {
    Left,
    Right,
    Jump,
    Fire,
    Interact,
    PrevWeapon,
    NextWeapon,
    DebugRestart,
    DebugWin,
    DebugKill,
    DebugAmmo,
    DebugProfile,
    DebugConsole,
}

const ALL_KEYS: [(KeyCode, VirtualKey); 13] = [
    (KeyCode::Left, VirtualKey::Left),
    (KeyCode::Right, VirtualKey::Right),
    (KeyCode::Z, VirtualKey::Jump),
    (KeyCode::X, VirtualKey::Fire),
    (KeyCode::C, VirtualKey::Interact),
    (KeyCode::A, VirtualKey::PrevWeapon),
    (KeyCode::S, VirtualKey::NextWeapon),
    (KeyCode::R, VirtualKey::DebugRestart),
    (KeyCode::W, VirtualKey::DebugWin),
    (KeyCode::K, VirtualKey::DebugKill),
    (KeyCode::F, VirtualKey::DebugAmmo),
    (KeyCode::P, VirtualKey::DebugProfile),
    (KeyCode::GraveAccent, VirtualKey::DebugConsole),
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

    pub fn state(&self, vk: VirtualKey) -> KeyState {
        if self.pressed.contains(&vk) {
            return KeyState::Pressed;
        }
        if self.down.contains(&vk) {
            return KeyState::Held;
        }
        KeyState::NotPressed
    }

    pub fn is_any_pressed(&self) -> bool {
        self.any_pressed
    }

    pub fn reset(&mut self) {
        self.down.clear();
        self.pressed.clear();
        self.any_pressed = false;
        while get_char_pressed().is_some() {}
    }
}
