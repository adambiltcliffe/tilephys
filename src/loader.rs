use crate::TileBody;
use hecs::{Entity, World};
use std::collections::HashMap;

pub(crate) struct LoadedMap {
    pub world: World,
    pub body_ids: HashMap<String, Entity>,
    pub paths: HashMap<String, Vec<(f32, f32)>>,
}

pub(crate) fn load_map() -> LoadedMap {
    let mut world: World = World::new();
    let mut body_ids: HashMap<String, Entity> = HashMap::new();
    let mut paths: HashMap<String, Vec<(f32, f32)>> = HashMap::new();

    let mut loader = tiled::Loader::new();
    let map = loader.load_tmx_map("testmap.tmx").unwrap();
    for layer in map.layers() {
        match layer.layer_type() {
            tiled::LayerType::TileLayer(tiled::TileLayer::Infinite(data)) => {
                println!("Found an infinite tiled layer named {}", layer.name);
                let (xmin, xmax, ymin, ymax) = data.chunks().fold(
                    (i32::MAX, i32::MIN, i32::MAX, i32::MIN),
                    |(x0, x1, y0, y1), ((x, y), _)| (x0.min(x), x1.max(x), y0.min(y), y1.max(y)),
                );
                const W: i32 = tiled::Chunk::WIDTH as i32;
                const H: i32 = tiled::Chunk::HEIGHT as i32;
                let (mut x0, mut x1, mut y0, mut y1) = (i32::MAX, i32::MIN, i32::MAX, i32::MIN);
                for y in ymin * H..(ymax + 1) * H {
                    for x in xmin * W..(xmax + 1) * W {
                        if data.get_tile(x, y).is_some() {
                            x0 = x0.min(x);
                            x1 = x1.max(x);
                            y0 = y0.min(y);
                            y1 = y1.max(y);
                        }
                    }
                }
                println!("Real chunk bounds are x:{}-{}, y:{}-{}", x0, x1, y0, y1);
                let mut tiledata = Vec::new();
                for y in y0..=y1 {
                    for x in x0..=x1 {
                        tiledata.push(data.get_tile(x, y).is_some());
                    }
                }
                body_ids.insert(
                    layer.name.clone(),
                    world.spawn((TileBody::new(
                        x0 * map.tile_width as i32,
                        y0 * map.tile_height as i32,
                        map.tile_width as i32,
                        (x1 - x0) + 1,
                        tiledata,
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
                        _ => (),
                    }
                }
            }
            _ => println!("(Something other than an infinite tiled layer)"),
        }
    }

    LoadedMap {
        world,
        body_ids,
        paths,
    }
}
