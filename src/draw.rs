use crate::enemy::EnemyHittable;
use crate::physics::{IntRect, TileBody};
use crate::resources::{GlobalAssets, SceneResources};
use crate::switch::Switch;
use crate::vfx::ZapFlash;
use hecs::World;
use macroquad::prelude::*;

pub(crate) struct ColorRect {
    color: Color,
}

impl ColorRect {
    #[allow(dead_code)] // will be used for a while each time we add a new actor
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

pub(crate) struct PlayerSprite {
    pub n: i32,
    pub firing: bool,
    pub flipped: bool,
    pub blink: bool,
    pub muzzle_flash: u8,
}

impl PlayerSprite {
    pub fn new() -> Self {
        Self {
            n: 0,
            firing: false,
            flipped: true,
            blink: false,
            muzzle_flash: 0,
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

pub(crate) struct ParrotSprite {
    pub frame: u8,
    pub flipped: bool,
    pub muzzle_flash: Option<u8>,
}

impl ParrotSprite {
    pub fn new() -> Self {
        Self {
            frame: 0,
            flipped: false,
            muzzle_flash: None,
        }
    }
}

pub(crate) struct PickupSprite {}

impl PickupSprite {
    pub fn new() -> Self {
        Self {}
    }
}

pub(crate) struct SwitchSprite {}

impl SwitchSprite {
    pub fn new() -> Self {
        Self {}
    }
}

pub(crate) struct ZapSprite {}

impl ZapSprite {
    pub fn new() -> Self {
        Self {}
    }
}

pub(crate) fn draw(world: &mut World, resources: &SceneResources, assets: &GlobalAssets) {
    // we don't actually need mutable access to the world but having it lets us tell
    // hecs we can skip dynamic borrow checking by using query_mut
    let _delta = get_frame_time();

    for id in &resources.draw_order {
        let chunk = world.get::<&TileBody>(*id).unwrap();
        let mut tx = chunk.x;
        let mut ty = chunk.y;
        for ii in 0..(chunk.data.len()) {
            if chunk.data[ii].is_visible() {
                let tsi = &resources.tileset_info;
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

    for (_, (rect, sw, _spr)) in world.query::<(&IntRect, &Switch, &SwitchSprite)>().iter() {
        let frame = i32::from(!sw.enabled);
        draw_texture_ex(
            assets.switch_sprite,
            rect.x as f32,
            rect.y as f32,
            WHITE,
            DrawTextureParams {
                //dest_size: Some(vec2(16.0, 24.0)),
                source: Some(Rect::new(16.0 * frame as f32, 0.0, 16.0, 16.0)),
                ..Default::default()
            },
        );
    }

    for (_, (rect, _spr)) in world.query::<(&IntRect, &PickupSprite)>().iter() {
        draw_texture(assets.pickup_sprite, rect.x as f32, rect.y as f32, WHITE);
    }

    for (_, (rect, _spr)) in world.query::<(&IntRect, &ZapSprite)>().iter() {
        draw_texture_ex(
            assets.zap_sprite,
            rect.x as f32,
            rect.y as f32,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(8.0, 5.0)),
                source: Some(Rect::new(0.0, 2.0, 8.0, 5.0)),
                ..Default::default()
            },
        );
    }

    for (_, (rect, spr)) in world.query::<(&IntRect, &PlayerSprite)>().iter() {
        if spr.blink {
            continue;
        }
        if spr.muzzle_flash < 6 {
            draw_texture_ex(
                assets.zap_sprite,
                rect.x as f32 + if spr.flipped { 11.0 } else { -6.0 },
                rect.y as f32 + 9.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(9.0, 9.0)),
                    source: Some(Rect::new(0.0, 9.0 * spr.muzzle_flash as f32, 9.0, 9.0)),
                    ..Default::default()
                },
            );
        }
        let frame = if spr.firing { 2 } else { spr.n * 5 % 2 };
        draw_texture_ex(
            assets.player_sprite,
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

    for (_, (rect, spr, hittable)) in world
        .query::<(&IntRect, &ParrotSprite, &EnemyHittable)>()
        .iter()
    {
        if hittable.was_hit {
            gl_use_material(assets.flash_material);
        }
        draw_texture_ex(
            assets.parrot_sprite,
            rect.x as f32,
            rect.y as f32,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(24.0, 24.0)),
                source: Some(Rect::new(0.0, 24.0 * spr.frame as f32, 24.0, 24.0)),
                flip_x: spr.flipped,
                ..Default::default()
            },
        );
        if let Some(mf) = spr.muzzle_flash {
            draw_texture_ex(
                assets.zap_sprite,
                rect.x as f32 + if spr.flipped { 16.0 } else { -1.0 },
                rect.y as f32 + 6.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(9.0, 9.0)),
                    source: Some(Rect::new(0.0, 9.0 * mf as f32, 9.0, 9.0)),
                    ..Default::default()
                },
            );
        }
        gl_use_default_material();
    }

    for (_, (rect, spr, hittable)) in world
        .query::<(&IntRect, &DogSprite, &EnemyHittable)>()
        .iter()
    {
        if hittable.was_hit {
            gl_use_material(assets.flash_material);
        }
        draw_texture_ex(
            assets.dog_sprite,
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
        gl_use_default_material();
    }

    for (_, zap) in world.query::<&ZapFlash>().iter() {
        draw_texture_ex(
            assets.zap_sprite,
            zap.x as f32,
            zap.y as f32,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(9.0, 9.0)),
                source: Some(Rect::new(0.0, 9.0 * (zap.n) as f32, 9.0, 9.0)),
                ..Default::default()
            },
        );
    }
}
