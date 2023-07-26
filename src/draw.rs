use crate::config::config;
use crate::enemy::{EnemyHittable, ParrotBossBehaviour, ParrotKind, Reticule};
use crate::physics::{IntRect, TileBody};
use crate::pickup::{Pickup, PickupType, WeaponPickup};
use crate::resources::{GlobalAssets, SceneResources};
use crate::switch::Switch;
use crate::vfx::ZapFlash;
use crate::weapon::{weapon_sprite_frame, AmmoType};
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
            muzzle_flash: 100,
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
    pub kind: ParrotKind,
    pub frame: u8,
    pub flipped: bool,
    pub muzzle_flash: Option<u8>,
}

impl ParrotSprite {
    pub fn new(kind: ParrotKind) -> Self {
        Self {
            kind,
            frame: 0,
            flipped: false,
            muzzle_flash: None,
        }
    }
}

pub(crate) struct DroneSprite {
    pub frame: u8,
    pub flipped_v: bool,
    pub flipped_h: bool,
}

impl DroneSprite {
    pub fn new() -> Self {
        Self {
            frame: 0,
            flipped_h: false,
            flipped_v: false,
        }
    }
}

pub(crate) struct ParrotHeadSprite {}

impl ParrotHeadSprite {
    pub fn new() -> Self {
        Self {}
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

pub(crate) fn draw_tiles(world: &mut World, resources: &SceneResources) {
    // we don't actually need mutable access to the world but having it lets us tell
    // hecs we can skip dynamic borrow checking by using query_mut
    let cam = resources.camera_pos;
    for id in &resources.draw_order {
        let chunk = world.get::<&TileBody>(*id).unwrap();
        let cx_min = ((cam.x as i32 - chunk.x - crate::RENDER_W as i32 / 2) / chunk.size).max(0);
        let cx_max = ((cam.x as i32 - chunk.x + crate::RENDER_W as i32 / 2) / chunk.size)
            .min(chunk.width - 1);
        let cy_min = ((cam.y as i32 - chunk.y - crate::RENDER_H as i32 / 2) / chunk.size).max(0);
        let cy_max = ((cam.y as i32 - chunk.y + crate::RENDER_H as i32 / 2) / chunk.size)
            .min((chunk.data.len() as i32 / chunk.width) - 1);
        let mut ty = chunk.y + (cy_min * chunk.size);
        for cy in cy_min..=cy_max {
            let mut tx = chunk.x + (cx_min * chunk.size);
            for cx in cx_min..=cx_max {
                let ii = ((cy * chunk.width) + cx) as usize;
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
            }
            ty += chunk.size as i32;
        }
    }
}

pub(crate) fn draw_sprites(world: &mut World, resources: &SceneResources, assets: &GlobalAssets) {
    let cam = resources.camera_pos;
    let camera_rect = IntRect::new(
        cam.x as i32 - crate::RENDER_W as i32 / 2 - 64,
        cam.y as i32 - crate::RENDER_H as i32 / 2 - 64,
        crate::RENDER_W as i32 + 128,
        crate::RENDER_H as i32 + 128,
    );

    for (_, (rect, draw)) in world.query::<(&IntRect, &ColorRect)>().iter() {
        if rect.intersects(&camera_rect) {
            draw_rectangle(
                rect.x as f32,
                rect.y as f32,
                rect.w as f32,
                rect.h as f32,
                draw.color,
            );
        }
    }

    for (_, (rect, _spr)) in world.query::<(&IntRect, &ParrotHeadSprite)>().iter() {
        if rect.intersects(&camera_rect) {
            draw_texture_ex(
                assets.boss_sprites,
                rect.x as f32,
                rect.y as f32,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(0.0, 32.0, 16.0, 16.0)),
                    ..Default::default()
                },
            );
        }
    }

    for (_, (rect, beh)) in world.query::<(&IntRect, &ParrotBossBehaviour)>().iter() {
        let leg_squared = config().boss_leg_length().powf(2.0);
        let c = rect.centre();
        for i in 0..4 {
            let foot_rect = world.get::<&IntRect>(beh.feet[i]).unwrap();
            let f = foot_rect.centre() - vec2(0.0, 4.0);
            let ct = Vec2::new(c.x + i as f32 * 8.0 - 12.0, c.y);
            let hv = (f - ct) / 2.0;
            let mut perp = hv.perp().normalize();
            if i > 1 {
                perp *= -1.0
            }
            let pl = (leg_squared - hv.length_squared()).sqrt();
            let mut knee = hv;
            if !pl.is_nan() {
                knee += perp * pl
            }
            draw_line(ct.x, ct.y, (ct + knee).x, (ct + knee).y, 4.0, GRAY);
            draw_line(f.x, f.y, (ct + knee).x, (ct + knee).y, 4.0, GRAY);
            draw_texture_ex(
                assets.boss_sprites,
                (ct + knee).x.round() - 4.0,
                (ct + knee).y.round() - 4.0,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(16.0, 48.0, 8.0, 8.0)),
                    ..Default::default()
                },
            );
            draw_texture_ex(
                assets.boss_sprites,
                foot_rect.x as f32,
                foot_rect.y as f32,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(16.0, 32.0, 8.0, 16.0)),
                    ..Default::default()
                },
            );
        }
        for i in 0..6 {
            let head_rect = world.get::<&IntRect>(beh.heads[i]).unwrap();
            let h = head_rect.centre();
            draw_line(c.x, c.y - 17.0, h.x, h.y + 3.0, 4.0, GRAY);
        }
        draw_texture_ex(
            assets.boss_sprites,
            rect.x as f32,
            rect.y as f32 - 12.0,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(0.0, 0.0, 64.0, 32.0)),
                ..Default::default()
            },
        );
    }

    for (_, (rect, sw, _spr)) in world.query::<(&IntRect, &Switch, &SwitchSprite)>().iter() {
        if rect.intersects(&camera_rect) {
            let frame = i32::from(!sw.enabled);
            draw_texture_ex(
                assets.switch_sprite,
                rect.x as f32,
                rect.y as f32,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(16.0 * frame as f32, 0.0, 16.0, 16.0)),
                    ..Default::default()
                },
            );
        }
    }

    for (_, (rect, p, _spr)) in world.query::<(&IntRect, &Pickup, &PickupSprite)>().iter() {
        if rect.intersects(&camera_rect) {
            let y = match p.typ {
                PickupType::Heart => 0.0,
                PickupType::Ammo(AmmoType::Cell, _) => 16.0,
                PickupType::Ammo(AmmoType::Shell, _) => 32.0,
                PickupType::Ammo(AmmoType::Rocket, _) => 48.0,
                PickupType::Ammo(AmmoType::Slug, _) => 64.0,
                PickupType::Ammo(AmmoType::Fuel, _) => 80.0, // placeholder
            };
            draw_texture_ex(
                assets.pickup_sprite,
                rect.x as f32,
                rect.y as f32,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(0.0, y, 16.0, 16.0)),
                    ..Default::default()
                },
            );
        }
    }

    for (_, (rect, w)) in world.query::<(&IntRect, &WeaponPickup)>().iter() {
        if rect.intersects(&camera_rect) {
            let frame = weapon_sprite_frame(w.typ);
            draw_texture_ex(
                assets.weapon_sprite,
                rect.x as f32,
                rect.y as f32,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(0.0, 16.0 * frame as f32, 24.0, 16.0)),
                    ..Default::default()
                },
            );
        }
    }

    for (_, (rect, _spr)) in world.query::<(&IntRect, &ZapSprite)>().iter() {
        if rect.intersects(&camera_rect) {
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
        if rect.intersects(&camera_rect) {
            if hittable.was_hit {
                gl_use_material(assets.flash_material);
            }
            let tex = match spr.kind {
                ParrotKind::Laser => assets.parrot_sprite,
                ParrotKind::Cannon => assets.parrot_sprite2,
            };
            draw_texture_ex(
                tex,
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
    }

    for (_, (rect, spr, hittable)) in world
        .query::<(&IntRect, &DogSprite, &EnemyHittable)>()
        .iter()
    {
        if rect.intersects(&camera_rect) {
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
    }

    for (_, (rect, spr, hittable)) in world
        .query::<(&IntRect, &DroneSprite, &EnemyHittable)>()
        .iter()
    {
        if rect.intersects(&camera_rect) {
            if hittable.was_hit {
                gl_use_material(assets.flash_material);
            }
            draw_texture_ex(
                assets.drone_sprite,
                rect.x as f32,
                rect.y as f32,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(16.0, 16.0)),
                    source: Some(Rect::new(0.0, 16.0 * spr.frame as f32, 16.0, 16.0)),
                    ..Default::default()
                },
            );
            gl_use_default_material();
            draw_texture_ex(
                assets.drone_sprite,
                (rect.x + 4) as f32,
                (rect.y + 4) as f32,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(8.0, 8.0)),
                    source: Some(Rect::new(0.0, 64.0, 8.0, 8.0)),
                    flip_x: spr.flipped_h,
                    flip_y: spr.flipped_v,
                    ..Default::default()
                },
            );
        }
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

    let lock_frames = { config().drone_lock_frames() as f32 };
    for (_, ret) in world.query::<&Reticule>().iter() {
        let r = match &ret.lock_timer {
            None => 8.0,
            Some(t) => 8.0 * (1.0 - (t.get() as f32 / lock_frames)),
        };
        draw_circle_lines(ret.pos.x, ret.pos.y, r, 1.0, RED);
    }
}
