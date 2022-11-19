use crate::loader::LoadedMap;
use crate::messages::Messages;
use hecs::Entity;
use macroquad::prelude::*;

pub struct Resources {
    pub player_sprite: Texture2D,
    pub dog_sprite: Texture2D,
    pub ui_sprite: Texture2D,

    pub player_id: Entity,
    pub draw_order: Vec<Entity>,
    pub messages: Messages,
}

impl Resources {
    pub(crate) async fn new(map: &LoadedMap, player_id: Entity) -> Self {
        Self {
            player_sprite: load_texture("princess.png").await.unwrap(),
            dog_sprite: load_texture("robodog.png").await.unwrap(),
            ui_sprite: load_texture("ui-heart.png").await.unwrap(),
            player_id,
            draw_order: map.draw_order.clone(),
            messages: Messages::new(),
        }
    }
}
