#[cfg(debug_assertions)]
mod dynamic_config {
    use once_cell::sync::Lazy;
    use rhai::plugin::*;
    use rhai::{def_package, export_module};
    use std::sync::{Mutex, MutexGuard};

    pub struct DynamicConfig {
        gravity: f32,
    }

    impl DynamicConfig {
        fn new() -> Self {
            Self { gravity: 1.0 }
        }

        pub fn gravity(&self) -> f32 {
            self.gravity
        }
    }

    static INSTANCE: Lazy<Mutex<DynamicConfig>> = Lazy::new(|| Mutex::new(DynamicConfig::new()));

    pub fn config() -> MutexGuard<'static, DynamicConfig> {
        INSTANCE.try_lock().unwrap()
    }

    #[derive(Clone)]
    pub struct ConfigProxy {}

    #[export_module]
    mod config_interface {
        #[rhai_fn(get = "gravity")]
        pub fn get_gravity(_this: &mut ConfigProxy) -> f32 {
            config().gravity
        }

        #[rhai_fn(set = "gravity")]
        pub fn set_gravity(_this: &mut ConfigProxy, val: f32) {
            config().gravity = val;
        }
    }

    def_package! {
        pub ConfigPackage(module) {
            combine_with_exported_module!(module, "config-mod", config_interface);
        } |> |engine| {
            engine.register_type_with_name::<ConfigProxy>("Config");
        }
    }
}

#[cfg(debug_assertions)]
pub use dynamic_config::*;

#[cfg(not(debug_assertions))]
mod fixed_config {
    pub struct FixedConfig {}

    impl FixedConfig {
        #[inline(always)]
        pub fn gravity(&self) -> f32 {
            1.0
        }
    }

    pub fn config() -> FixedConfig {
        FixedConfig {}
    }
}

#[cfg(not(debug_assertions))]
pub use fixed_config::*;
