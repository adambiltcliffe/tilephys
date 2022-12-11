use std::collections::HashSet;

use crate::messages::Messages;
use crate::scene::NewScene;
use crate::script::ScriptEngine;
use crate::stats::LevelStats;
use crate::transition::TransitionEffectType;
use hecs::Entity;
use macroquad::prelude::*;

pub struct Resources {
    pub script_engine: ScriptEngine,

    pub player_sprite: Texture2D,
    pub dog_sprite: Texture2D,
    pub pickup_sprite: Texture2D,
    pub switch_sprite: Texture2D,
    pub ui_sprite: Texture2D,
    pub interstitial: Texture2D,

    pub player_id: Entity,
    pub eye_pos: Vec2,
    pub camera_pos: Vec2,
    pub draw_order: Vec<Entity>,
    pub tileset_info: TilesetInfo,
    pub messages: Messages,
    pub stats: LevelStats,
    pub triggers: HashSet<String>,
    pub new_scene: Option<(NewScene, TransitionEffectType)>,
}

#[derive(Clone)]
pub struct TilesetInfo {
    pub texture: Texture2D,
    pub tile_width: u32,
    pub tile_height: u32,
    pub columns: u32,
}
