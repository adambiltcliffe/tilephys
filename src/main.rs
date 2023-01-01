use camera::PlayerCamera;
use enemy::update_enemies;
use hecs::CommandBuffer;
use input::{Input, VirtualKey};
use loader::LoadingManager;
use macroquad::prelude::*;
use physics::{Actor, ConstantMotion, PathMotion};
use pickup::Pickup;
use player::Controller;
use projectile::Projectile;
use render::Renderer;
use resources::load_assets;
use scene::{NewScene, Scene};
use timer::Timer;
use transition::TransitionEffectType;
use vfx::update_vfx;

mod camera;
mod draw;
mod enemy;
mod index;
mod input;
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
        window_title: "Platform tile physics test".to_owned(),
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
    let name = if argv.len() > 1 {
        argv[1].clone()
    } else {
        "intro".to_owned()
    };

    let mut loader = LoadingManager::new();
    let mut assets = load_assets().await;
    let mut scene = Scene::PreLevel;

    let mut renderer = Renderer::new(RENDER_W, RENDER_H);
    let mut clock = Timer::new();
    let mut input = Input::new();

    loop {
        match assets.new_scene {
            None => (),
            Some((NewScene::PreLevel, typ)) => {
                renderer.start_transition(typ);
                scene = Scene::PreLevel;
                assets.new_scene = None;
                println!("transitioning to prelevel");
            }
            Some((NewScene::PlayLevel, typ)) => {
                renderer.start_transition(typ);
                scene = loader.load_level(&name).await.unwrap();
                assets.new_scene = None;
                clock = Timer::new();
                input = Input::new();
                println!("transitioning to level");
            }
            Some((NewScene::PostLevel(stats), typ)) => {
                renderer.start_transition(typ);
                scene = Scene::PostLevel(stats);
                assets.new_scene = None;
                println!("transitioning to postlevel");
            }
        }

        input.update();

        match scene {
            Scene::PreLevel => {
                for _ in 0..clock.get_num_updates() {
                    renderer.tick();
                }
                if renderer.transition_finished() {
                    assets.new_scene = Some((NewScene::PlayLevel, TransitionEffectType::Open))
                }
            }
            Scene::PlayLevel(ref mut resources) => {
                for _ in 0..clock.get_num_updates() {
                    let mut buffer = CommandBuffer::new();
                    ConstantMotion::apply(resources);
                    PathMotion::apply(resources);
                    Controller::update(resources, &mut buffer, &input);
                    update_enemies(resources, &mut buffer);
                    Actor::update(resources);
                    Projectile::update(resources, &mut buffer);
                    Pickup::update(resources, &mut buffer);
                    update_vfx(resources, &mut buffer);
                    buffer.run_on(&mut resources.world_ref.borrow_mut());

                    PlayerCamera::update(resources);

                    if input.is_pressed(VirtualKey::DebugKill) {
                        resources
                            .world_ref
                            .borrow_mut()
                            .get::<&mut Controller>(resources.player_id)
                            .unwrap()
                            .hp = 0
                    }

                    for t in &resources.triggers {
                        resources.script_engine.call_entry_point(&t);
                    }
                    resources.triggers.clear();

                    if input.is_pressed(VirtualKey::DebugRestart) {
                        assets.new_scene = Some((
                            crate::scene::NewScene::PlayLevel,
                            TransitionEffectType::Shatter,
                        ));
                    }
                    if input.is_pressed(VirtualKey::DebugWin) || resources.script_engine.win_flag()
                    {
                        assets.new_scene = Some((
                            crate::scene::NewScene::PostLevel(resources.stats.clone()),
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
                    assets.new_scene = Some((
                        crate::scene::NewScene::PreLevel,
                        TransitionEffectType::Shatter,
                    ));
                }
            }
        }

        renderer.draw_scene(&scene, &assets);
        next_frame().await;
    }
}
