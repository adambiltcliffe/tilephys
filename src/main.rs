use camera::PlayerCamera;
use enemy::update_enemies;
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
    // need a way to initialise resources without loading a level
    let (mut scene, mut resources): (Scene, Resources) = loader.load_level(&name).await.unwrap();
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
                (scene, resources) = loader.load_level(&name).await.unwrap();
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
                    ConstantMotion::apply(&world, &mut resources);
                    PathMotion::apply(&world, &mut resources);
                    Controller::update(&world, &mut resources, &mut buffer, &input);
                    update_enemies(&world, &resources);
                    Actor::update(&world, &resources);
                    Projectile::update(&world, &mut resources, &mut buffer);
                    Pickup::update(&world, &mut resources, &mut buffer);
                    update_vfx(&world, &mut buffer);
                    buffer.run_on(&mut world);

                    PlayerCamera::update(&world, &mut resources);

                    if input.is_pressed(VirtualKey::DebugKill) {
                        world
                            .get::<&mut Controller>(resources.player_id)
                            .unwrap()
                            .hp = 0
                    }

                    drop(world);

                    for t in &resources.triggers {
                        println!("calling entry point {}", t);
                        resources.script_engine.call_entry_point(&t);
                    }
                    resources.triggers.clear();

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

                    if resources.stats.frames % 100 == 0 {
                        resources.body_index.debug();
                    }
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

        renderer.draw_scene(&scene, &resources);
        next_frame().await;
    }
}
