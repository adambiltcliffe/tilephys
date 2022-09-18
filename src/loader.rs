use crate::physics::{IntRect, TileBody, TriggerZone};
use bitflags::bitflags;
use hecs::{Entity, World};
use macroquad::texture::{load_texture, Texture2D};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub(crate) struct LoadedMap {
    pub world_ref: Rc<RefCell<World>>,
    pub body_ids: Rc<HashMap<String, Entity>>,
    pub paths: Rc<HashMap<String, Vec<(f32, f32)>>>,
    pub tileset_info: TilesetInfo,
}

#[derive(Clone)]
pub(crate) struct TilesetInfo {
    pub texture: Texture2D,
    pub tile_width: u32,
    pub tile_height: u32,
    pub count: u32,
    pub columns: u32,
}

bitflags! {
    pub struct TileFlags: u8 {
        const VISIBLE = 0b00000001;
        const BLOCKER = 0b00000010;
        const OBSCURER = 0b00000100;
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
}

pub(crate) async fn load_map(name: &str) -> Result<LoadedMap, String> {
    let mut world: World = World::new();
    let mut body_ids: HashMap<String, Entity> = HashMap::new();
    let mut paths: HashMap<String, Vec<(f32, f32)>> = HashMap::new();

    let mut loader = tiled::Loader::new();
    let map = loader.load_tmx_map(name).unwrap();

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
        tilecount,
        columns,
        ..
    } = **ts;
    let tileset_info = TilesetInfo {
        texture,
        tile_width,
        tile_height,
        count: tilecount,
        columns,
    };

    for layer in map.layers() {
        match layer.layer_type() {
            tiled::LayerType::TileLayer(tiled::TileLayer::Infinite(layer_data)) => {
                println!("Found an infinite tiled layer named {}", layer.name);
                let (xmin, xmax, ymin, ymax) = layer_data.chunks().fold(
                    (i32::MAX, i32::MIN, i32::MAX, i32::MIN),
                    |(x0, x1, y0, y1), ((x, y), _)| (x0.min(x), x1.max(x), y0.min(y), y1.max(y)),
                );
                const W: i32 = tiled::Chunk::WIDTH as i32;
                const H: i32 = tiled::Chunk::HEIGHT as i32;
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
                                } else {
                                    TileFlags::BLOCKER | TileFlags::VISIBLE | TileFlags::OBSCURER
                                }
                            }
                        };
                        data.push(td);
                        tiles.push(t.map(|t| t.id() as u16).unwrap_or(0));
                    }
                }
                body_ids.insert(
                    layer.name.clone(),
                    world.spawn((TileBody::new(
                        x0 * map.tile_width as i32,
                        y0 * map.tile_height as i32,
                        tileset_info.tile_width as i32,
                        (x1 - x0) + 1,
                        data,
                        tiles,
                    ),)),
                );
            }
            tiled::LayerType::ObjectLayer(data) => {
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
                            shape: tiled::ObjectShape::Rect { width, height },
                            x,
                            y,
                            ..
                        } => {
                            println!("found a trigger zone named {}", name);
                            let tz = TriggerZone { name: name.clone() };
                            let rect =
                                IntRect::new(*x as i32, *y as i32, *width as i32, *height as i32);
                            world.spawn((tz, rect));
                        }
                        _ => (),
                    }
                }
            }
            _ => println!("(Something other than an infinite tiled layer)"),
        }
    }

    Ok(LoadedMap {
        world_ref: Rc::new(RefCell::new(world)),
        body_ids: Rc::new(body_ids),
        paths: Rc::new(paths),
        tileset_info,
    })
}
