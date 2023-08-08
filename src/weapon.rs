use crate::config::config;
use crate::index::SpatialIndex;
use crate::input::KeyState;
use crate::physics::{collide_any, Actor, IntRect};
use crate::projectile::{
    make_napalm, make_player_fireball, make_player_laser, make_railgun_hitbox, make_smoke,
    DamageEnemies, Projectile, ProjectileDrag,
};
use crate::ray::ray_collision;
use crate::vfx::{make_railgun_trail, FireballEffect, SmokeParticle};
use enum_map::EnumMap;
use hecs::{CommandBuffer, World};
use macroquad::math::Vec2;
use std::collections::VecDeque;
use std::sync::MutexGuard;

pub enum FiringResult {
    No,
    Yes(bool),
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum WeaponType {
    BackupLaser,
    BurstLaser,
    AutoLaser,
    DoubleLaser,
    ValkyrieLaser,
    Shotgun,
    SuperShotgun,
    ReverseShotgun,
    Railgun,
    Flamer,
    Smoker,
}

pub fn weapon_name(typ: WeaponType) -> &'static str {
    match typ {
        WeaponType::BackupLaser => "backup laser",
        WeaponType::AutoLaser => "auto-laser",
        WeaponType::BurstLaser => "burst laser",
        WeaponType::DoubleLaser => "double laser",
        WeaponType::ValkyrieLaser => "Valkyrie laser",
        WeaponType::Shotgun => "shotgun",
        WeaponType::SuperShotgun => "super shotgun",
        WeaponType::ReverseShotgun => "reverse shotgun",
        WeaponType::Railgun => "railgun",
        WeaponType::Flamer => "flamer",
        WeaponType::Smoker => "smoker",
    }
}

pub fn weapon_name_indef(typ: WeaponType) -> &'static str {
    match typ {
        WeaponType::BackupLaser => unreachable!(),
        WeaponType::AutoLaser => "an auto-laser",
        WeaponType::BurstLaser => "a burst laser",
        WeaponType::DoubleLaser => "a double laser",
        WeaponType::ValkyrieLaser => "a Valkyrie laser",
        WeaponType::Shotgun => "a shotgun",
        WeaponType::SuperShotgun => "a super shotgun",
        WeaponType::ReverseShotgun => "the reverse shotgun",
        WeaponType::Railgun => "a railgun",
        WeaponType::Flamer => "a flamer",
        WeaponType::Smoker => "a smoker",
    }
}

pub fn weapon_sprite_frame(typ: WeaponType) -> usize {
    match typ {
        WeaponType::BackupLaser => 0,
        WeaponType::AutoLaser => 2,
        WeaponType::BurstLaser => 1,
        WeaponType::DoubleLaser => 8,
        WeaponType::ValkyrieLaser => 7,
        WeaponType::Shotgun => 3,
        WeaponType::SuperShotgun => 4,
        WeaponType::ReverseShotgun => 5,
        WeaponType::Railgun => 6,
        WeaponType::Flamer => 9,
        WeaponType::Smoker => 10,
    }
}

pub fn weapon_v_offset(typ: WeaponType) -> f32 {
    match typ {
        WeaponType::BackupLaser => 4.0,
        WeaponType::AutoLaser => 3.0,
        WeaponType::BurstLaser => 3.0,
        WeaponType::DoubleLaser => 3.0,
        WeaponType::ValkyrieLaser => 2.0,
        WeaponType::Shotgun => 4.0,
        WeaponType::SuperShotgun => 3.0,
        WeaponType::ReverseShotgun => 1.0,
        WeaponType::Railgun => 3.0,
        WeaponType::Flamer => 1.0,
        WeaponType::Smoker => 0.0,
    }
}

#[derive(enum_iterator::Sequence, enum_map::Enum, Copy, Clone)]
pub enum AmmoType {
    Cell,
    Shell,
    Rocket,
    Slug,
    Fuel,
}

pub type AmmoQuantity = u8;

pub fn ammo_symbol(typ: AmmoType) -> &'static str {
    match typ {
        AmmoType::Cell => "CEL",
        AmmoType::Shell => "SHL",
        AmmoType::Rocket => "RKT",
        AmmoType::Slug => "SLG",
        AmmoType::Fuel => "FUE",
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
        AmmoType::Slug => "railgun slugs",
        AmmoType::Fuel => "omnifuel",
    }
}

pub fn ammo_max(typ: AmmoType) -> AmmoQuantity {
    match typ {
        AmmoType::Cell => 99,
        AmmoType::Shell => 40,
        AmmoType::Rocket => 20,
        AmmoType::Slug => 50,
        AmmoType::Fuel => 99,
    }
}

pub trait Weapon {
    fn get_type(&self) -> WeaponType;
    fn get_ammo_type(&self) -> AmmoType;
    fn get_ammo_use(&self) -> AmmoQuantity;
    fn update(
        &mut self,
        world_ref: &MutexGuard<World>,
        body_index: &SpatialIndex,
        buffer: &mut CommandBuffer,
        player: &mut Actor,
        player_rect: &IntRect,
        facing: i8,
        fire_timer: u32,
        key_state: KeyState,
    ) -> FiringResult;
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
        _world: &MutexGuard<World>,
        _body_index: &SpatialIndex,
        buffer: &mut CommandBuffer,
        player: &mut Actor,
        player_rect: &IntRect,
        facing: i8,
        _fire_timer: u32,
        key_state: KeyState,
    ) -> FiringResult {
        if key_state == KeyState::Pressed {
            let new_x = player_rect.x + 3 + facing as i32 * 9;
            let rect = IntRect::new(new_x, player_rect.y + 11, 8, 5);
            make_player_laser(buffer, rect, facing as f32 * 10.0, 0.0);
            player.vx -= facing as f32 * config().recoil();
            return FiringResult::Yes(true);
        }
        FiringResult::No
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
            ProjectileDrag::new(2.0),
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
        _world: &MutexGuard<World>,
        _body_index: &SpatialIndex,
        buffer: &mut CommandBuffer,
        player: &mut Actor,
        player_rect: &IntRect,
        facing: i8,
        fire_timer: u32,
        key_state: KeyState,
    ) -> FiringResult {
        if fire_timer < config().ft_shotgun() as u32 {
            return FiringResult::No;
        }
        if key_state == KeyState::Pressed {
            make_shotgun_spray(
                buffer,
                player_rect.x + 3,
                player_rect.y + 11,
                facing,
                7,
                5.0,
            );
            player.vx -= facing as f32 * config().recoil();
            return FiringResult::Yes(false);
        }
        FiringResult::No
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
        _world: &MutexGuard<World>,
        _body_index: &SpatialIndex,
        buffer: &mut CommandBuffer,
        player: &mut Actor,
        player_rect: &IntRect,
        facing: i8,
        fire_timer: u32,
        key_state: KeyState,
    ) -> FiringResult {
        if fire_timer < config().ft_super_shotgun() as u32 {
            return FiringResult::No;
        }
        if key_state == KeyState::Pressed {
            make_shotgun_spray(
                buffer,
                player_rect.x + 3,
                player_rect.y + 11,
                facing,
                15,
                10.0,
            );
            player.vx -= facing as f32 * config().recoil() * 2.0;
            return FiringResult::Yes(false);
        }
        FiringResult::No
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
        _world: &MutexGuard<World>,
        _body_index: &SpatialIndex,
        buffer: &mut CommandBuffer,
        player: &mut Actor,
        player_rect: &IntRect,
        facing: i8,
        fire_timer: u32,
        key_state: KeyState,
    ) -> FiringResult {
        if fire_timer < config().ft_shotgun() as u32 {
            return FiringResult::No;
        }
        if key_state == KeyState::Pressed {
            make_shotgun_spray(
                buffer,
                player_rect.x + 3 + facing as i32 * 20,
                player_rect.y + 11,
                -facing,
                7,
                5.0,
            );
            player.vx += facing as f32 * config().recoil();
            return FiringResult::Yes(false);
        }
        FiringResult::No
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
        _world: &MutexGuard<World>,
        _body_index: &SpatialIndex,
        buffer: &mut CommandBuffer,
        player: &mut Actor,
        player_rect: &IntRect,
        facing: i8,
        _fire_timer: u32,
        key_state: KeyState,
    ) -> FiringResult {
        if self.delay > 0 {
            self.delay -= 1
        }
        if key_state != KeyState::NotPressed && self.delay == 0 {
            let new_x = player_rect.x + 3 + facing as i32 * 9;
            let rect = IntRect::new(new_x, player_rect.y + 11, 8, 5);
            make_player_laser(buffer, rect, facing as f32 * 10.0, 0.0);
            player.vx -= facing as f32 * config().recoil();
            self.delay = 3;
            return FiringResult::Yes(true);
        }
        FiringResult::No
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
        _world: &MutexGuard<World>,
        _body_index: &SpatialIndex,
        buffer: &mut CommandBuffer,
        player: &mut Actor,
        player_rect: &IntRect,
        facing: i8,
        _fire_timer: u32,
        key_state: KeyState,
    ) -> FiringResult {
        if self.delay > 0 {
            self.delay -= 1
        }
        if key_state != KeyState::NotPressed && self.delay == 0 && self.shots < 3 {
            let new_x = player_rect.x + 3 + facing as i32 * 9;
            let rect = IntRect::new(new_x, player_rect.y + 11, 8, 5);
            make_player_laser(buffer, rect, facing as f32 * 10.0, 0.0);
            player.vx -= facing as f32 * config().recoil();
            self.delay = 2;
            self.shots += 1;
            return FiringResult::Yes(true);
        }
        if key_state == KeyState::NotPressed {
            self.shots = 0;
        }
        FiringResult::No
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
        _world: &MutexGuard<World>,
        _body_index: &SpatialIndex,
        buffer: &mut CommandBuffer,
        player: &mut Actor,
        player_rect: &IntRect,
        facing: i8,
        _fire_timer: u32,
        key_state: KeyState,
    ) -> FiringResult {
        if key_state == KeyState::Pressed {
            let new_x = player_rect.x + 3 + facing as i32 * 9;
            let rect = IntRect::new(new_x, player_rect.y + 8, 8, 5);
            make_player_laser(buffer, rect, facing as f32 * 99.0_f32.sqrt(), -1.0);
            let rect = IntRect::new(new_x, player_rect.y + 14, 8, 5);
            make_player_laser(buffer, rect, facing as f32 * 99.0_f32.sqrt(), 1.0);
            player.vx -= facing as f32 * config().recoil();
            return FiringResult::Yes(true);
        }
        FiringResult::No
    }
}

struct ValkyrieLaser {
    delay: u8,
    count: u8,
}

impl ValkyrieLaser {
    fn new() -> Self {
        Self { delay: 0, count: 0 }
    }
}

impl Weapon for ValkyrieLaser {
    fn get_type(&self) -> WeaponType {
        WeaponType::ValkyrieLaser
    }
    fn get_ammo_type(&self) -> AmmoType {
        AmmoType::Cell
    }
    fn get_ammo_use(&self) -> AmmoQuantity {
        1
    }
    fn update(
        &mut self,
        _world: &MutexGuard<World>,
        _body_index: &SpatialIndex,
        buffer: &mut CommandBuffer,
        player: &mut Actor,
        player_rect: &IntRect,
        facing: i8,
        _fire_timer: u32,
        key_state: KeyState,
    ) -> FiringResult {
        if self.delay > 0 {
            self.delay -= 1
        }
        if key_state != KeyState::NotPressed && self.delay == 0 {
            let new_x = player_rect.x + 3 + facing as i32 * 9;
            let rect = IntRect::new(new_x, player_rect.y + 11, 8, 5);
            let max_vy = config().valkyrie_max_vy();
            let vy = quad_rand::gen_range::<f32>(0.0, max_vy);
            let vx = (100.0_f32 - vy.powf(2.0)).sqrt();
            if self.count % 3 == 1 {
                make_player_fireball(buffer, rect, facing as f32 * vx * 0.5, vy);
            } else {
                make_player_laser(buffer, rect, facing as f32 * vx, vy);
            }
            let recoil = config().recoil() / 15.0;
            player.vx -= facing as f32 * recoil * vx;
            player.vy -= recoil * (vy / max_vy) + config().valkyrie_lift();
            self.delay = 2;
            self.count += 1;
            return FiringResult::Yes(true);
        }
        if key_state == KeyState::NotPressed {
            self.count = 0;
        }
        FiringResult::No
    }
}

struct Railgun {}

impl Railgun {
    fn new() -> Self {
        Self {}
    }
}

impl Weapon for Railgun {
    fn get_type(&self) -> WeaponType {
        WeaponType::Railgun
    }
    fn get_ammo_type(&self) -> AmmoType {
        AmmoType::Slug
    }
    fn get_ammo_use(&self) -> AmmoQuantity {
        1
    }
    fn update(
        &mut self,
        world: &MutexGuard<World>,
        body_index: &SpatialIndex,
        buffer: &mut CommandBuffer,
        player: &mut Actor,
        player_rect: &IntRect,
        facing: i8,
        fire_timer: u32,
        key_state: KeyState,
    ) -> FiringResult {
        if fire_timer < config().ft_railgun() as u32 {
            return FiringResult::No;
        }
        if key_state == KeyState::Pressed {
            let xoff1 = config().rg_xoff1();
            let xoff2 = config().rg_xoff2();
            let yoff = config().rg_yoff();
            let new_x = player_rect.x + xoff1 + facing as i32 * xoff2;
            let y = player_rect.y + yoff;
            // if the shot would originate inside a wall, don't create it, because the ray collision
            // won't stop it from passing through a wall it's already inside
            if !collide_any(&*world, body_index, &IntRect::new(new_x, y, 1, 1)) {
                let orig = Vec2::new(new_x as f32, y as f32);
                let disp = Vec2::new(300.0 * facing as f32, 0.0);
                let dest = match ray_collision(&*world, body_index, &orig, &(orig + disp)) {
                    None => orig + disp,
                    Some((v, _)) => v,
                };
                make_railgun_hitbox(buffer, orig.x, orig.y, dest.x, dest.y);
                make_railgun_trail(buffer, orig.x, orig.y, dest.x, dest.y);
            }
            player.vx -= facing as f32 * config().recoil();
            return FiringResult::Yes(true);
        }
        FiringResult::No
    }
}

struct Flamer {
    delay: u8,
}

impl Flamer {
    fn new() -> Self {
        Self { delay: 0 }
    }
}

impl Weapon for Flamer {
    fn get_type(&self) -> WeaponType {
        WeaponType::Flamer
    }
    fn get_ammo_type(&self) -> AmmoType {
        AmmoType::Fuel
    }
    fn get_ammo_use(&self) -> AmmoQuantity {
        1
    }
    fn update(
        &mut self,
        _world: &MutexGuard<World>,
        _body_index: &SpatialIndex,
        buffer: &mut CommandBuffer,
        _player: &mut Actor,
        player_rect: &IntRect,
        facing: i8,
        _fire_timer: u32,
        key_state: KeyState,
    ) -> FiringResult {
        if self.delay > 0 {
            self.delay -= 1
        }
        if key_state != KeyState::NotPressed {
            let dx = config().flamer_dx();
            let dxv = config().flamer_dxv();
            let dyv = config().flamer_dyv();
            let new_x = player_rect.x + 3 + facing as i32 * 9;
            let rect = IntRect::new(new_x, player_rect.y + 11, 8, 8);
            make_napalm(
                buffer,
                rect,
                facing as f32 * (dx + quad_rand::gen_range(-dxv, dxv)),
                quad_rand::gen_range(-dyv, dyv),
                self.delay == 0,
            );
            if self.delay == 0 {
                self.delay = 4;
            }
            return FiringResult::No;
        }
        FiringResult::No
    }
}

struct Smoker {
    delay: u8,
}

impl Smoker {
    fn new() -> Self {
        Self { delay: 0 }
    }
}

impl Weapon for Smoker {
    fn get_type(&self) -> WeaponType {
        WeaponType::Smoker
    }
    fn get_ammo_type(&self) -> AmmoType {
        AmmoType::Fuel
    }
    fn get_ammo_use(&self) -> AmmoQuantity {
        1
    }
    fn update(
        &mut self,
        _world: &MutexGuard<World>,
        _body_index: &SpatialIndex,
        buffer: &mut CommandBuffer,
        _player: &mut Actor,
        player_rect: &IntRect,
        facing: i8,
        _fire_timer: u32,
        key_state: KeyState,
    ) -> FiringResult {
        if self.delay > 0 {
            self.delay -= 1
        }
        if key_state != KeyState::NotPressed && self.delay == 0 {
            let new_x = player_rect.x + 3 + facing as i32 * 9;
            let rect = IntRect::new(new_x, player_rect.y + 11, 8, 5);
            make_smoke(buffer, rect, facing as f32 * 5.0, 0.0);
            self.delay = 4;
            return FiringResult::Yes(true);
        }
        FiringResult::No
    }
}

pub fn new_weapon(typ: WeaponType) -> Box<dyn Weapon> {
    match typ {
        WeaponType::BackupLaser => Box::new(BackupLaser::new()),
        WeaponType::AutoLaser => Box::new(AutoLaser::new()),
        WeaponType::BurstLaser => Box::new(BurstLaser::new()),
        WeaponType::DoubleLaser => Box::new(DoubleLaser::new()),
        WeaponType::ValkyrieLaser => Box::new(ValkyrieLaser::new()),
        WeaponType::Shotgun => Box::new(Shotgun::new()),
        WeaponType::SuperShotgun => Box::new(SuperShotgun::new()),
        WeaponType::ReverseShotgun => Box::new(ReverseShotgun::new()),
        WeaponType::Railgun => Box::new(Railgun::new()),
        WeaponType::Flamer => Box::new(Flamer::new()),
        WeaponType::Smoker => Box::new(Smoker::new()),
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
    for idx in 0..weapons.len() {
        let (t, u) = {
            let w = &weapons[idx];
            (w.get_ammo_type(), w.get_ammo_use())
        };
        if ammo[t] >= u {
            if idx == 0 {
                // our previous selected weapon was a backup laser that was just removed
                // so the current weapon was actually fireable, but we need to show a
                // visual as it wasn't selected a moment ago
                selector.change(-1.0);
            } else {
                weapons.rotate_left(idx);
                selector.change(-(idx as f32));
            }
            return;
        }
    }
    // if we couldn't find anything, add a backup laser to inventory
    weapons.push_front(new_weapon(WeaponType::BackupLaser));
    selector.change(-1.0);
}
