use crate::loader::LoadedMap;
use crate::messages::Messages;
use hecs::Entity;
use macroquad::prelude::*;

pub struct Resources {
    pub player_sprite: Texture2D,
    pub dog_sprite: Texture2D,
    pub pickup_sprite: Texture2D,
    pub ui_sprite: Texture2D,

    pub player_id: Entity,
    pub draw_order: Vec<Entity>,
    pub tileset_info: TilesetInfo,
    pub messages: Messages,
}

impl Resources {
    pub(crate) async fn new(map: &LoadedMap, player_id: Entity) -> Self {
        Self {
            player_sprite: load_texture("princess.png").await.unwrap(),
            dog_sprite: load_texture("robodog.png").await.unwrap(),
            pickup_sprite: load_texture("pickup.png").await.unwrap(),
            ui_sprite: load_texture("ui-heart.png").await.unwrap(),
            player_id,
            draw_order: map.draw_order.clone(),
            tileset_info: map.tileset_info.clone(),
            messages: Messages::new(),
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
