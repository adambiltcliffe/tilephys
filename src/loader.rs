use crate::camera::add_camera;
use crate::draw::PlayerSprite;
use crate::enemy::{add_enemy, EnemyKind};
use crate::physics::{Actor, IntRect, TileBody, TriggerZone};
use crate::pickup::add_pickup;
use crate::player::Controller;
use crate::resources::Resources;
use crate::resources::TilesetInfo;
use crate::script::ScriptEngine;
use crate::visibility::compute_obscurers;
use bitflags::bitflags;
use hecs::{Entity, World};
use macroquad::prelude::*;
use macroquad::{file::load_file, texture::load_texture};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;
use std::rc::Rc;

pub(crate) struct LoadedMap {
    pub world_ref: Rc<RefCell<World>>,
    pub tileset_info: TilesetInfo,
    pub player_start: (i32, i32),
    pub draw_order: Vec<Entity>,
}

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

pub struct AsyncPreloadReader {
    cache: HashMap<tiled::ResourcePathBuf, Rc<[u8]>>,
}

impl AsyncPreloadReader {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub(crate) async fn preload(&mut self, path: &str) {
        let data = load_file(path).await.unwrap();
        self.cache.insert(path.into(), Rc::from(data));
    }
}

impl tiled::ResourceReader for AsyncPreloadReader {
    type Resource = std::io::Cursor<Rc<[u8]>>;
    type Error = std::io::Error;
    fn read_from(&mut self, path: &Path) -> std::result::Result<Self::Resource, Self::Error> {
        self.cache
            .get(path)
            .map(|data| Cursor::new(Rc::clone(data)))
            .ok_or(std::io::Error::from(std::io::ErrorKind::NotFound))
    }
}

pub struct LoadingManager {
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
    pub(crate) async fn load(&mut self, name: &str) -> Result<(LoadedMap, ScriptEngine), String> {
        //self.loader.reader_mut().preload("testset.tsx").await; // MEGA HACK
        self.loader.reader_mut().preload(name).await;

        let map = loop {
            match self.loader.load_tmx_map(name) {
                Ok(map) => break map,
                Err(tiled::Error::ResourceLoadingError { path, err: _ }) => {
                    if path.as_os_str().to_str().unwrap() == name {
                        return Err("Resource loading error".to_owned());
                    }
                    println!("loading additional resource: {:?}", path);
                    self.loader
                        .reader_mut()
                        .preload(path.as_os_str().to_str().unwrap())
                        .await;
                }
                Err(other_err) => return Err(other_err.to_string()),
            }
        };

        let mut world: World = World::new();
        let mut body_ids: HashMap<String, Entity> = HashMap::new();
        let mut paths: HashMap<String, Vec<(f32, f32)>> = HashMap::new();
        let (mut psx, mut psy) = (0, 0);
        let mut secret_count = 0;

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
                    println!("Found an infinite tiled layer named {}", layer.name);
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
                    println!("Real chunk bounds are x:{}-{}, y:{}-{}", x0, x1, y0, y1);
                    let mut data = Vec::new();
                    let mut tiles = Vec::new();
                    for y in y0..=y1 {
                        for x in x0..=x1 {
                            let t = layer_data.get_tile(x, y);
                            let td = match t {
                                None => TileFlags::empty(),
                                Some(ltd) => {
                                    // if map parsing is ever slow, we could cache this per tile
                                    let t = ltd.get_tile().unwrap();
                                    if t.properties.contains_key("background") {
                                        TileFlags::VISIBLE
                                    } else if t.properties.contains_key("transparent") {
                                        TileFlags::BLOCKER | TileFlags::VISIBLE
                                    } else if t.properties.contains_key("platform") {
                                        TileFlags::PLATFORM | TileFlags::VISIBLE
                                    } else {
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
                    let id = world.spawn((TileBody::new(
                        x0 * map.tile_width as i32,
                        y0 * map.tile_height as i32,
                        tileset_info.tile_width as i32,
                        (x1 - x0) + 1,
                        data,
                        tiles,
                    ),));
                    body_ids.insert(layer.name.clone(), id);
                    draw_order.push(id);
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
                                println!("found a path named {}", name);
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
                                    secret_count += 1
                                }
                                println!(
                                    "found a{} trigger zone named {}",
                                    if secret { " secret" } else { "" },
                                    name
                                );
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
                                ..
                            } => {
                                if obj_type == "player" {
                                    (psx, psy) = (*x as i32, *y as i32);
                                } else if obj_type == "enemy" {
                                    add_enemy(&mut world, EnemyKind::JumpyDog, *x as i32, *y as i32)
                                } else if obj_type == "walker_enemy" {
                                    add_enemy(&mut world, EnemyKind::Dog, *x as i32, *y as i32);
                                } else if obj_type == "heart" {
                                    add_pickup(&mut world, *x as i32, *y as i32);
                                } else {
                                    println!("found an unknown point object type: {}", obj_type)
                                }
                            }
                            _ => (),
                        }
                    }
                }
                _ => println!("(Something other than an infinite tiled layer)"),
            }
        }

        println!("map has {} secret areas", secret_count);

        let world_ref = Rc::new(RefCell::new(world));
        let mut script_engine =
            ScriptEngine::new(Rc::clone(&world_ref), Rc::new(body_ids), Rc::new(paths));
        script_engine.load_file("intro.rhai").await;
        script_engine.call_entry_point("init");

        Ok((
            LoadedMap {
                world_ref,
                tileset_info,
                player_start: (psx, psy),
                draw_order,
            },
            script_engine,
        ))
    }

    pub(crate) async fn load_level(
        &mut self,
        name: &str,
    ) -> Result<(Rc<RefCell<World>>, Resources), String> {
        let (map, engine) = self.load(name).await.unwrap();

        let LoadedMap { player_start, .. } = map;
        let world_ref = Rc::clone(&map.world_ref);

        let (player_id, eye, cam) = {
            let mut world = world_ref.borrow_mut();

            let player_rect = IntRect::new(player_start.0 - 8, player_start.1 - 24, 14, 24);
            let player_eye = player_rect.centre();
            let camera_pos = add_camera(&mut world, player_rect.centre());
            let player = Actor::new(&player_rect, 0.6);
            let controller = Controller::new();
            let sprite = PlayerSprite::new();
            let player_id = world.spawn((player_rect, player, controller, sprite));

            (player_id, player_eye, camera_pos)
        };

        compute_obscurers(&mut world_ref.borrow_mut());

        let resources = Resources::new(&map, engine, player_id, eye, cam).await;
        Ok((world_ref, resources))
    }
}
