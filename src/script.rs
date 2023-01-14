use crate::physics::{PathMotion, PathMotionType, TileBody};
use crate::switch::Switch;
use hecs::{Entity, World};
use macroquad::file::load_string;
use rhai::packages::{Package, StandardPackage};
use rhai::plugin::*;
use rhai::{def_package, Engine, FnPtr, Scope, AST};
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

pub struct ScriptFlags {
    win: bool,
    queued_funcs: Vec<(rhai::INT, FnPtr)>,
}

impl ScriptFlags {
    fn new() -> Self {
        Self {
            win: false,
            queued_funcs: Vec::new(),
        }
    }
}

#[export_module]
mod script_interface {
    pub type EntityProxy = ScriptEntityProxy;
    pub type Path = Arc<Vec<(f32, f32)>>;
    pub type Flags = Arc<Mutex<ScriptFlags>>;

    // TileBody methods

    pub fn set_path(this: &mut EntityProxy, path: Path) {
        let mut world = this.world_ref.lock().unwrap();
        let (x, y) = {
            let body = world.get::<&TileBody>(this.id).unwrap();
            (body.x as f32, body.y as f32)
        };
        world
            .insert_one(
                this.id,
                PathMotion::new(x, y, &path, 0.0, PathMotionType::Static),
            )
            .unwrap();
        println!("set_path ok");
    }

    pub fn set_motion(this: &mut EntityProxy, motion_type: PathMotionType, speed: f32) {
        let world = this.world_ref.lock().unwrap();
        let mut pm = world.get::<&mut PathMotion>(this.id).unwrap(); // fails if no path set
        pm.motion_type = motion_type;
        pm.speed = speed;
        println!("set_motion ok");
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

    // Context methods

    pub fn after_frames(this: &mut Flags, n: rhai::INT, func: FnPtr) {
        this.lock().unwrap().queued_funcs.push((n, func));
    }

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
        let mut engine = Engine::new_raw();
        let mut scope = Scope::new();
        let flags = Arc::new(Mutex::new(ScriptFlags::new()));

        let pkg = ScriptPackage::new();
        pkg.register_into_engine(&mut engine);
        scope.push("context", Arc::clone(&flags));
        scope.push("static", PathMotionType::Static);
        scope.push("forward_once", PathMotionType::ForwardOnce);
        scope.push("forward_cycle", PathMotionType::ForwardCycle);
        for (name, id) in ids.iter() {
            scope.push(name, ScriptEntityProxy::new(Arc::clone(&world_ref), *id));
        }
        for (name, path) in paths.iter() {
            scope.push(name, Arc::new(path.clone()));
        }

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
                .unwrap_or_else(|err| println!("calling entry point {} failed: {:?}", name, err)),
        }
    }

    pub fn schedule_queued_funcs(&mut self) {
        let mut context = self.flags.lock().unwrap();
        let mut funcs = Vec::new();
        for (n, f) in &mut context.queued_funcs {
            *n -= 1;
            if *n == 0 {
                funcs.push(f.clone());
            }
        }
        context.queued_funcs.retain(|(n, _)| *n > 0);
        drop(context);
        for f in funcs {
            f.call::<()>(&self.engine, &self.ast.as_ref().unwrap(), ())
                .unwrap();
        }
    }

    pub fn win_flag(&self) -> bool {
        self.flags.lock().unwrap().win
    }
}
