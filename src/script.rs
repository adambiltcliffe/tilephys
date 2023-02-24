use crate::physics::{PathMotion, PathMotionType, TileBody};
use crate::switch::Switch;
use hecs::{Entity, World};
use macroquad::file::load_string;
use rhai::packages::{Package, StandardPackage};
use rhai::plugin::*;
use rhai::{def_package, Engine, FnPtr, Scope, AST};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

#[cfg(debug_assertions)]
use crate::console::{CommandExecutor, ConsoleEntryType};

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
    new_popups: Vec<String>,
}

impl ScriptFlags {
    fn new() -> Self {
        Self {
            win: false,
            queued_funcs: Vec::new(),
            new_popups: Vec::new(),
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
    }

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

    // Context methods

    pub fn popup(this: &mut Flags, msg: ImmutableString) {
        this.lock().unwrap().new_popups.push(msg.to_string());
    }

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
    pub buffer: Arc<Mutex<VecDeque<String>>>,
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
        //let bsp = BasicStringPackage::new();
        //bsp.register_into_engine(&mut engine);
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
        let buffer = Arc::new(Mutex::new(VecDeque::new()));

        #[cfg(debug_assertions)]
        {
            let buf = Arc::clone(&buffer);
            engine.on_print(move |msg| {
                println!("{}", msg);
                buf.lock().unwrap().push_back(msg.to_owned());
            });
            let buf = Arc::clone(&buffer);
            engine.on_debug(move |msg, src, pos| {
                let line = format!("{:?}@{:?}: {}", src, pos, msg);
                println!("{}", &line);
                buf.lock().unwrap().push_back(msg.to_owned());
            });
        }

        Self {
            engine,
            scope,
            ast: None,
            flags,
            buffer,
        }
    }

    pub async fn load_file(&mut self, path: &str) {
        self.ast = Some(
            self.engine
                .compile(load_string(path).await.unwrap())
                .unwrap(),
        );
    }

    pub fn call_entry_point(&mut self, name: &str) {
        match &self.ast {
            None => panic!("no script loaded"),
            Some(ast) => self
                .engine
                .call_fn::<()>(&mut self.scope, ast, name, ())
                .unwrap_or_else(|err| match *err {
                    // if the entry point itself didn't exist, that's not an error
                    EvalAltResult::ErrorFunctionNotFound(fname, _) if name == fname => (),
                    _ => {
                        println!("calling entry point {} failed: {:?}", name, err)
                    }
                }),
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
            f.call::<()>(&self.engine, self.ast.as_ref().unwrap(), ())
                .unwrap();
        }
    }

    pub fn new_popups(&mut self) -> Vec<String> {
        self.flags.lock().unwrap().new_popups.drain(..).collect()
    }

    pub fn win_flag(&self) -> bool {
        self.flags.lock().unwrap().win
    }
}

#[cfg(debug_assertions)]
impl CommandExecutor for ScriptEngine {
    fn exec(&mut self, command: &str) -> (ConsoleEntryType, String) {
        match self
            .engine
            .eval_with_scope::<Dynamic>(&mut self.scope, command)
        {
            Ok(v) => (ConsoleEntryType::Output, format!("{}", v)),
            Err(e) => (ConsoleEntryType::InteractiveError, format!("{}", e)),
        }
    }
}

#[cfg(debug_assertions)]
pub struct BasicEngine {
    engine: Engine,
}

#[cfg(debug_assertions)]
impl BasicEngine {
    pub(crate) fn new() -> Self {
        Self {
            engine: Engine::new_raw(),
        }
    }
}

#[cfg(debug_assertions)]
impl CommandExecutor for BasicEngine {
    fn exec(&mut self, command: &str) -> (ConsoleEntryType, String) {
        match self.engine.eval::<Dynamic>(command) {
            Ok(v) => (ConsoleEntryType::Output, format!("{}", v)),
            Err(e) => (ConsoleEntryType::InteractiveError, format!("{}", e)),
        }
    }
}
