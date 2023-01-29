use std::num::NonZeroU8;

use camera::PlayerCamera;
use enemy::update_enemies;
use hecs::CommandBuffer;
use input::Input;
use macroquad::experimental::coroutines::{start_coroutine, stop_all_coroutines};
use macroquad::prelude::*;
use physics::{Actor, PathMotion};
use pickup::{Pickup, WeaponPickup};
use player::Controller;
use projectile::Projectile;
use render::Renderer;
use resources::load_assets;
use scene::{new_prelevel, Scene};
use timer::Timer;
use transition::TransitionEffectType;
use vfx::update_vfx;

#[cfg(debug_assertions)]
use input::VirtualKey;

mod camera;
mod draw;
mod enemy;
mod index;
mod input;
mod level;
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
mod weapon;

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
    let argv: Vec<String> = std::env::args().collect();

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

    let info = if argv.len() > 1 {
        assets.get_level_with_path(&argv[1])
    } else {
        assets.get_first_level()
    };

    let mut scene: Scene = new_prelevel(info, false).await;

    loop {
        match assets.next_scene {
            None => (),
            Some((next_scene, typ)) => {
                clock = Timer::new();
                input = Input::new();
                renderer.start_transition(typ);
                scene = next_scene;
                assets.next_scene = None;
            }
        }

        input.update();

        match &mut scene {
            Scene::PreLevel(_n, coro, fast) => {
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
                    Pickup::update(resources, &mut buffer);
                    WeaponPickup::update(resources);
                    Controller::update(resources, &mut buffer, &input);
                    update_enemies(resources, &mut buffer);
                    Actor::update(resources);
                    Projectile::update(resources, &mut buffer);
                    update_vfx(resources, &mut buffer);
                    buffer.run_on(&mut resources.world_ref.lock().unwrap());

                    PlayerCamera::update(resources);

                    let mut player_dead = true;
                    {
                        let w = resources.world_ref.lock().unwrap();
                        #[allow(unused_mut)] // needs to be mut in debug mode but not release
                        if let Ok(mut controller) = w.get::<&mut Controller>(resources.player_id) {
                            #[cfg(debug_assertions)]
                            if input.is_pressed(VirtualKey::DebugKill) {
                                controller.hp = 0
                            }
                            player_dead = controller.hp == 0;
                        };
                    };
                    if player_dead {
                        let dt = &mut resources
                            .death_timer
                            .get_or_insert(NonZeroU8::new(1).unwrap());
                        **dt = dt.saturating_add(1);
                        let n = dt.get();
                        if n == 60 {
                            resources
                                .messages
                                .add("Press any key to restart.".to_owned());
                        }
                        if n > 30 && input.is_any_pressed() {
                            stop_all_coroutines();
                            assets.next_scene = Some((
                                new_prelevel(resources.stats.info.clone(), false).await,
                                TransitionEffectType::Shatter,
                            ));
                        }
                    }

                    for t in &resources.triggers {
                        resources.script_engine.call_entry_point(t);
                    }
                    resources.triggers.clear();
                    resources.script_engine.schedule_queued_funcs();
                    for m in resources.script_engine.new_popups() {
                        resources.messages.add(m);
                    }

                    #[cfg(debug_assertions)]
                    if input.is_pressed(VirtualKey::DebugRestart) {
                        stop_all_coroutines();
                        assets.next_scene = Some((
                            // skip the transition for faster debugging
                            new_prelevel(resources.stats.info.clone(), true).await,
                            TransitionEffectType::Shatter,
                        ));
                    }
                    #[cfg(debug_assertions)]
                    let won = input.is_pressed(VirtualKey::DebugWin)
                        || resources.script_engine.win_flag();
                    #[cfg(not(debug_assertions))]
                    let won = resources.script_engine.win_flag();
                    if won {
                        stop_all_coroutines();
                        assets.next_scene = Some((
                            crate::scene::Scene::PostLevel(resources.stats.clone()),
                            TransitionEffectType::Shatter,
                        ));
                    }

                    input.reset();
                    resources.messages.update();
                    resources.selector.update();
                    resources.stats.frames += 1;
                    renderer.tick();

                    /* if resources.stats.frames % 100 == 0 {
                        resources.body_index.debug();
                    } */
                }
            }
            Scene::PostLevel(stats) => {
                for _ in 0..clock.get_num_updates() {
                    renderer.tick();
                }
                if input.is_any_pressed() {
                    let info = assets.get_next_level(&stats.info);
                    assets.next_scene = Some((
                        new_prelevel(info, false).await,
                        TransitionEffectType::Shatter,
                    ));
                }
            }
        }

        renderer.render_scene(&scene, &assets);
        next_frame().await;
    }
}
