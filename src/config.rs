#[cfg(debug_assertions)]
use paste::paste;

#[cfg(debug_assertions)]
macro_rules! make_config {
    [$(($name:ident, $str_name:tt, $typ:ty, $val:expr)),*] => {
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
    [$(($name:ident, $str_name:tt, $typ:ty, $val:tt)),*] => {
        pub struct FixedConfig {}

        impl FixedConfig {
            $(
                #[inline(always)]
                pub fn $name(&self) -> $typ {
                    $val
                }
            )*
        }

        pub fn config() -> FixedConfig {
            FixedConfig {}
        }
    };
}

make_config![
    (gravity, "gravity", f32, 1.0),
    (player_accel, "player_accel", f32, 3.0),
    (player_drag, "player_drag", f32, 0.6),
    (flyer_drag, "flyer_drag", f32, 0.99),
    (actor_drag, "actor_drag", f32, 0.4),
    (recoil, "recoil", f32, 10.0),
    (rg_damage, "rg_damage", i32, 5),
    (rg_thickness, "rg_thickness", f32, 1.0),
    (rg_frames, "rg_frames", i32, 3),
    (rg_xoff1, "rg_xoff1", i32, 7),
    (rg_xoff2, "rg_xoff2", i32, 11),
    (rg_yoff, "rg_yoff", i32, 14),
    (rg_smoke_da, "rg_smoke_da", f32, 1.0),
    (rg_smoke_sp, "rg_smoke_sp", f32, 5.0),
    (rg_smoke_r, "rg_smoke_r", f32, 4.0),
    (drone_bob_h, "drone_bob_h", f32, 8.0),
    (drone_bob_a, "drone_bob_a", f32, 0.1),
    (drone_sensor_w, "drone_sensor_w", i32, 48),
    (drone_sensor_h, "drone_sensor_h", i32, 64),
    (drone_lock_frames, "drone_lock_frames", i32, 20),
    (ft_shotgun, "ft_shotgun", i32, 15),
    (ft_super_shotgun, "ft_super_shotgun", i32, 20),
    (ft_railgun, "ft_railgun", i32, 20),
    (valkyrie_max_vy, "valkyrie_max_vy", f32, 6.0),
    (valkyrie_lift, "valkyrie_lift", f32, 1.75),
    (boss_leg_length, "boss_leg_length", f32, 60.0)
];
