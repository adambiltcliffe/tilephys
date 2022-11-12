use macroquad::prelude::*;

pub struct Resources {
    pub player_sprite: Texture2D,
    pub dog_sprite: Texture2D,
}

impl Resources {
    pub async fn new() -> Self {
        Self {
            player_sprite: load_texture("princess.png").await.unwrap(),
            dog_sprite: load_texture("robodog.png").await.unwrap(),
        }
    }
}
