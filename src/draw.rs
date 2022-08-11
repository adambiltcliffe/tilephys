use crate::physics::{Actor, Controller, IntRect, TileBody};
use hecs::{Satisfies, World};
use macroquad::prelude::*;

pub fn draw(world: &mut World) {
    // we don't actually need mutable access to the world but having it lets us tell
    // hecs we can skip dynamic borrow checking by using query_mut
    clear_background(SKYBLUE);

    let _delta = get_frame_time();
    let (mx, my) = mouse_position();
    let mouse_rect = IntRect::new(mx as i32 - 5, my as i32 - 5, 10, 10);

    for (_, chunk) in world.query::<&TileBody>().iter() {
        let mut tx = chunk.x;
        let mut ty = chunk.y;
        for ii in 0..(chunk.data.len()) {
            if chunk.data[ii] {
                let c = if chunk.collide(&mouse_rect) {
                    RED
                } else {
                    BLUE
                };
                draw_rectangle(
                    tx as f32,
                    ty as f32,
                    chunk.size as f32,
                    chunk.size as f32,
                    c,
                );
            }
            tx += chunk.size as i32;
            if ((ii + 1) % chunk.width as usize) == 0 {
                tx = chunk.x;
                ty += chunk.size as i32;
            }
        }
    }

    draw_rectangle(mx - 5., my - 5., 10., 10., ORANGE);

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
