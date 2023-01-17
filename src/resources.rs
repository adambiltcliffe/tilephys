use crate::index::SpatialIndex;
use crate::messages::Messages;
use crate::render::load_flash_material;
use crate::scene::Scene;
use crate::script::ScriptEngine;
use crate::stats::LevelStats;
use crate::transition::TransitionEffectType;
use hecs::{Entity, World};
use macroquad::prelude::*;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

pub struct GlobalAssets {
    pub sky: Texture2D,
    pub player_sprite: Texture2D,
    pub dog_sprite: Texture2D,
    pub parrot_sprite: Texture2D,
    pub pickup_sprite: Texture2D,
    pub switch_sprite: Texture2D,
    pub ui_sprite: Texture2D,
    pub zap_sprite: Texture2D,
    pub interstitial: Texture2D,
    pub controls: Texture2D,
    pub flash_material: Material,
    // should this be here?
    pub next_scene: Option<(Scene, TransitionEffectType)>,
}

pub async fn load_assets() -> GlobalAssets {
    GlobalAssets {
        sky: load_texture("sky.png").await.unwrap(),
        player_sprite: load_texture("princess.png").await.unwrap(),
        dog_sprite: load_texture("robodog.png").await.unwrap(),
        parrot_sprite: load_texture("spiderparrot.png").await.unwrap(),
        pickup_sprite: load_texture("pickup.png").await.unwrap(),
        switch_sprite: load_texture("switch.png").await.unwrap(),
        ui_sprite: load_texture("ui-heart.png").await.unwrap(),
        zap_sprite: load_texture("zap.png").await.unwrap(),
        interstitial: load_texture("interstitial.png").await.unwrap(),
        controls: load_texture("controls.png").await.unwrap(),
        flash_material: load_flash_material(),
        next_scene: None,
    }
}

pub struct SceneResources {
    pub world_ref: Arc<Mutex<World>>,
    pub script_engine: ScriptEngine,
    pub player_id: Entity,
    pub eye_pos: Vec2,
    pub camera_pos: Vec2,
    pub draw_order: Vec<Entity>,
    pub body_index: SpatialIndex,
    pub tileset_info: TilesetInfo,
    pub messages: Messages,
    pub stats: LevelStats,
    pub triggers: HashSet<String>,
}

#[derive(Clone)]
pub struct TilesetInfo {
    pub texture: Texture2D,
    pub tile_width: u32,
    pub tile_height: u32,
    pub columns: u32,
}
