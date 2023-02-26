#[cfg(debug_assertions)]
use paste::paste;

/*#[cfg(debug_assertions)]
macro_rules! make_setter_getter {
    ($name:ident, $str_name:tt, $typ:ty) => {

    }
}*/

#[cfg(debug_assertions)]
macro_rules! make_config {
    [$(($name:ident, $str_name:tt, $typ:ty, $val:tt)),*] => {
        paste! {
            use once_cell::sync::Lazy;
            use rhai::plugin::*;
            use rhai::{def_package, export_module};
            use std::sync::{Mutex, MutexGuard};

            pub struct DynamicConfig {
                $($name: $typ, )*
            }

            impl DynamicConfig {
                fn new() -> Self {
                    Self {
                        $($name: $val, )*
                    }
                }

                $(
                    pub fn $name(&self) -> $typ {
                        self.$name
                    }
                )*
            }

            static INSTANCE: Lazy<Mutex<DynamicConfig>> =
                Lazy::new(|| Mutex::new(DynamicConfig::new()));

            pub fn config() -> MutexGuard<'static, DynamicConfig> {
                INSTANCE.try_lock().unwrap()
            }

            #[derive(Clone)]
            pub struct ConfigProxy {}

            #[export_module]
            mod config_interface {

                    $(
                        #[rhai_fn(get = $str_name)]
                        pub fn [<get_ $name>](_this: &mut ConfigProxy) -> $typ {
                            config().$name
                        }

                        #[rhai_fn(set = $str_name)]
                        pub fn [<set_ $name>](_this: &mut ConfigProxy, val: $typ) {
                            config().$name = val;
                        }
                    )*

            }

            def_package! {
                pub ConfigPackage(module) {
                    combine_with_exported_module!(module, "config-mod", config_interface);
                } |> |engine| {
                    engine.register_type_with_name::<ConfigProxy>("Config");
                }
            }
        }
    }
}

#[cfg(not(debug_assertions))]
macro_rules! make_config {
    () => {
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
    };
}

make_config![(gravity, "gravity", f32, 1.0)];
