use camera::PlayerCamera;
use enemy::update_enemies;
use enum_iterator::first;
use hecs::CommandBuffer;
use input::{Input, VirtualKey};
use levels::Level;
use macroquad::experimental::coroutines::{start_coroutine, stop_all_coroutines};
use macroquad::prelude::*;
use physics::{Actor, PathMotion};
use pickup::Pickup;
use player::Controller;
use projectile::Projectile;
use render::Renderer;
use resources::load_assets;
use scene::Scene;
use timer::Timer;
use transition::TransitionEffectType;
use vfx::update_vfx;

mod camera;
mod draw;
mod enemy;
mod index;
mod input;
mod levels;
mod loader;
mod messages;
mod physics;
mod pickup;
mod player;
mod projectile;
mod render;
mod resources;
mod scene;
mod script;
mod stats;
mod switch;
mod timer;
mod transition;
mod vfx;
mod visibility;

const RENDER_W: u32 = 320;
const RENDER_H: u32 = 200;

fn window_conf() -> Conf {
    Conf {
        window_title: "Princess Robot".to_owned(),
        fullscreen: false,
        window_width: RENDER_W as i32 * 2,
        window_height: RENDER_H as i32 * 2,
        ..Default::default()
    }
}

#[macroquad::main(window_conf())]
async fn main() {
    set_pc_assets_folder("assets");
    /*let argv: Vec<String> = std::env::args().collect();
    let name = if argv.len() > 1 {
        argv[1].clone()
    } else {
        "intro".to_owned()
    };*/

    let mut level: Level = first::<Level>().unwrap();
    let mut scene: Scene = level.init_scene(false).await;

    let mut renderer = Renderer::new(RENDER_W, RENDER_H);
    let mut clock = Timer::new();
    let mut input = Input::new();

    let coro = start_coroutine(load_assets());
    let mut result = None;
    let mut loading_frames = 0;
    while result.is_none() {
        loading_frames += 1;
        if loading_frames > 2 {
            renderer.render_loading();
        }
        next_frame().await;
        result = coro.retrieve();
    }
    let mut assets = result.unwrap();

    loop {
        match assets.next_scene {
            None => (),
            Some((next_scene, typ)) => {
                clock = Timer::new();
                input = Input::new();
                renderer.start_transition(typ);
                scene = next_scene;
                assets.next_scene = None;
                println!("transitioning to next scene");
            }
        }

        input.update(&renderer);

        match &mut scene {
            Scene::PreLevel(coro, fast) => {
                for _ in 0..clock.get_num_updates() {
                    renderer.tick();
                }
                if (*fast || renderer.transition_finished()) && coro.is_done() {
                    assets.next_scene = Some((
                        coro.retrieve().unwrap().unwrap(),
                        TransitionEffectType::Open,
                    ))
                }
            }
            Scene::PlayLevel(ref mut resources) => {
                for _ in 0..clock.get_num_updates() {
                    let mut buffer = CommandBuffer::new();
                    PathMotion::apply(resources);
                    Controller::update(resources, &mut buffer, &input);
                    update_enemies(resources, &mut buffer);
                    Actor::update(resources);
                    Projectile::update(resources, &mut buffer);
                    Pickup::update(resources, &mut buffer);
                    update_vfx(resources, &mut buffer);
                    buffer.run_on(&mut resources.world_ref.lock().unwrap());

                    PlayerCamera::update(resources);

                    if input.is_pressed(VirtualKey::DebugKill) {
                        resources
                            .world_ref
                            .lock()
                            .unwrap()
                            .get::<&mut Controller>(resources.player_id)
                            .unwrap()
                            .hp = 0
                    }

                    for t in &resources.triggers {
                        resources.script_engine.call_entry_point(&t);
                    }
                    resources.triggers.clear();
                    resources.script_engine.schedule_queued_funcs();

                    if input.is_pressed(VirtualKey::DebugRestart) {
                        stop_all_coroutines();
                        assets.next_scene = Some((
                            // skip the transition for faster debugging
                            level.init_scene(true).await,
                            TransitionEffectType::Shatter,
                        ));
                    }
                    if input.is_pressed(VirtualKey::DebugWin) || resources.script_engine.win_flag()
                    {
                        stop_all_coroutines();
                        assets.next_scene = Some((
                            crate::scene::Scene::PostLevel(resources.stats.clone()),
                            TransitionEffectType::Shatter,
                        ));
                    }

                    input.reset();
                    resources.messages.update();
                    resources.stats.frames += 1;
                    renderer.tick();

                    /* if resources.stats.frames % 100 == 0 {
                        resources.body_index.debug();
                    } */
                }
            }
            Scene::PostLevel(_) => {
                for _ in 0..clock.get_num_updates() {
                    renderer.tick();
                }
                if input.is_any_pressed() {
                    level = level.next();
                    assets.next_scene = Some((
                        level.init_scene(false).await,
                        TransitionEffectType::Shatter,
                    ));
                }
            }
        }

        renderer.render_scene(&scene, &assets, &input, level.as_level_name());
        next_frame().await;
    }
}
