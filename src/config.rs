#[cfg(debug_assertions)]
mod dynamic_config {
    use once_cell::sync::Lazy;
    use std::sync::{Mutex, MutexGuard};

    pub struct DynamicConfig {
        gravity: f32,
    }

    impl DynamicConfig {
        fn new() -> Self {
            Self { gravity: 0.5 }
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
}

#[cfg(debug_assertions)]
pub use dynamic_config::*;

#[cfg(not(debug_assertions))]
mod fixed_config {
    pub struct FixedConfig {}

    impl FixedConfig {
        #[inline(always)]
        pub fn gravity(&self) -> f32 {
            0.5
        }
    }

    pub fn config() -> FixedConfig {
        FixedConfig {}
    }
}

#[cfg(not(debug_assertions))]
pub use fixed_config::*;
