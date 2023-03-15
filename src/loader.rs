use crate::camera::add_camera;
use crate::draw::PlayerSprite;
use crate::enemy::{add_enemy, EnemyKind, ParrotKind};
use crate::index::SpatialIndex;
use crate::level::LevelInfo;
use crate::log::{info as log_info, warn};
use crate::messages::Messages;
use crate::physics::{Actor, IntRect, PhysicsCoeffs, TileBody, TriggerZone};
use crate::pickup::{add_ammo, add_heart, add_weapon};
use crate::player::Controller;
use crate::resources::TilesetInfo;
use crate::resources::{Inventory, SceneResources};
use crate::scene::Scene;
use crate::script::ScriptEngine;
use crate::stats::LevelStats;
use crate::switch::add_switch;
use crate::visibility::compute_obscurers;
use crate::weapon::{new_weapon, AmmoType, WeaponSelectorUI, WeaponType};
use bitflags::bitflags;
use hecs::{Entity, World};
use macroquad::prelude::*;
use macroquad::{file::load_file, texture::load_texture};
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::Cursor;
use std::path::Path;
use std::sync::{Arc, Mutex};

bitflags! {
    pub struct TileFlags: u8 {
        const VISIBLE = 0b00000001;
        const BLOCKER = 0b00000010;
        const OBSCURER = 0b00000100;
        const PLATFORM = 0b00001000;
    }
}

impl TileFlags {
    #[inline]
    pub fn is_visible(&self) -> bool {
        self.contains(Self::VISIBLE)
    }

    #[inline]
    pub fn is_blocker(&self) -> bool {
        self.contains(Self::BLOCKER)
    }

    #[inline]
    pub fn is_obscurer(&self) -> bool {
        self.contains(Self::OBSCURER)
    }

    #[inline]
    pub fn is_platform(&self) -> bool {
        self.contains(Self::PLATFORM)
    }
}

struct AsyncPreloadReader {
    cache: HashMap<tiled::ResourcePathBuf, Arc<[u8]>>,
}

impl AsyncPreloadReader {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub(crate) async fn preload(&mut self, path: &str) {
        let data = load_file(path).await.unwrap();
        self.cache.insert(path.into(), Arc::from(data));
    }
}

impl tiled::ResourceReader for AsyncPreloadReader {
    type Resource = std::io::Cursor<Arc<[u8]>>;
    type Error = std::io::Error;
    fn read_from(&mut self, path: &Path) -> std::result::Result<Self::Resource, Self::Error> {
        self.cache
            .get(path)
            .map(|data| Cursor::new(Arc::clone(data)))
            .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))
    }
}

struct LoadingManager {
    loader: tiled::Loader<tiled::DefaultResourceCache, AsyncPreloadReader>,
}

impl LoadingManager {
    pub fn new() -> Self {
        let preloader = AsyncPreloadReader::new();
        let loader =
            tiled::Loader::with_cache_and_reader(tiled::DefaultResourceCache::new(), preloader);
        Self { loader }
    }

    // eventually this should probably not use String as its error type
    pub(crate) async fn load_level(
        &mut self,
        info: &LevelInfo,
        inv: Inventory,
    ) -> Result<Scene, String> {
        let map_name = format!("{}.tmx", info.path).to_owned();
        self.loader.reader_mut().preload(&map_name).await;

        let map = loop {
            match self.loader.load_tmx_map(&map_name) {
                Ok(map) => break map,
                Err(tiled::Error::ResourceLoadingError { path, err: _ }) => {
                    if path.as_os_str().to_str().unwrap() == map_name {
                        return Err("Resource loading error".to_owned());
                    }
                    self.loader
                        .reader_mut()
                        .preload(path.as_os_str().to_str().unwrap())
                        .await;
                }
                Err(other_err) => return Err(other_err.to_string()),
            }
        };

        let mut world: World = World::new();
        let mut ids: HashMap<String, Entity> = HashMap::new();
        let mut paths: HashMap<String, Vec<(f32, f32)>> = HashMap::new();
        let mut body_index = SpatialIndex::new();
        let mut body_index_count = 0;
        let mut selector = WeaponSelectorUI::new();
        let (mut psx, mut psy) = (0, 0);
        let mut max_kills = 0;
        let mut max_items = 0;
        let mut max_secrets = 0;

        if map.tilesets().len() != 1 {
            return Err("map should contain only one tileset".to_owned());
        }
        let ts = &map.tilesets()[0];
        let texture = load_texture(
            ts.image
                .as_ref()
                .ok_or("tileset needs to contain a source filename")?
                .source
                .as_path()
                .to_str()
                .unwrap(),
        )
        .await
        .unwrap();
        let tiled::Tileset {
            tile_width,
            tile_height,
            columns,
            ..
        } = **ts;
        let tileset_info = TilesetInfo {
            texture,
            tile_width,
            tile_height,
            columns,
        };

        let mut draw_order = Vec::new();

        for layer in map.layers() {
            match layer.layer_type() {
                tiled::LayerType::Tiles(tiled::TileLayer::Infinite(layer_data)) => {
                    let (xmin, xmax, ymin, ymax) = layer_data.chunks().fold(
                        (i32::MAX, i32::MIN, i32::MAX, i32::MIN),
                        |(x0, x1, y0, y1), ((x, y), _)| {
                            (x0.min(x), x1.max(x), y0.min(y), y1.max(y))
                        },
                    );
                    const W: i32 = tiled::ChunkData::WIDTH as i32;
                    const H: i32 = tiled::ChunkData::HEIGHT as i32;
                    let (mut x0, mut x1, mut y0, mut y1) = (i32::MAX, i32::MIN, i32::MAX, i32::MIN);
                    for y in ymin * H..(ymax + 1) * H {
                        for x in xmin * W..(xmax + 1) * W {
                            if layer_data.get_tile(x, y).is_some() {
                                x0 = x0.min(x);
                                x1 = x1.max(x);
                                y0 = y0.min(y);
                                y1 = y1.max(y);
                            }
                        }
                    }
                    let mut data = Vec::new();
                    let mut tiles = Vec::new();
                    let mut solid = 0;
                    let mut not_solid = 0;
                    for y in y0..=y1 {
                        for x in x0..=x1 {
                            let t = layer_data.get_tile(x, y);
                            let td = match t {
                                None => TileFlags::empty(),
                                Some(ltd) => {
                                    // if map parsing is ever slow, we could cache this per tile
                                    let t = ltd.get_tile().unwrap();
                                    if t.properties.contains_key("background") {
                                        not_solid += 1;
                                        TileFlags::VISIBLE
                                    } else if t.properties.contains_key("transparent") {
                                        solid += 1;
                                        TileFlags::BLOCKER | TileFlags::VISIBLE
                                    } else if t.properties.contains_key("platform") {
                                        solid += 1;
                                        TileFlags::PLATFORM | TileFlags::VISIBLE
                                    } else {
                                        solid += 1;
                                        TileFlags::BLOCKER
                                            | TileFlags::VISIBLE
                                            | TileFlags::OBSCURER
                                    }
                                }
                            };
                            data.push(td);
                            tiles.push(t.map(|t| t.id() as u16).unwrap_or(0));
                        }
                    }
                    let door = layer.properties.contains_key("door");
                    let indexed = solid > 0;
                    if solid > 0 && solid < not_solid {
                        warn(&format!(
                            "Warning: layer {} has {} solid tiles and {} background tiles so must be indexed",
                            layer.name, solid, not_solid
                        ));
                    }
                    let body = TileBody::new(
                        x0 * map.tile_width as i32,
                        y0 * map.tile_height as i32,
                        tileset_info.tile_width as i32,
                        (x1 - x0) + 1,
                        data,
                        tiles,
                        door,
                        indexed,
                    );
                    let rect = body.get_rect();
                    let id = world.spawn((body,));
                    ids.insert(layer.name.clone(), id);
                    draw_order.push(id);
                    if indexed {
                        body_index.insert_at(id, &rect);
                        body_index_count += 1;
                    }
                }
                tiled::LayerType::Objects(data) => {
                    for obj in data.objects() {
                        match &*obj {
                            tiled::ObjectData {
                                name,
                                shape: tiled::ObjectShape::Polyline { points },
                                ..
                            }
                            | tiled::ObjectData {
                                name,
                                shape: tiled::ObjectShape::Polygon { points },
                                ..
                            } => {
                                if points[0] != (0.0, 0.0) {
                                    warn(&format!(
                                        "path {} has an initial position which is not (0,0)",
                                        name
                                    ));
                                }
                                paths.insert(name.clone(), points.clone());
                            }
                            tiled::ObjectData {
                                name,
                                obj_type,
                                shape: tiled::ObjectShape::Rect { width, height },
                                x,
                                y,
                                ..
                            } => {
                                let secret = obj_type == "secret";
                                if secret {
                                    max_secrets += 1
                                }
                                let tz = TriggerZone::new(name.clone(), secret);
                                let rect = IntRect::new(
                                    *x as i32,
                                    *y as i32,
                                    *width as i32,
                                    *height as i32,
                                );
                                world.spawn((tz, rect));
                            }
                            tiled::ObjectData {
                                shape: tiled::ObjectShape::Point(x, y),
                                obj_type,
                                name,
                                properties,
                                ..
                            } => {
                                if obj_type == "player" {
                                    (psx, psy) = (*x as i32, *y as i32);
                                    if properties.contains_key("hide-weapon-ui") && inv.is_default {
                                        // in case we re-entered the intro level with weapons by looping
                                        selector.hidden = true;
                                    }
                                } else if obj_type == "enemy" {
                                    add_enemy(
                                        &mut world,
                                        EnemyKind::JumpyDog,
                                        *x as i32,
                                        *y as i32,
                                    );
                                    max_kills += 1;
                                } else if obj_type == "walker_enemy" {
                                    add_enemy(&mut world, EnemyKind::Dog, *x as i32, *y as i32);
                                    max_kills += 1;
                                } else if obj_type == "parrot_enemy" {
                                    add_enemy(
                                        &mut world,
                                        EnemyKind::SpiderParrot(ParrotKind::Laser),
                                        *x as i32,
                                        *y as i32,
                                    );
                                    max_kills += 1;
                                } else if obj_type == "cannon_parrot_enemy" {
                                    add_enemy(
                                        &mut world,
                                        EnemyKind::SpiderParrot(ParrotKind::Cannon),
                                        *x as i32,
                                        *y as i32,
                                    );
                                    max_kills += 1;
                                } else if obj_type == "railgun_drone_enemy" {
                                    add_enemy(&mut world, EnemyKind::Drone, *x as i32, *y as i32);
                                    max_kills += 1;
                                } else if obj_type == "heart" {
                                    add_heart(&mut world, *x as i32, *y as i32);
                                    max_items += 1;
                                } else if obj_type == "ammo_cells_6" {
                                    add_ammo(&mut world, *x as i32, *y as i32, AmmoType::Cell, 6);
                                    max_items += 1;
                                } else if obj_type == "ammo_shells_4" {
                                    add_ammo(&mut world, *x as i32, *y as i32, AmmoType::Shell, 4);
                                    max_items += 1;
                                } else if obj_type == "ammo_slugs_5" {
                                    add_ammo(&mut world, *x as i32, *y as i32, AmmoType::Slug, 5);
                                    max_items += 1;
                                } else if obj_type == "ammo_rocket" {
                                    add_ammo(&mut world, *x as i32, *y as i32, AmmoType::Rocket, 1);
                                    max_items += 1;
                                } else if obj_type == "weapon_shotgun" {
                                    add_weapon(
                                        &mut world,
                                        *x as i32,
                                        *y as i32,
                                        WeaponType::Shotgun,
                                        properties.contains_key("gravity"),
                                    );
                                    max_items += 1;
                                } else if obj_type == "weapon_super_shotgun" {
                                    add_weapon(
                                        &mut world,
                                        *x as i32,
                                        *y as i32,
                                        WeaponType::SuperShotgun,
                                        properties.contains_key("gravity"),
                                    );
                                    max_items += 1;
                                } else if obj_type == "weapon_reverse_shotgun" {
                                    add_weapon(
                                        &mut world,
                                        *x as i32,
                                        *y as i32,
                                        WeaponType::ReverseShotgun,
                                        properties.contains_key("gravity"),
                                    );
                                    max_items += 1;
                                } else if obj_type == "weapon_auto_laser" {
                                    add_weapon(
                                        &mut world,
                                        *x as i32,
                                        *y as i32,
                                        WeaponType::AutoLaser,
                                        properties.contains_key("gravity"),
                                    );
                                    max_items += 1;
                                } else if obj_type == "weapon_burst_laser" {
                                    add_weapon(
                                        &mut world,
                                        *x as i32,
                                        *y as i32,
                                        WeaponType::BurstLaser,
                                        properties.contains_key("gravity"),
                                    );
                                    max_items += 1;
                                } else if obj_type == "weapon_double_laser" {
                                    add_weapon(
                                        &mut world,
                                        *x as i32,
                                        *y as i32,
                                        WeaponType::DoubleLaser,
                                        properties.contains_key("gravity"),
                                    );
                                    max_items += 1;
                                } else if obj_type == "weapon_railgun" {
                                    add_weapon(
                                        &mut world,
                                        *x as i32,
                                        *y as i32,
                                        WeaponType::Railgun,
                                        properties.contains_key("gravity"),
                                    );
                                    max_items += 1;
                                } else if obj_type == "switch" {
                                    let id =
                                        add_switch(&mut world, name.clone(), *x as i32, *y as i32);
                                    ids.insert(name.clone(), id);
                                } else {
                                    warn(&format!(
                                        "found an unknown point object type: {}",
                                        obj_type
                                    ));
                                }
                            }
                            _ => (),
                        }
                    }
                }
                _ => warn("found a layer type other than an infinite tiled layer"),
            }
        }

        log_info(&format!(
            "map '{}' loaded with {} indexed layers and {} total layers",
            info.path,
            body_index_count,
            draw_order.len()
        ));

        let world_ref = Arc::new(Mutex::new(world));
        let mut script_engine =
            ScriptEngine::new(Arc::clone(&world_ref), Arc::new(ids), Arc::new(paths));
        script_engine
            .load_file(&format!("{}.rhai", info.path))
            .await;
        script_engine.call_entry_point("init");

        let player_start = (psx, psy);

        let (player_id, eye_pos, camera_pos) = {
            let mut world = world_ref.lock().unwrap();

            let player_rect = IntRect::new(player_start.0 - 8, player_start.1 - 24, 14, 24);
            let player_eye = player_rect.centre();
            let camera_pos = add_camera(&mut world, player_rect.centre());
            let player = Actor::new(&player_rect, PhysicsCoeffs::Player);
            let controller = Controller::new();
            let sprite = PlayerSprite::new();
            let player_id = world.spawn((player_rect, player, controller, sprite));

            (player_id, player_eye, camera_pos)
        };

        compute_obscurers(&mut world_ref.lock().unwrap());

        let stats = LevelStats::new(info.clone(), max_kills, max_items, max_secrets);
        let mut weapons = VecDeque::with_capacity(4);
        for t in inv.weapon_types {
            weapons.push_back(new_weapon(t));
        }

        let resources = SceneResources {
            world_ref,
            script_engine,
            player_id,
            eye_pos,
            camera_pos,
            draw_order,
            body_index,
            tileset_info,
            messages: Messages::new(),
            stats,
            triggers: HashSet::new(),
            weapons,
            ammo: inv.ammo,
            selector,
            death_timer: None,
        };
        Ok(Scene::PlayLevel(resources))
    }
}

pub async fn load_level(info: LevelInfo, inv: Inventory) -> Result<Scene, String> {
    LoadingManager::new().load_level(&info, inv).await
}
