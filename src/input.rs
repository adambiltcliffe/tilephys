use crate::{
    physics::IntRect,
    render::{Renderer, WALL_VISION_DEPTH},
    RENDER_H, RENDER_W,
};
use enum_iterator::Sequence;
use macroquad::{
    input::{
        get_char_pressed, is_key_down, is_key_pressed, is_mouse_button_down,
        is_mouse_button_pressed, mouse_position, KeyCode, MouseButton,
    },
    texture::DrawTextureParams,
};
use std::{collections::HashSet, f32::consts::PI};

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
            x: WVDC + 48,
            y: RENDER_H as i32 - WVDC,
            w: 16,
            h: 16,
        },
        VirtualKey::Left,
    ),
    (
        IntRect {
            x: WVDC + 64,
            y: RENDER_H as i32 - WVDC,
            w: 16,
            h: 16,
        },
        VirtualKey::Right,
    ),
    (
        IntRect {
            x: WVDC + 56,
            y: RENDER_H as i32 - WVDC - 12,
            w: 16,
            h: 16,
        },
        VirtualKey::Jump,
    ),
];

pub struct KeyTrigger {
    kc: KeyCode,
    vk: VirtualKey,
}

impl KeyTrigger {
    pub fn new(kc: KeyCode, vk: VirtualKey) -> Self {
        Self { kc, vk }
    }

    pub fn is_down(&self) -> bool {
        is_key_down(self.kc)
    }

    pub fn is_pressed(&self) -> bool {
        is_key_pressed(self.kc)
    }
}

pub struct ClickTrigger {
    cl: IntRect,
    mb: MouseButton,
    vk: VirtualKey,
}

impl ClickTrigger {
    pub fn new(cl: IntRect, mb: MouseButton, vk: VirtualKey) -> Self {
        Self { cl, mb, vk }
    }

    pub fn is_hovered(&self, renderer: &Renderer) -> bool {
        let mouse_pos = renderer.format_abs_pos(mouse_position());
        let mouse_rect: IntRect = IntRect {
            x: mouse_pos.0.clamp(0.0, RENDER_W as f32).round() as i32,
            y: mouse_pos.1.clamp(0.0, RENDER_H as f32).round() as i32,
            w: 1,
            h: 1,
        };
        self.cl.intersects(&mouse_rect)
    }

    pub fn is_down(&self, renderer: &Renderer) -> bool {
        self.is_hovered(renderer) && is_mouse_button_down(self.mb)
    }

    pub fn is_pressed(&self, renderer: &Renderer) -> bool {
        self.is_hovered(renderer) && is_mouse_button_pressed(self.mb)
    }
}

#[derive(Sequence, Debug)]
pub enum ScreenButtons {
    Left,
    Jump,
    Right,
}

impl ScreenButtons {
    pub fn get_pos(&self, renderer: &Renderer) -> (i32, i32) {
        println!("{}", renderer.height);
        match self {
            Self::Left => (WVDC + 48, RENDER_H as i32 - WVDC),
            Self::Jump => (WVDC + 64, RENDER_H as i32 - WVDC - 16),
            Self::Right => (WVDC + 56, RENDER_H as i32 - WVDC),
        }
    }

    pub fn get_texture_params(&self) -> DrawTextureParams {
        match self {
            Self::Left => {
                DrawTextureParams {
                    ..Default::default()
                }
            },
            Self::Jump => {
                DrawTextureParams {
                    rotation: PI,
                    ..Default::default()
                }
            },
            Self::Right => {
                DrawTextureParams {
                    flip_x: true,
                    ..Default::default()
                }
            }
        }
    }
}

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
            if cl.intersects(&mouse_rect) {
                if is_mouse_button_down(MouseButton::Left) {
                    self.down.insert(*vk);
                }
                if is_mouse_button_pressed(MouseButton::Left) {
                    self.pressed.insert(*vk);
                }
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
        self.any_pressed = get_char_pressed().is_some() || is_mouse_button_down(MouseButton::Left);
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
