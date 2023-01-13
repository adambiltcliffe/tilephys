use crate::physics::{PathMotion, PathMotionType, TileBody};
use crate::switch::Switch;
use hecs::{Entity, World};
use macroquad::file::load_string;
use rhai::packages::{Package, StandardPackage};
use rhai::plugin::*;
use rhai::{def_package, Engine, Scope, AST};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct ScriptEntityProxy {
    world_ref: Arc<Mutex<World>>,
    id: Entity,
}

impl ScriptEntityProxy {
    pub fn new(world_ref: Arc<Mutex<World>>, id: Entity) -> Self {
        Self { world_ref, id }
    }
}

#[export_module]
mod script_interface {
    pub type EntityProxy = ScriptEntityProxy;
    pub type Flags = Arc<Mutex<ScriptFlags>>;

    pub fn report(this: &mut EntityProxy) {
        println!("I am an EntityProxy with id {:?}", this.id);
    }

    // TileBody methods

    pub fn set_motion(this: &mut EntityProxy, motion_type: PathMotionType, speed: f32) {
        let world = this.world_ref.lock().unwrap();
        let mut pm = world.get::<&mut PathMotion>(this.id).unwrap(); // fails if no path set
        pm.motion_type = motion_type;
        pm.speed = speed;
    }

    pub fn go_to(this: &mut EntityProxy, index: i32, speed: f32) {
        let world = this.world_ref.lock().unwrap();
        let mut pm = world.get::<&mut PathMotion>(this.id).unwrap();
        pm.set_dest_node(index as usize);
        pm.speed = speed;
    }

    // Switch methods

    pub fn set_enabled(this: &mut EntityProxy, on: bool) {
        let world = this.world_ref.lock().unwrap();
        let mut s = world.get::<&mut Switch>(this.id).unwrap(); // fails if not switch
        s.enabled = on;
    }

    // Flags methods

    pub fn win(this: &mut Flags) {
        this.lock().unwrap().win = true;
    }
}

def_package! {
    pub ScriptPackage(module): StandardPackage {
        combine_with_exported_module!(module, "script-mod", script_interface);
    } |> |engine| {
        engine.register_type_with_name::<PathMotionType>("PathMotionType");
    }
}

pub struct ScriptFlags {
    win: bool,
}

pub struct ScriptEngine {
    engine: Engine,
    scope: Scope<'static>,
    ast: Option<AST>,
    flags: Arc<Mutex<ScriptFlags>>,
}

impl ScriptEngine {
    pub(crate) fn new(
        world_ref: Arc<Mutex<World>>,
        ids: Arc<HashMap<String, Entity>>,
        paths: Arc<HashMap<String, Vec<(f32, f32)>>>,
    ) -> Self {
        let mut engine = Engine::new();
        let mut scope = Scope::new();
        let flags = Arc::new(Mutex::new(ScriptFlags { win: false }));

        let pkg = ScriptPackage::new();
        pkg.register_into_engine(&mut engine);
        scope.push("flags", Arc::clone(&flags));
        scope.push("static", PathMotionType::Static);
        scope.push("forward_once", PathMotionType::ForwardOnce);
        scope.push("forward_cycle", PathMotionType::ForwardCycle);
        for (name, id) in ids.iter() {
            scope.push(name, ScriptEntityProxy::new(Arc::clone(&world_ref), *id));
        }

        let cloned_world = Arc::clone(&world_ref);
        let cloned_ids = Arc::clone(&ids);
        let cloned_paths = Arc::clone(&paths);
        engine.register_fn("set_path", move |body_name: &str, path_name: &str| {
            let id = cloned_ids[body_name];
            let mut world = cloned_world.lock().unwrap();
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
        self.flags.lock().unwrap().win
    }
}
