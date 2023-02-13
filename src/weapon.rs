use crate::input::KeyState;
use crate::physics::{Actor, IntRect};
use crate::projectile::{make_player_projectile, DamageEnemies, Projectile, ProjectileDrag};
use crate::vfx::{FireballEffect, SmokeParticle};
use enum_map::EnumMap;
use hecs::CommandBuffer;
use std::collections::VecDeque;

// eventually there will be variants whose names don't end in "...Laser"
#[allow(clippy::enum_variant_names)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum WeaponType {
    BackupLaser,
    BurstLaser,
    AutoLaser,
    DoubleLaser,
    Shotgun,
    SuperShotgun,
    ReverseShotgun,
}

pub fn weapon_name(typ: WeaponType) -> &'static str {
    match typ {
        WeaponType::BackupLaser => "backup laser",
        WeaponType::AutoLaser => "auto-laser",
        WeaponType::BurstLaser => "burst laser",
        WeaponType::DoubleLaser => "double laser",
        WeaponType::Shotgun => "shotgun",
        WeaponType::SuperShotgun => "super shotgun",
        WeaponType::ReverseShotgun => "reverse shotgun",
    }
}

pub fn weapon_name_indef(typ: WeaponType) -> &'static str {
    match typ {
        WeaponType::BackupLaser => unreachable!(),
        WeaponType::AutoLaser => "an auto-laser",
        WeaponType::BurstLaser => "a burst laser",
        WeaponType::DoubleLaser => "a double laser",
        WeaponType::Shotgun => "a shotgun",
        WeaponType::SuperShotgun => "a super shotgun",
        WeaponType::ReverseShotgun => "the reverse shotgun",
    }
}

pub fn weapon_sprite_frame(typ: WeaponType) -> usize {
    match typ {
        WeaponType::BackupLaser => 0,
        WeaponType::AutoLaser => 2,
        WeaponType::BurstLaser => 1,
        WeaponType::DoubleLaser => 7,
        WeaponType::Shotgun => 3,
        WeaponType::SuperShotgun => 4,
        WeaponType::ReverseShotgun => 5,
    }
}

pub fn weapon_v_offset(typ: WeaponType) -> f32 {
    match typ {
        WeaponType::BackupLaser => 4.0,
        WeaponType::AutoLaser => 3.0,
        WeaponType::BurstLaser => 3.0,
        WeaponType::DoubleLaser => 2.0,
        WeaponType::Shotgun => 4.0,
        WeaponType::SuperShotgun => 3.0,
        WeaponType::ReverseShotgun => 1.0,
    }
}

#[derive(enum_iterator::Sequence, enum_map::Enum, Copy, Clone)]
pub enum AmmoType {
    Cell,
    Shell,
    Rocket,
}

pub type AmmoQuantity = u8;

pub fn ammo_symbol(typ: AmmoType) -> &'static str {
    match typ {
        AmmoType::Cell => "CEL",
        AmmoType::Shell => "SHL",
        AmmoType::Rocket => "RKT",
    }
}

pub fn ammo_name(typ: AmmoType, amt: AmmoQuantity) -> &'static str {
    // currently only rockets are available individually
    match typ {
        AmmoType::Cell => "laser cells",
        AmmoType::Shell => "shotgun shells",
        AmmoType::Rocket => {
            if amt == 1 {
                "a rocket"
            } else {
                "rockets"
            }
        }
    }
}

pub fn ammo_max(typ: AmmoType) -> AmmoQuantity {
    match typ {
        AmmoType::Cell => 99,
        AmmoType::Shell => 40,
        AmmoType::Rocket => 20,
    }
}

pub trait Weapon {
    fn get_type(&self) -> WeaponType;
    fn get_ammo_type(&self) -> AmmoType;
    fn get_ammo_use(&self) -> AmmoQuantity;
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
    fn get_ammo_type(&self) -> AmmoType {
        AmmoType::Cell
    }
    fn get_ammo_use(&self) -> AmmoQuantity {
        0
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

fn make_shotgun_spray(
    buffer: &mut CommandBuffer,
    x: i32,
    y: i32,
    facing: i8,
    n: usize,
    spread: f32,
) {
    let rect = IntRect::new(x + facing as i32 * 9, y, 5, 5);
    let vx = facing as f32 * 15.0;
    for i in 0..n {
        let c = rect.clone();
        let proj = Projectile::new(
            &c,
            vx * quad_rand::gen_range(0.1, 1.0),
            ((i as f32 / (n - 1) as f32) - 0.5) * spread * quad_rand::gen_range(0.8, 1.2),
        );
        buffer.spawn((
            c,
            FireballEffect::new(3.0),
            proj,
            DamageEnemies {},
            ProjectileDrag {},
        ));
    }
    for _ in 0..(n / 2) {
        buffer.spawn((SmokeParticle::new_from_centre(
            x + 2,
            y + 2,
            std::f32::consts::PI / -2.0 + quad_rand::gen_range(-0.3, 0.3),
            4.0,
        ),));
    }
}

struct Shotgun {}

impl Shotgun {
    fn new() -> Self {
        Self {}
    }
}

impl Weapon for Shotgun {
    fn get_type(&self) -> WeaponType {
        WeaponType::Shotgun
    }
    fn get_ammo_type(&self) -> AmmoType {
        AmmoType::Shell
    }
    fn get_ammo_use(&self) -> AmmoQuantity {
        1
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
            make_shotgun_spray(
                buffer,
                player_rect.x + 3,
                player_rect.y + 11,
                facing,
                7,
                5.0,
            );
            player.vx -= facing as f32 * 10.0;
            return true;
        }
        false
    }
}
struct SuperShotgun {}

impl SuperShotgun {
    fn new() -> Self {
        Self {}
    }
}

impl Weapon for SuperShotgun {
    fn get_type(&self) -> WeaponType {
        WeaponType::SuperShotgun
    }
    fn get_ammo_type(&self) -> AmmoType {
        AmmoType::Shell
    }
    fn get_ammo_use(&self) -> AmmoQuantity {
        2
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
            make_shotgun_spray(
                buffer,
                player_rect.x + 3,
                player_rect.y + 11,
                facing,
                15,
                10.0,
            );
            player.vx -= facing as f32 * 20.0;
            return true;
        }
        false
    }
}

struct ReverseShotgun {}

impl ReverseShotgun {
    fn new() -> Self {
        Self {}
    }
}

impl Weapon for ReverseShotgun {
    fn get_type(&self) -> WeaponType {
        WeaponType::ReverseShotgun
    }
    fn get_ammo_type(&self) -> AmmoType {
        AmmoType::Shell
    }
    fn get_ammo_use(&self) -> AmmoQuantity {
        1
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
            make_shotgun_spray(
                buffer,
                player_rect.x + 3 + facing as i32 * 20,
                player_rect.y + 11,
                -facing,
                7,
                5.0,
            );
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
    fn get_ammo_type(&self) -> AmmoType {
        AmmoType::Cell
    }
    fn get_ammo_use(&self) -> AmmoQuantity {
        1
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
    fn get_ammo_type(&self) -> AmmoType {
        AmmoType::Cell
    }
    fn get_ammo_use(&self) -> AmmoQuantity {
        1
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

struct DoubleLaser {}

impl DoubleLaser {
    fn new() -> Self {
        Self {}
    }
}

impl Weapon for DoubleLaser {
    fn get_type(&self) -> WeaponType {
        WeaponType::DoubleLaser
    }
    fn get_ammo_type(&self) -> AmmoType {
        AmmoType::Cell
    }
    fn get_ammo_use(&self) -> AmmoQuantity {
        2
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
            let rect = IntRect::new(new_x, player_rect.y + 8, 8, 5);
            make_player_projectile(buffer, rect, facing as f32 * 10.0);
            let rect = IntRect::new(new_x, player_rect.y + 14, 8, 5);
            make_player_projectile(buffer, rect, facing as f32 * 10.0);
            player.vx -= facing as f32 * 10.0;
            return true;
        }
        false
    }
}

pub fn new_weapon(typ: WeaponType) -> Box<dyn Weapon> {
    match typ {
        WeaponType::BackupLaser => Box::new(BackupLaser::new()),
        WeaponType::AutoLaser => Box::new(AutoLaser::new()),
        WeaponType::BurstLaser => Box::new(BurstLaser::new()),
        WeaponType::DoubleLaser => Box::new(DoubleLaser::new()),
        WeaponType::Shotgun => Box::new(Shotgun::new()),
        WeaponType::SuperShotgun => Box::new(SuperShotgun::new()),
        WeaponType::ReverseShotgun => Box::new(ReverseShotgun::new()),
    }
}

pub struct WeaponSelectorUI {
    pub timer: u16,
    pub offset: f32,
    pub hidden: bool,
}

impl WeaponSelectorUI {
    pub fn new() -> Self {
        Self {
            timer: 0,
            offset: 0.0,
            hidden: false,
        }
    }

    pub fn change(&mut self, delta: f32) {
        self.timer = 45;
        self.offset += delta;
    }

    pub fn update(&mut self) {
        if self.timer > 0 {
            self.timer -= 1;
        }
        self.offset *= 0.8;
    }
}

pub fn add_ammo(
    weapons: &mut VecDeque<Box<dyn Weapon>>,
    ammo: &mut EnumMap<AmmoType, AmmoQuantity>,
    selector: &mut WeaponSelectorUI,
    typ: AmmoType,
    amt: AmmoQuantity,
) {
    ammo[typ] = (ammo[typ] + amt).min(ammo_max(typ));
    if let Some(n) = weapons
        .iter()
        .position(|w| w.get_type() == WeaponType::BackupLaser)
    {
        // there is a backup laser in inventory at position n
        // we should remove it if the player can now use anything else
        if weapons.iter().any(|w| {
            ammo[w.get_ammo_type()] >= w.get_ammo_use() && w.get_type() != WeaponType::BackupLaser
        }) {
            weapons.remove(n);
            if n == 0 {
                // backup laser was previously the selected weapon
                select_fireable_weapon(weapons, ammo, selector)
            }
        }
    }
}

pub fn select_fireable_weapon(
    weapons: &mut VecDeque<Box<dyn Weapon>>,
    ammo: &mut EnumMap<AmmoType, AmmoQuantity>,
    selector: &mut WeaponSelectorUI,
) {
    for idx in 1..weapons.len() {
        let (t, u) = {
            let w = &weapons[idx];
            (w.get_ammo_type(), w.get_ammo_use())
        };
        if ammo[t] >= u {
            weapons.rotate_left(idx);
            selector.change(-(idx as f32));
            return;
        }
    }
    // if we couldn't find anything, add a backup laser to inventory
    weapons.push_front(new_weapon(WeaponType::BackupLaser));
    selector.change(-1.0);
}
