#![feature(trace_macros)]

use std::num::NonZeroU8;

use camera::PlayerCamera;
use enemy::update_enemies;
use hecs::CommandBuffer;
use input::Input;
use log::warn;
use macroquad::experimental::coroutines::{start_coroutine, stop_all_coroutines};
use macroquad::prelude::*;
use physics::{Actor, PathMotion};
use pickup::{Pickup, WeaponPickup};
use player::Controller;
use profile::{Phase, Profiler};
use projectile::Projectile;
use render::Renderer;
use resources::{load_assets, Inventory};
use scene::{new_prelevel, Scene};
use timer::Timer;
use transition::TransitionEffectType;
use vfx::update_vfx;

#[cfg(debug_assertions)]
use console::{ConsoleEntryType, CONSOLE};
#[cfg(debug_assertions)]
use enum_iterator::all;
#[cfg(debug_assertions)]
use input::VirtualKey;
#[cfg(debug_assertions)]
use script::BasicEngine;
#[cfg(debug_assertions)]
use weapon::{add_ammo, AmmoType};

mod camera;
mod config;
mod draw;
mod enemy;
mod index;
mod input;
mod level;
mod loader;
mod log;
mod messages;
mod physics;
mod pickup;
mod player;
mod profile;
mod projectile;
mod ray;
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

#[cfg(debug_assertions)]
mod console;

pub(crate) const RENDER_W: u32 = 320;
pub(crate) const RENDER_H: u32 = 200;

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
    let mut profiler = Profiler::new();

    #[cfg(debug_assertions)]
    let mut show_profile = false;
    #[cfg(debug_assertions)]
    let mut basic_engine = BasicEngine::new();

    let coro = start_coroutine(load_assets());
    let mut loading_frames = 0;
    let assets_result = loop {
        loading_frames += 1;
        if loading_frames > 2 {
            renderer.render_loading();
        }
        next_frame().await;
        if let Some(res) = coro.retrieve() {
            break res;
        }
    };
    let mut assets = match assets_result {
        Err(e) => {
            // we literally can't do anything if we couldn't load the global assets
            // apart from display this one screen
            warn(&format!("{:?}", e));
            loop {
                renderer.render_hard_error(&e.to_string());
                next_frame().await
            }
        }
        Ok(res) => res,
    };

    let mut scene: Scene = if argv.len() > 1 {
        let info = assets.get_level_with_path(&argv[1]);
        new_prelevel(info, Inventory::new(), false).await
    } else {
        Scene::Title(0)
    };

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

        #[cfg(debug_assertions)]
        {
            let mut con = CONSOLE.lock().unwrap();
            if input.is_pressed(VirtualKey::DebugConsole) {
                con.toggle_visible();
            }
            if con.is_visible() {
                if is_key_pressed(KeyCode::Escape) {
                    con.escape();
                }
                if is_key_pressed(KeyCode::Enter) {
                    if let Some(cmd) = con.take_input() {
                        con.add(cmd.clone(), ConsoleEntryType::Input);
                        drop(con);
                        let (out_typ, out) = match &mut scene {
                            Scene::PlayLevel(ref mut resources) => {
                                resources.script_engine.exec(&cmd)
                            }
                            _ => basic_engine.exec(&cmd),
                        };
                        if !out.is_empty() {
                            let mut con = CONSOLE.lock().unwrap();
                            con.add(out, out_typ);
                        }
                    }
                }
                input.reset(); // suppress all other input
            }
        }

        match &mut scene {
            Scene::Title(ref mut frames) => {
                for _ in 0..clock.get_num_updates() {
                    renderer.tick();
                }
                *frames = (*frames + 1) % 128;
                if input.is_any_pressed() {
                    let info = assets.get_first_level();
                    assets.next_scene = Some((
                        new_prelevel(info, Inventory::new(), false).await,
                        TransitionEffectType::Shatter,
                    ));
                }
            }
            Scene::PreLevel(_n, coro, fast) => {
                for _ in 0..clock.get_num_updates() {
                    renderer.tick();
                }
                if (*fast || renderer.transition_finished()) && coro.is_done() {
                    let res = coro.retrieve().unwrap();
                    assets.next_scene = Some(match res {
                        Ok(scene) => (scene, TransitionEffectType::Open),
                        Err(e) => {
                            warn(&format!("{:?}", e));
                            (Scene::Error(e.to_string()), TransitionEffectType::Shatter)
                        }
                    });
                }
            }
            Scene::PlayLevel(ref mut resources) => {
                for _ in 0..clock.get_num_updates() {
                    let mut buffer = CommandBuffer::new();
                    profiler.start(Phase::Motion);
                    PathMotion::apply(resources);
                    profiler.start(Phase::Pickups);
                    Pickup::update(resources, &mut buffer);
                    WeaponPickup::update(resources);
                    profiler.start(Phase::Player);
                    Controller::update(resources, &mut buffer, &input);
                    profiler.start(Phase::Enemies);
                    update_enemies(resources, &mut buffer);
                    profiler.start(Phase::Actor);
                    Actor::update(resources);
                    profiler.start(Phase::Projectile);
                    Projectile::update(resources, &mut buffer);
                    profiler.start(Phase::Vfx);
                    update_vfx(resources, &mut buffer);
                    profiler.stop();
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
                                new_prelevel(resources.stats.info.clone(), Inventory::new(), false)
                                    .await,
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
                    if input.is_pressed(VirtualKey::DebugProfile) {
                        show_profile = !show_profile;
                    }
                    #[cfg(debug_assertions)]
                    if input.is_pressed(VirtualKey::DebugAmmo) {
                        for typ in all::<AmmoType>() {
                            add_ammo(
                                &mut resources.weapons,
                                &mut resources.ammo,
                                &mut resources.selector,
                                typ,
                                5,
                            );
                        }
                    }
                    #[cfg(debug_assertions)]
                    if input.is_pressed(VirtualKey::DebugRestart) {
                        stop_all_coroutines();
                        assets.next_scene = Some((
                            // skip the transition for faster debugging
                            new_prelevel(resources.stats.info.clone(), Inventory::new(), true)
                                .await,
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
                            crate::scene::Scene::PostLevel(
                                resources.stats.clone(),
                                resources.persist_inventory(),
                            ),
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
            Scene::PostLevel(stats, inv) => {
                for _ in 0..clock.get_num_updates() {
                    renderer.tick();
                }
                if input.is_any_pressed() {
                    let info = assets.get_next_level(&stats.info);
                    assets.next_scene = Some((
                        new_prelevel(info, inv.clone(), false).await,
                        TransitionEffectType::Shatter,
                    ));
                }
            }
            Scene::Error(_) => {
                for _ in 0..clock.get_num_updates() {
                    renderer.tick();
                }
                if input.is_any_pressed() {
                    // restart the game (will eventually go to title instead of first level)
                    assets.next_scene = Some((
                        new_prelevel(assets.get_first_level(), Inventory::new(), false).await,
                        TransitionEffectType::Shatter,
                    ));
                }
            }
        }

        renderer.render_scene(&scene, &assets, &mut profiler);
        #[cfg(debug_assertions)]
        {
            if show_profile {
                profiler.draw();
            }
            CONSOLE.lock().unwrap().draw();
        }
        next_frame().await;
    }
}
