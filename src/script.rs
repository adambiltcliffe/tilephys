use crate::loader::LoadedMap;
use crate::physics::{ConstantMotion, PathMotion, TileBody};
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
        let scope = Scope::new();

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
        engine.register_fn(
            "set_path_motion",
            move |body_name: &str, path_name: &str, speed: f32, cycle: bool| {
                let id = cloned_body_ids[body_name];
                let mut world = cloned_world.borrow_mut();
                let (x, y) = {
                    let body = world.get::<&TileBody>(id).unwrap();
                    (body.x as f32, body.y as f32)
                };
                world
                    .insert_one(
                        id,
                        PathMotion::new(x, y, cloned_paths[path_name].clone(), speed, cycle),
                    )
                    .unwrap();
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
