use crate::loader::LoadedMap;
use crate::messages::Messages;
use hecs::Entity;
use macroquad::prelude::*;

pub struct Resources {
    pub player_sprite: Texture2D,
    pub dog_sprite: Texture2D,
    pub draw_order: Vec<Entity>,
    pub messages: Messages,
}

impl Resources {
    pub(crate) async fn new(map: &LoadedMap) -> Self {
        Self {
            player_sprite: load_texture("princess.png").await.unwrap(),
            dog_sprite: load_texture("robodog.png").await.unwrap(),
            draw_order: map.draw_order.clone(),
            messages: Messages::new(),
        }
    }
}
