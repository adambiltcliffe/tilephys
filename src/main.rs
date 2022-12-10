use camera::PlayerCamera;
use enemy::Enemy;
use hecs::CommandBuffer;
use input::{Input, VirtualKey};
use loader::LoadingManager;
use macroquad::prelude::*;
use physics::{Actor, ConstantMotion, PathMotion, Projectile};
use pickup::Pickup;
use player::Controller;
use render::Renderer;
use resources::Resources;
use scene::{NewScene, Scene};
use timer::Timer;
use transition::TransitionEffectType;

mod camera;
mod draw;
mod enemy;
mod input;
mod loader;
mod messages;
mod physics;
mod pickup;
mod player;
mod render;
mod resources;
mod scene;
mod script;
mod stats;
mod timer;
mod transition;
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
    let mut loader = LoadingManager::new();
    // need a way to initialise resources without loading a level
    let (mut scene, mut resources): (Scene, Resources) =
        loader.load_level("intro.tmx").await.unwrap();
    scene = Scene::PreLevel;

    let mut renderer = Renderer::new(RENDER_W, RENDER_H);
    let mut clock = Timer::new();
    let mut input = Input::new();

    loop {
        match resources.new_scene {
            None => (),
            Some((NewScene::PreLevel, typ)) => {
                renderer.start_transition(typ);
                scene = Scene::PreLevel;
                resources.new_scene = None;
                println!("transitioning to prelevel");
            }
            Some((NewScene::PlayLevel, typ)) => {
                renderer.start_transition(typ);
                (scene, resources) = loader.load_level("intro.tmx").await.unwrap();
                clock = Timer::new();
                input = Input::new();
            }
            Some((NewScene::PostLevel, typ)) => {
                renderer.start_transition(typ);
                scene = Scene::PostLevel;
                resources.new_scene = None;
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
                    resources.new_scene = Some((NewScene::PlayLevel, TransitionEffectType::Open))
                }
            }
            Scene::PlayLevel(ref world_ref) => {
                for _ in 0..clock.get_num_updates() {
                    let mut world = world_ref.borrow_mut();
                    let mut buffer = CommandBuffer::new();
                    ConstantMotion::apply(&world);
                    PathMotion::apply(&world);
                    let (new_triggers, new_secrets) =
                        Controller::update(&world, &mut resources, &mut buffer, &input);
                    Enemy::update(&world, &resources);
                    Actor::update(&world);
                    Projectile::update(&world, &mut resources, &mut buffer);
                    Pickup::update(&world, &mut resources, &mut buffer);
                    buffer.run_on(&mut world);

                    PlayerCamera::update(&world, &mut resources);

                    drop(world);

                    if new_secrets > 0 {
                        resources.messages.add("Found a secret area!".to_owned());
                    }

                    for t in new_triggers {
                        println!("entered new trigger zone {}", t);
                        resources
                            .script_engine
                            .call_entry_point(&format!("{}_enter", t));
                    }

                    if input.is_pressed(VirtualKey::DebugRestart) {
                        resources.new_scene = Some((
                            crate::scene::NewScene::PlayLevel,
                            TransitionEffectType::Shatter,
                        ));
                    }
                    if input.is_pressed(VirtualKey::DebugWin) || resources.script_engine.win_flag()
                    {
                        resources.new_scene = Some((
                            crate::scene::NewScene::PostLevel,
                            TransitionEffectType::Shatter,
                        ));
                    }

                    input.reset();
                    resources.messages.update();
                    resources.stats.frames += 1;
                    renderer.tick();
                }
            }
            Scene::PostLevel => {
                for _ in 0..clock.get_num_updates() {
                    renderer.tick();
                }
                if input.is_any_pressed() {
                    resources.new_scene = Some((
                        crate::scene::NewScene::PreLevel,
                        TransitionEffectType::Shatter,
                    ));
                }
            }
        }

        renderer.draw_scene(&scene, &resources, clock.get_fps());
        next_frame().await;
    }
}
