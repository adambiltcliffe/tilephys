use crate::input::KeyState;
use crate::physics::{Actor, IntRect};
use crate::projectile::make_player_projectile;
use hecs::CommandBuffer;

#[derive(Copy, Clone)]
pub enum WeaponType {
    BackupLaser,
    ReverseLaser,
    AutoLaser,
    BurstLaser,
}

pub fn weapon_name(typ: WeaponType) -> &'static str {
    match typ {
        WeaponType::BackupLaser => "backup laser",
        WeaponType::ReverseLaser => "reverse laser",
        WeaponType::AutoLaser => "auto-laser",
        WeaponType::BurstLaser => "burst laser",
    }
}

pub trait Weapon {
    fn get_type(&self) -> WeaponType;
    fn update(
        &mut self,
        buffer: &mut CommandBuffer,
        player: &mut Actor,
        player_rect: &IntRect,
        facing: i8,
        key_state: KeyState,
    ) -> bool;
}

struct BackupLaser {}

impl BackupLaser {
    fn new() -> Self {
        Self {}
    }
}

impl Weapon for BackupLaser {
    fn get_type(&self) -> WeaponType {
        WeaponType::BackupLaser
    }
    fn update(
        &mut self,
        buffer: &mut CommandBuffer,
        player: &mut Actor,
        player_rect: &IntRect,
        facing: i8,
        key_state: KeyState,
    ) -> bool {
        if key_state == KeyState::Pressed {
            let new_x = player_rect.x + 3 + facing as i32 * 9;
            let rect = IntRect::new(new_x, player_rect.y + 11, 8, 5);
            make_player_projectile(buffer, rect, facing as f32 * 10.0);
            player.vx -= facing as f32 * 10.0;
            return true;
        }
        false
    }
}

struct ReverseLaser {}

impl ReverseLaser {
    fn new() -> Self {
        Self {}
    }
}

impl Weapon for ReverseLaser {
    fn get_type(&self) -> WeaponType {
        WeaponType::ReverseLaser
    }
    fn update(
        &mut self,
        buffer: &mut CommandBuffer,
        player: &mut Actor,
        player_rect: &IntRect,
        facing: i8,
        key_state: KeyState,
    ) -> bool {
        if key_state == KeyState::Pressed {
            let new_x = player_rect.x + 3 - facing as i32 * 9;
            let rect = IntRect::new(new_x, player_rect.y + 11, 8, 5);
            make_player_projectile(buffer, rect, facing as f32 * -10.0);
            player.vx += facing as f32 * 10.0;
            return true;
        }
        false
    }
}

struct AutoLaser {
    delay: u8,
}

impl AutoLaser {
    fn new() -> Self {
        Self { delay: 0 }
    }
}

impl Weapon for AutoLaser {
    fn get_type(&self) -> WeaponType {
        WeaponType::AutoLaser
    }
    fn update(
        &mut self,
        buffer: &mut CommandBuffer,
        player: &mut Actor,
        player_rect: &IntRect,
        facing: i8,
        key_state: KeyState,
    ) -> bool {
        if self.delay > 0 {
            self.delay -= 1
        }
        if key_state != KeyState::NotPressed && self.delay == 0 {
            let new_x = player_rect.x + 3 + facing as i32 * 9;
            let rect = IntRect::new(new_x, player_rect.y + 11, 8, 5);
            make_player_projectile(buffer, rect, facing as f32 * 10.0);
            player.vx -= facing as f32 * 10.0;
            self.delay = 3;
            return true;
        }
        false
    }
}

struct BurstLaser {
    delay: u8,
    shots: u8,
}

impl BurstLaser {
    fn new() -> Self {
        Self { delay: 0, shots: 0 }
    }
}

impl Weapon for BurstLaser {
    fn get_type(&self) -> WeaponType {
        WeaponType::BurstLaser
    }
    fn update(
        &mut self,
        buffer: &mut CommandBuffer,
        player: &mut Actor,
        player_rect: &IntRect,
        facing: i8,
        key_state: KeyState,
    ) -> bool {
        if self.delay > 0 {
            self.delay -= 1
        }
        if key_state != KeyState::NotPressed && self.delay == 0 && self.shots < 3 {
            let new_x = player_rect.x + 3 + facing as i32 * 9;
            let rect = IntRect::new(new_x, player_rect.y + 11, 8, 5);
            make_player_projectile(buffer, rect, facing as f32 * 10.0);
            player.vx -= facing as f32 * 10.0;
            self.delay = 2;
            self.shots += 1;
            return true;
        }
        if key_state == KeyState::NotPressed {
            self.shots = 0;
        }
        false
    }
}

pub fn new_weapon(typ: WeaponType) -> Box<dyn Weapon> {
    match typ {
        WeaponType::BackupLaser => Box::new(BackupLaser::new()),
        WeaponType::ReverseLaser => Box::new(ReverseLaser::new()),
        WeaponType::AutoLaser => Box::new(AutoLaser::new()),
        WeaponType::BurstLaser => Box::new(BurstLaser::new()),
    }
}
