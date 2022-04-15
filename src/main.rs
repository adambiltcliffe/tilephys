use macroquad::prelude::*;

const SCR_W: i32 = 400;
const SCR_H: i32 = 400;

struct TileChunk {
    width: i32,
    size: i32,
    data: Vec<bool>,
    x: i32,
    y: i32,
}

impl TileChunk {
    fn new(x: i32, y: i32, size: i32, width: i32, data: Vec<bool>) -> Self {
        Self {
            x,
            y,
            size,
            width,
            data,
        }
    }

    // signature of this function should change when we have a proper rectangle struct
    fn collide(&self, x: i32, y: i32, w: i32, h: i32) -> bool {
        let min_kx = (x - self.x).div_euclid(self.size);
        let max_kx = (x + w - 1 - self.x).div_euclid(self.size);
        let min_ky = (y - self.y).div_euclid(self.size);
        let max_ky = (y + h - 1 - self.y).div_euclid(self.size);
        for ky in min_ky..=max_ky {
            if ky >= 0 {
                for kx in min_kx..=max_kx {
                    if kx >= 0 && kx < self.width {
                        let index = ky * self.width + kx;
                        if index >= 0 && index < self.data.len() as i32 && self.data[index as usize]
                        {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

struct Actor {
    x: i32,
    y: i32,
    prec_x: f32,
    prec_y: f32,
    old_prec_x: f32,
    old_prec_y: f32,
    width: i32,
    height: i32,
    drag_x: f32,
    drag_y: f32,
}

impl Actor {
    fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            prec_x: x as f32,
            prec_y: y as f32,
            old_prec_x: x as f32,
            old_prec_y: y as f32,
            drag_x: 0.9,
            drag_y: 0.9,
        }
    }
}

fn move_actor(actor: &mut Actor, chunks: &Vec<TileChunk>) {
    let prec_dx = actor.prec_x - actor.old_prec_x;
    actor.old_prec_x = actor.prec_x;
    actor.prec_x += prec_dx * actor.drag_x;
    let targ_x = actor.prec_x.round() as i32;
    while actor.x != targ_x {
        let dx = (targ_x - actor.x).signum();
        if chunks
            .iter()
            .any(|c| c.collide(actor.x + dx, actor.y, actor.width, actor.height))
        {
            actor.prec_x = actor.x as f32;
            break;
        } else {
            actor.x += dx;
        }
    }
    let prec_dy = actor.prec_y - actor.old_prec_y;
    actor.old_prec_y = actor.prec_y;
    actor.prec_y += prec_dy * actor.drag_y;
    let targ_y = actor.prec_y.round() as i32;
    while actor.y != targ_y {
        let dy = (targ_y - actor.y).signum();
        if chunks
            .iter()
            .any(|c| c.collide(actor.x, actor.y + dy, actor.width, actor.height))
        {
            actor.prec_y = actor.y as f32;
            break;
        } else {
            actor.y += dy
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Platform tile physics test".to_owned(),
        fullscreen: false,
        window_width: SCR_W,
        window_height: SCR_H,
        ..Default::default()
    }
}

#[macroquad::main(window_conf())]
async fn main() {
    /*set_camera(&Camera2D {
        zoom: (vec2(1.0, 1.0)),
        target: vec2(SCR_W / 2., SCR_H / 2.),
        ..Default::default()
    });*/

    let mut chunks: Vec<TileChunk> = Vec::new();

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
                chunks.push(TileChunk::new(
                    x0 * map.tile_width as i32,
                    y0 * map.tile_height as i32,
                    map.tile_width as i32,
                    (x1 - x0) + 1,
                    tiledata,
                ))
            }
            _ => println!("(Something other than an infinite tiled layer)"),
        }
    }

    let mut player = Actor::new(50, 10, 10, 10);

    loop {
        clear_background(SKYBLUE);

        let _delta = get_frame_time();
        let (mx, my) = mouse_position();

        player.prec_y += 1.0;
        if is_key_down(KeyCode::Left) {
            player.prec_x -= 1.0;
        }
        if is_key_down(KeyCode::Right) {
            player.prec_x += 1.0;
        }
        move_actor(&mut player, &chunks);

        for chunk in &chunks {
            let mut tx = chunk.x;
            let mut ty = chunk.y;
            for ii in 0..(chunk.data.len()) {
                if chunk.data[ii] {
                    let c = if chunk.collide(mx as i32 - 5, my as i32 - 5, 10, 10) {
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

        draw_rectangle(
            player.x as f32,
            player.y as f32,
            player.width as f32,
            player.height as f32,
            GREEN,
        );

        chunks[1].x -= 1;
        chunks[2].x += 1;
        chunks[3].y -= 1;
        chunks[4].y += 1;

        next_frame().await
    }
}
