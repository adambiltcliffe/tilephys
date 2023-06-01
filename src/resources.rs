use crate::camera::EyeballState;
use crate::index::SpatialIndex;
use crate::level::{load_level_info, LevelInfo};
use crate::messages::Messages;
use crate::render::load_flash_material;
use crate::scene::Scene;
use crate::script::ScriptEngine;
use crate::stats::LevelStats;
use crate::transition::TransitionEffectType;
use crate::weapon::{AmmoQuantity, AmmoType, Weapon, WeaponSelectorUI, WeaponType};
use anyhow::Context;
use enum_map::EnumMap;
use hecs::{Entity, World};
use macroquad::prelude::*;
use std::collections::{HashSet, VecDeque};
use std::num::NonZeroU8;
use std::sync::{Arc, Mutex};

pub struct GlobalAssets {
    pub title: Texture2D,
    pub sky: Texture2D,
    pub player_sprite: Texture2D,
    pub dog_sprite: Texture2D,
    pub parrot_sprite: Texture2D,
    pub parrot_sprite2: Texture2D,
    pub drone_sprite: Texture2D,
    pub boss_sprites: Texture2D,
    pub pickup_sprite: Texture2D,
    pub switch_sprite: Texture2D,
    pub ui_sprite: Texture2D,
    pub weapon_sprite: Texture2D,
    pub zap_sprite: Texture2D,
    pub interstitial: Texture2D,
    pub flash_material: Material,
    pub levels: Vec<LevelInfo>,
    // should this be here?
    pub next_scene: Option<(Scene, TransitionEffectType)>,
}

async fn load_texture(name: &str) -> anyhow::Result<macroquad::texture::Texture2D> {
    macroquad::texture::load_texture(name)
        .await
        .with_context(|| format!("Couldn't load texture '{}'", name))
}

pub async fn load_assets() -> anyhow::Result<GlobalAssets> {
    let levels = load_level_info().await?;
    Ok(GlobalAssets {
        title: load_texture("title.png").await?,
        sky: load_texture("sky.png").await?,
        player_sprite: load_texture("princess.png").await?,
        dog_sprite: load_texture("robodog.png").await?,
        parrot_sprite: load_texture("spiderparrot.png").await?,
        parrot_sprite2: load_texture("greenparrot.png").await?,
        drone_sprite: load_texture("drone.png").await?,
        boss_sprites: load_texture("bossparts.png").await?,
        pickup_sprite: load_texture("pickup.png").await?,
        switch_sprite: load_texture("switch.png").await?,
        ui_sprite: load_texture("ui-heart.png").await?,
        weapon_sprite: load_texture("weapons.png").await?,
        zap_sprite: load_texture("zap.png").await?,
        interstitial: load_texture("interstitial.png").await?,
        flash_material: load_flash_material(),
        levels,
        next_scene: None,
    })
}

pub struct SceneResources {
    pub world_ref: Arc<Mutex<World>>,
    pub script_engine: ScriptEngine,
    pub player_id: Entity,
    pub eye: EyeballState,
    pub camera_pos: Vec2,
    pub death_timer: Option<NonZeroU8>,
    pub draw_order: Vec<Entity>,
    pub body_index: SpatialIndex,
    pub tileset_info: TilesetInfo,
    pub messages: Messages,
    pub selector: WeaponSelectorUI,
    pub stats: LevelStats,
    pub triggers: HashSet<String>,
    pub weapons: VecDeque<Box<dyn Weapon>>,
    pub ammo: EnumMap<AmmoType, AmmoQuantity>,
}

impl SceneResources {
    pub fn persist_inventory(&self) -> Inventory {
        Inventory {
            weapon_types: self.weapons.iter().map(|w| w.get_type()).collect(),
            ammo: self.ammo,
            is_default: false,
        }
    }
}

#[derive(Clone)]
pub struct Inventory {
    pub weapon_types: Vec<WeaponType>,
    pub ammo: EnumMap<AmmoType, AmmoQuantity>,
    pub is_default: bool,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            weapon_types: vec![WeaponType::BackupLaser],
            ammo: EnumMap::default(),
            is_default: true,
        }
    }
}

#[derive(Clone)]
pub struct TilesetInfo {
    pub texture: Texture2D,
    pub tile_width: u32,
    pub tile_height: u32,
    pub columns: u32,
}
