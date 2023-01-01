use crate::physics::{ConstantMotion, PathMotion, PathMotionType, TileBody};
use hecs::{Entity, World};
use macroquad::file::load_string;
use rhai::{Engine, Scope, AST};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct ScriptFlags {
    win: bool,
}

pub struct ScriptEngine {
    engine: Engine,
    scope: Scope<'static>,
    ast: Option<AST>,
    flags: Rc<RefCell<ScriptFlags>>,
}

impl ScriptEngine {
    pub(crate) fn new(
        world_ref: Rc<RefCell<World>>,
        body_ids: Rc<HashMap<String, Entity>>,
        paths: Rc<HashMap<String, Vec<(f32, f32)>>>,
    ) -> Self {
        let mut engine = Engine::new();
        let mut scope = Scope::new();
        let flags = Rc::new(RefCell::new(ScriptFlags { win: false }));

        engine.register_type_with_name::<PathMotionType>("PathMotionType");
        scope.push("Static", PathMotionType::Static);
        scope.push("ForwardOnce", PathMotionType::ForwardOnce);
        scope.push("ForwardCycle", PathMotionType::ForwardCycle);

        let cloned_world = Rc::clone(&world_ref);
        let cloned_body_ids = Rc::clone(&body_ids);
        engine.register_fn(
            "set_constant_motion",
            move |name: &str, vx: i32, vy: i32| {
                cloned_world
                    .borrow_mut()
                    .insert_one(cloned_body_ids[name], ConstantMotion::new(vx, vy))
                    .unwrap();
            },
        );

        let cloned_world = Rc::clone(&world_ref);
        let cloned_body_ids = Rc::clone(&body_ids);
        let cloned_paths = Rc::clone(&paths);
        engine.register_fn("set_path", move |body_name: &str, path_name: &str| {
            let id = cloned_body_ids[body_name];
            let mut world = cloned_world.borrow_mut();
            let (x, y) = {
                let body = world.get::<&TileBody>(id).unwrap();
                (body.x as f32, body.y as f32)
            };
            world
                .insert_one(
                    id,
                    PathMotion::new(
                        path_name,
                        x,
                        y,
                        cloned_paths[path_name].clone(),
                        0.0,
                        PathMotionType::Static,
                    ),
                )
                .unwrap();
        });

        let cloned_world = Rc::clone(&world_ref);
        let cloned_body_ids = Rc::clone(&body_ids);
        engine.register_fn(
            "set_motion",
            move |body_name: &str, motion_type: PathMotionType, speed: f32| {
                let id = cloned_body_ids[body_name];
                let world = cloned_world.borrow_mut();
                let mut pm = world.get::<&mut PathMotion>(id).unwrap();
                pm.motion_type = motion_type;
                pm.speed = speed;
            },
        );

        let cloned_world = Rc::clone(&world_ref);
        let cloned_body_ids = Rc::clone(&body_ids);
        engine.register_fn(
            "set_motion_goto",
            move |body_name: &str, index: i32, speed: f32| {
                let id = cloned_body_ids[body_name];
                let world = cloned_world.borrow_mut();
                let mut pm = world.get::<&mut PathMotion>(id).unwrap();
                pm.motion_type = PathMotionType::GoToNode(index as usize);
                pm.speed = speed;
            },
        );

        let cloned_flags = Rc::clone(&flags);
        engine.register_fn("win", move || {
            cloned_flags.borrow_mut().win = true;
        });

        Self {
            engine,
            scope,
            ast: None,
            flags,
        }
    }

    pub async fn load_file(&mut self, path: &str) {
        self.ast = Some(
            self.engine
                .compile(load_string(path.into()).await.unwrap())
                .unwrap(),
        );
    }

    pub fn call_entry_point(&mut self, name: &str) {
        match &self.ast {
            None => panic!("no script loaded"),
            Some(ast) => self
                .engine
                .call_fn::<()>(&mut self.scope, &ast, name, ())
                .unwrap_or_else(|_| println!("calling entry point {} failed", name)),
        }
    }

    pub fn win_flag(&self) -> bool {
        self.flags.borrow().win
    }
}
