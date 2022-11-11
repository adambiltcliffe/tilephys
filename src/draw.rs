use crate::loader::TilesetInfo;
use crate::physics::{IntRect, TileBody};
use hecs::{Entity, World};
use macroquad::prelude::*;

pub(crate) struct ColorRect {
    color: Color,
}

impl ColorRect {
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

pub(crate) struct PlayerSprite {
    pub n: i32,
    pub firing: bool,
    pub flipped: bool,
}

impl PlayerSprite {
    pub fn new() -> Self {
        Self {
            n: 0,
            firing: false,
            flipped: true,
        }
    }
}

pub(crate) struct DogSprite {
    pub n: i32,
    pub flipped: bool,
}

impl DogSprite {
    pub fn new() -> Self {
        Self {
            n: 0,
            flipped: false,
        }
    }
}

pub(crate) fn draw(
    world: &mut World,
    tsi: &TilesetInfo,
    tex: &[Texture2D; 2],
    draw_order: &Vec<Entity>,
) {
    // we don't actually need mutable access to the world but having it lets us tell
    // hecs we can skip dynamic borrow checking by using query_mut
    clear_background(DARKGRAY);

    let _delta = get_frame_time();

    for id in draw_order {
        let chunk = world.get::<&TileBody>(*id).unwrap();
        let mut tx = chunk.x;
        let mut ty = chunk.y;
        for ii in 0..(chunk.data.len()) {
            if chunk.data[ii].is_visible() {
                let sx = (chunk.tiles[ii] as u32 % tsi.columns) * tsi.tile_width;
                let sy = (chunk.tiles[ii] as u32 / tsi.columns) * tsi.tile_height;
                draw_texture_ex(
                    tsi.texture,
                    tx as f32,
                    ty as f32,
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect::new(
                            sx as f32,
                            sy as f32,
                            chunk.size as f32,
                            chunk.size as f32,
                        )),
                        ..Default::default()
                    },
                );
            }
            tx += chunk.size as i32;
            if ((ii + 1) % chunk.width as usize) == 0 {
                tx = chunk.x;
                ty += chunk.size as i32;
            }
        }
    }

    for (_, (rect, draw)) in world.query::<(&IntRect, &ColorRect)>().iter() {
        draw_rectangle(
            rect.x as f32,
            rect.y as f32,
            rect.w as f32,
            rect.h as f32,
            draw.color,
        );
    }

    for (_, (rect, spr)) in world.query::<(&IntRect, &PlayerSprite)>().iter() {
        let frame = if spr.firing { 2 } else { spr.n * 5 % 2 };
        draw_texture_ex(
            tex[0],
            (rect.x - 1) as f32,
            rect.y as f32,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(16.0, 24.0)),
                source: Some(Rect::new(0.0, 24.0 * frame as f32, 16.0, 24.0)),
                flip_x: spr.flipped,
                ..Default::default()
            },
        );
    }

    for (_, (rect, spr)) in world.query::<(&IntRect, &DogSprite)>().iter() {
        draw_texture_ex(
            tex[1],
            rect.x as f32,
            rect.y as f32,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(24.0, 16.0)),
                source: Some(Rect::new(0.0, 16.0 * (spr.n / 5 % 2) as f32, 24.0, 16.0)),
                flip_x: spr.flipped,
                ..Default::default()
            },
        );
    }
}
