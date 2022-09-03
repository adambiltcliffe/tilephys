use crate::loader::TilesetInfo;
use crate::physics::{Actor, Controller, IntRect, TileBody};
use hecs::{Satisfies, World};
use macroquad::prelude::*;

pub(crate) fn draw(world: &mut World, tsi: &TilesetInfo) {
    // we don't actually need mutable access to the world but having it lets us tell
    // hecs we can skip dynamic borrow checking by using query_mut
    clear_background(SKYBLUE);

    let _delta = get_frame_time();

    for (_, chunk) in world.query::<&TileBody>().iter() {
        let mut tx = chunk.x;
        let mut ty = chunk.y;
        for ii in 0..(chunk.data.len()) {
            if chunk.data[ii] {
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

    for (_, (_, rect, ctrl)) in world
        .query::<(&Actor, &IntRect, Satisfies<&Controller>)>()
        .iter()
    {
        draw_rectangle(
            rect.x as f32,
            rect.y as f32,
            rect.w as f32,
            rect.h as f32,
            if ctrl { GREEN } else { GRAY },
        );
    }
}
