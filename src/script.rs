use crate::loader::LoadedMap;
use crate::physics::{ConstantMotion, PathMotion, PathMotionType, TileBody};
use rhai::{Engine, Scope, AST};
use std::rc::Rc;

pub struct ScriptEngine {
    engine: Engine,
    scope: Scope<'static>,
    ast: Option<AST>,
}

impl ScriptEngine {
    pub(crate) fn new(map: &LoadedMap) -> Self {
        let mut engine = Engine::new();
        let mut scope = Scope::new();

        engine.register_type_with_name::<PathMotionType>("PathMotionType");
        scope.push("Static", PathMotionType::Static);
        scope.push("ForwardOnce", PathMotionType::ForwardOnce);
        scope.push("ForwardCycle", PathMotionType::ForwardCycle);

        let cloned_world = Rc::clone(&map.world_ref);
        let cloned_body_ids = Rc::clone(&map.body_ids);
        engine.register_fn(
            "set_constant_motion",
            move |name: &str, vx: i32, vy: i32| {
                cloned_world
                    .borrow_mut()
                    .insert_one(cloned_body_ids[name], ConstantMotion::new(vx, vy))
                    .unwrap();
            },
        );

        let cloned_world = Rc::clone(&map.world_ref);
        let cloned_body_ids = Rc::clone(&map.body_ids);
        let cloned_paths = Rc::clone(&map.paths);
        engine.register_fn("set_path", move |body_name: &str, path_name: &str| {
            let id = cloned_body_ids[body_name];
            let mut world = cloned_world.borrow_mut();
            let (x, y) = {
                let body = world.get::<&TileBody>(id).unwrap();
                (body.x as f32, body.y as f32)
            };
            println!("set_path at x:{}, y:{}", x, y);
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

        let cloned_world = Rc::clone(&map.world_ref);
        let cloned_body_ids = Rc::clone(&map.body_ids);
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

        let cloned_world = Rc::clone(&map.world_ref);
        let cloned_body_ids = Rc::clone(&map.body_ids);
        engine.register_fn(
            "set_motion_goto",
            move |body_name: &str, index: usize, speed: f32| {
                let id = cloned_body_ids[body_name];
                let world = cloned_world.borrow_mut();
                let mut pm = world.get::<&mut PathMotion>(id).unwrap();
                pm.motion_type = PathMotionType::GoToNode(index);
                pm.speed = speed;
            },
        );

        Self {
            engine,
            scope,
            ast: None,
        }
    }

    pub fn load_file(&mut self, path: &str) {
        self.ast = Some(self.engine.compile_file(path.into()).unwrap());
    }

    pub fn call_entry_point(&mut self, name: &str) {
        match &self.ast {
            None => panic!("no script loaded"),
            Some(ast) => self
                .engine
                .call_fn::<()>(&mut self.scope, &ast, name, ())
                .unwrap(),
        }
    }
}
