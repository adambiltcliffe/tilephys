use crate::draw::draw;
use crate::input::{Input, ScreenButtons};
use crate::messages::Messages;
use crate::player::Controller;
use crate::resources::{GlobalAssets, SceneResources};
use crate::scene::Scene;
use crate::stats::LevelStats;
use crate::transition::{new_transition, TransitionEffect, TransitionEffectType};
use crate::vfx::draw_vfx;
use crate::visibility::draw_visibility;
use enum_iterator::all;
use macroquad::prelude::*;
use miniquad::graphics::{BlendFactor, BlendState, BlendValue, Equation};

pub const WALL_VISION_DEPTH: f32 = 16.5;
const PARALLAX_FACTOR: f32 = 1.4;

enum Origin {
    TopLeft,
    BottomLeft,
}

fn zoom_coeff(o: Origin) -> f32 {
    match o {
        Origin::TopLeft => -2.,
        Origin::BottomLeft => 2.,
    }
}

fn get_camera_for_target(target: &RenderTarget, camera: Vec2, o: Origin) -> Camera2D {
    let width = target.texture.width();
    let height = target.texture.height();
    Camera2D {
        render_target: Some(*target),
        zoom: (vec2(2. / width, zoom_coeff(o) / height)),
        target: camera,
        ..Default::default()
    }
}

pub struct Renderer {
    pub width: f32,
    pub height: f32,
    final_width: f32,
    final_height: f32,
    transition: Option<(RenderTarget, Box<dyn TransitionEffect>)>,
    draw_target: RenderTarget,
    vis_targets: [RenderTarget; 2],
    outline_material: Material,
    jfa_init_material: Material,
    jfa_step_material: Material,
    jfa_final_material: Material,
}

impl Renderer {
    pub fn new(final_width: u32, final_height: u32) -> Self {
        let margin = WALL_VISION_DEPTH.ceil() as u32 * 2;
        let width = final_width + margin;
        let height = final_height + margin;
        let bs = BlendState::new(
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        );
        let outline_material = load_material(
            VERTEX_SHADER,
            OUTLINE_FRAGMENT_SHADER,
            MaterialParams {
                pipeline_params: PipelineParams {
                    color_blend: Some(bs),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .unwrap();
        let jfa_init_material = load_material(
            VERTEX_SHADER,
            JFA_INIT_FRAGMENT_SHADER,
            MaterialParams {
                ..Default::default()
            },
        )
        .unwrap();
        let jfa_step_material = load_material(
            VERTEX_SHADER_PASS_COLOR,
            JFA_STEP_FRAGMENT_SHADER,
            MaterialParams {
                ..Default::default()
            },
        )
        .unwrap();
        let jfa_final_material = load_material(
            VERTEX_SHADER,
            JFA_FINAL_FRAGMENT_SHADER,
            MaterialParams {
                pipeline_params: PipelineParams {
                    color_blend: Some(bs),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .unwrap();
        let draw_target = render_target(width, height);
        draw_target.texture.set_filter(FilterMode::Nearest);
        Self {
            width: width as f32,
            height: height as f32,
            final_width: final_width as f32,
            final_height: final_height as f32,
            transition: None,
            draw_target,
            vis_targets: [render_target(width, height), render_target(width, height)],
            outline_material,
            jfa_init_material,
            jfa_step_material,
            jfa_final_material,
        }
    }

    pub fn format_abs_pos(&self, pos: (f32, f32)) -> (f32, f32) {
        let x = pos.0;
        let y = pos.1;
        let sw = screen_width();
        let sh = screen_height();
        let scale = (sw / self.final_width)
            .min(sh / self.final_height)
            .floor()
            .max(1.);
        let zoomed_width = self.final_width * scale;
        let zoomed_height = self.final_height * scale;
        return ((x - ((sw - zoomed_width) / 2.))/scale, (y - ((sh - zoomed_height) / 2.))/scale);
    }

    pub(crate) fn render_scene(&self, scene: &Scene, assets: &GlobalAssets, input: &Input, level_name: &str) {
        // draw the current scene
        match scene {
            Scene::PreLevel(_, _) => {
                self.draw_prelevel(assets, level_name);
            }
            Scene::PlayLevel(resources) => {
                self.draw_world(resources, assets, input);
            }
            Scene::PostLevel(stats) => {
                self.draw_postlevel(stats);
            }
        }

        // draw the outgoing scene if there is one
        if let Some((ff, effect)) = &self.transition {
            gl_use_default_material();
            set_camera(&get_camera_for_target(
                &self.draw_target,
                vec2(self.width / 2., self.height / 2.),
                Origin::TopLeft,
            ));
            effect.draw(&ff.texture);
        }

        // finally draw to the screen
        self.render_to_screen();
    }

    fn render_to_screen(&self) {
        gl_use_default_material();
        let sw = screen_width();
        let sh = screen_height();
        set_camera(&Camera2D {
            zoom: (vec2(2. / sw, 2. / sh)),
            target: vec2(sw / 2., sh / 2.),
            ..Default::default()
        });
        let scale = (sw / self.final_width)
            .min(sh / self.final_height)
            .floor()
            .max(1.);
        let zoomed_width = self.final_width * scale;
        let zoomed_height = self.final_height * scale;
        draw_texture_ex(
            self.draw_target.texture,
            ((sw - zoomed_width) / 2.).floor(),
            ((sh - zoomed_height) / 2.).floor(),
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(
                    WALL_VISION_DEPTH.ceil(),
                    WALL_VISION_DEPTH.ceil(),
                    self.final_width,
                    self.final_height,
                )),
                dest_size: Some(vec2(zoomed_width, zoomed_height)),
                ..Default::default()
            },
        );
    }

    pub(crate) fn render_loading(&self) {
        gl_use_default_material();
        set_camera(&get_camera_for_target(
            &self.draw_target,
            vec2(self.width / 2., self.height / 2.),
            Origin::TopLeft,
        ));
        let wvdc = WALL_VISION_DEPTH.ceil();
        let msg = "STAND BY";
        let td1 = measure_text(msg, None, 32, 1.0);
        draw_text(
            msg,
            wvdc + 160.0 - td1.width / 2.,
            wvdc + 100.0,
            32.0,
            WHITE,
        );
        self.render_to_screen();
    }

    pub(crate) fn draw_prelevel(&self, assets: &GlobalAssets, level_name: &str) {
        gl_use_default_material();
        set_camera(&get_camera_for_target(
            &self.draw_target,
            vec2(self.width / 2., self.height / 2.),
            Origin::TopLeft,
        ));
        let wvdc = WALL_VISION_DEPTH.ceil();
        for x in 0..8 {
            for y in 0..5 {
                draw_texture(
                    assets.interstitial,
                    wvdc + x as f32 * 40.0,
                    wvdc + y as f32 * 40.0,
                    WHITE,
                );
            }
        }
        let td1 = measure_text(level_name, None, 32, 1.0);
        draw_text(
            level_name,
            wvdc + 160.0 - td1.width / 2.,
            wvdc + 100.0,
            32.0,
            WHITE,
        );
        let msg = if self.transition_finished() {
            "Entering"
        } else {
            "Loading"
        };
        let td2 = measure_text(msg, None, 16, 1.0);
        draw_text(
            msg,
            (wvdc + 160.0 - td2.width / 2.).floor(),
            (wvdc + 100.0 - td1.height - 6.0).floor(),
            16.0,
            WHITE,
        );
    }

    pub(crate) fn draw_postlevel(&self, stats: &LevelStats) {
        gl_use_default_material();
        set_camera(&get_camera_for_target(
            &self.draw_target,
            vec2(self.width / 2., self.height / 2.),
            Origin::TopLeft,
        ));
        let wvdc = WALL_VISION_DEPTH.ceil();
        if self.transition.is_some() {
            draw_texture(
                self.transition.as_ref().unwrap().0.texture,
                wvdc,
                wvdc,
                Color {
                    r: 0.2,
                    g: 0.,
                    b: 0.,
                    a: 1.0,
                },
            );
        }
        self.draw_centred_text("Completed", 16, 72.0);
        self.draw_centred_text("Entryway", 32, 100.0);
        self.draw_centred_text(&format!("Time: {}", stats.pretty_time()), 16, 128.0);
        self.draw_centred_text(
            &format!("Enemies defeated: {}/{}", stats.kills, stats.max_kills),
            16,
            144.0,
        );
        self.draw_centred_text(
            &format!("Items found: {}/{}", stats.items, stats.max_items),
            16,
            160.0,
        );
        self.draw_centred_text(
            &format!("Secrets entered: {}/{}", stats.secrets, stats.max_secrets),
            16,
            176.0,
        );
    }

    pub fn draw_centred_text(&self, text: &str, size: u16, y: f32) {
        let wvdc = WALL_VISION_DEPTH.ceil();
        let td1 = measure_text(text, None, size, 1.0);
        draw_text(
            text,
            (wvdc + 160.0 - td1.width / 2.).floor(),
            (wvdc + y).floor(),
            size as f32,
            WHITE,
        );
    }

    pub(crate) fn draw_world(&self, resources: &SceneResources, assets: &GlobalAssets, input: &Input) {
        gl_use_default_material();
        set_camera(&get_camera_for_target(
            &self.draw_target,
            vec2(self.width / 2., self.height / 2.),
            Origin::TopLeft,
        ));

        let mut world = resources.world_ref.lock().unwrap();
        let (flash, hp) = match world.get::<&Controller>(resources.player_id) {
            Ok(c) => (c.was_hurt(), c.hp),
            Err(_) => (false, 0),
        };
        if flash {
            clear_background(RED);
            return;
        }

        // draw the sky
        let wvdc = WALL_VISION_DEPTH.ceil();
        for x in -1..4 {
            for y in -1..3 {
                draw_texture(
                    assets.sky,
                    wvdc - (resources.camera_pos.x / PARALLAX_FACTOR) % 128.0 + x as f32 * 128.0,
                    wvdc - (resources.camera_pos.y / PARALLAX_FACTOR) % 128.0 + y as f32 * 128.0,
                    WHITE,
                );
            }
        }

        // draw the basic graphics
        gl_use_default_material();
        set_camera(&get_camera_for_target(
            &self.draw_target,
            resources.camera_pos,
            Origin::TopLeft,
        ));
        draw(&mut world, resources, assets);

        // draw explosions onto an offscreen texture
        set_camera(&get_camera_for_target(
            &self.vis_targets[0],
            resources.camera_pos,
            Origin::TopLeft,
        ));

        draw_vfx(&world);
        // now draw the explosion texture back to the draw target
        gl_use_material(self.outline_material);
        set_camera(&get_camera_for_target(
            &self.draw_target,
            vec2(self.width / 2., self.height / 2.),
            Origin::BottomLeft,
        ));
        draw_texture(self.vis_targets[0].texture, 0., 0., WHITE);

        // initialise the offscreen texture for jump flood algorithm
        gl_use_material(self.jfa_init_material);
        set_camera(&get_camera_for_target(
            &self.vis_targets[0],
            resources.camera_pos,
            Origin::TopLeft,
        ));
        draw_rectangle(
            resources.camera_pos.x - self.width / 2.,
            resources.camera_pos.y - self.height / 2.,
            self.width,
            self.height,
            WHITE,
        );

        // draw black shapes from each obscurer into an offscreen texture
        gl_use_default_material();
        let e = resources.eye_pos - resources.camera_pos;
        let r = e.x.max(self.width - e.x).max(e.y).max(self.height - e.y) + 1.;
        draw_visibility(&world, resources.eye_pos, r);

        let mut current_rt = 1;
        let mut step_size = 2_u32.pow(WALL_VISION_DEPTH.log2().ceil() as u32);
        loop {
            // do the shader pass to process the visibility texture
            gl_use_material(self.jfa_step_material);
            set_camera(&get_camera_for_target(
                &self.vis_targets[current_rt],
                vec2(self.width / 2., self.height / 2.),
                Origin::BottomLeft,
            ));
            // we don't use the actual colour but we use it to encode some other info
            // easier than setting up custom shader inputs!
            let c = Color::new(step_size as f32 / 256.0, 0.0, 0.0, 0.0);
            draw_texture_ex(
                self.vis_targets[1 - current_rt].texture,
                0.,
                0.,
                c,
                DrawTextureParams {
                    dest_size: Some(vec2(self.width, self.height)),
                    ..Default::default()
                },
            );
            if step_size == 1 {
                break;
            }
            step_size /= 2;
            current_rt = 1 - current_rt;
        }

        // draw the visibility texture over the main texture
        gl_use_material(self.jfa_final_material);
        set_camera(&get_camera_for_target(
            &self.draw_target,
            vec2(self.width / 2., self.height / 2.),
            Origin::BottomLeft,
        ));
        let c = Color::new(WALL_VISION_DEPTH / 128.0, 0.0, 0.0, 0.0);
        draw_texture_ex(
            self.vis_targets[1 - current_rt].texture,
            0.,
            0.,
            c,
            DrawTextureParams {
                dest_size: Some(vec2(self.width, self.height)),
                ..Default::default()
            },
        );

        // draw text and ui here
        gl_use_default_material();
        set_camera(&get_camera_for_target(
            &self.draw_target,
            vec2(self.width / 2., self.height / 2.),
            Origin::TopLeft,
        ));
        let wvdc = WALL_VISION_DEPTH.ceil();
        let mut y = wvdc - resources.messages.offset as f32 + 9.0; // why is it 9?
        for m in resources.messages.iter_messages() {
            draw_text(m, wvdc, y, 16.0, WHITE);
            y += Messages::HEIGHT as f32;
        }
        for ii in 0..3 {
            let sy = if ii < hp { 0.0 } else { 16.0 };
            draw_texture_ex(
                assets.ui_sprite,
                wvdc + 16.0 * ii as f32,
                self.height - wvdc - 16.0,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(0.0, sy, 16.0, 16.0)),
                    ..Default::default()
                },
            );
        }

        for btn in all::<ScreenButtons>() {
            draw_texture_ex(
                assets.controls,
                btn.get_pos().0 as f32,
                btn.get_pos().1 as f32,
                WHITE,
                btn.get_texture_params(!(input.is_hovered(btn.get_vk()) || input.is_down(btn.get_vk()))),
            );
        }
    }

    pub fn start_transition(&mut self, typ: TransitionEffectType) {
        let ff = render_target(self.final_width as u32, self.final_height as u32);
        ff.texture.set_filter(FilterMode::Nearest);
        gl_use_default_material();
        set_camera(&get_camera_for_target(
            &ff,
            vec2(self.final_width / 2., self.final_height / 2.),
            Origin::TopLeft,
        ));
        draw_texture_ex(
            self.draw_target.texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(
                    WALL_VISION_DEPTH.ceil(),
                    WALL_VISION_DEPTH.ceil(),
                    self.final_width,
                    self.final_height,
                )),
                ..Default::default()
            },
        );
        self.transition = Some((ff, new_transition(typ)));
    }

    pub fn transition_finished(&self) -> bool {
        self.transition.is_none()
    }

    pub fn tick(&mut self) {
        if let Some((_, ref mut effect)) = self.transition {
            effect.tick();
            if effect.finished() {
                self.transition = None;
            }
        }
    }
}

pub fn load_flash_material() -> Material {
    let bs = BlendState::new(
        Equation::Add,
        BlendFactor::Value(BlendValue::SourceAlpha),
        BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
    );
    load_material(
        VERTEX_SHADER,
        FLASH_FRAGMENT_SHADER,
        MaterialParams {
            pipeline_params: PipelineParams {
                color_blend: Some(bs),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .unwrap()
}

const OUTLINE_FRAGMENT_SHADER: &str = "#version 100
precision lowp float;
varying vec2 uv;
uniform sampler2D Texture;
void main() {
    vec4 col = texture2D(Texture, uv);
    gl_FragColor = col;
    if (col.a == 0.0) {
        //vec2 size = vec2(textureSize(Texture, 0));
        vec2 size = vec2(354.0, 234.0);
        vec4 l = texture2D(Texture, uv + vec2(-1.0 / size.x, 0));
        vec4 r = texture2D(Texture, uv + vec2(1.0 / size.x, 0));
        vec4 u = texture2D(Texture, uv + vec2(0, -1.0 / size.y));
        vec4 d = texture2D(Texture, uv + vec2(0, 1.0 / size.y));
        if (l.a == 1.0 || r.a == 1.0 || u.a == 1.0 || d.a == 1.0) {
            gl_FragColor = vec4(vec3(0.0), 1.0);
        }
    }
}
";

const FLASH_FRAGMENT_SHADER: &str = "#version 100
precision lowp float;
varying vec2 uv;
uniform sampler2D Texture;
void main() {
    gl_FragColor = texture2D(Texture, uv).aaaa;
}
";

const JFA_INIT_FRAGMENT_SHADER: &str = r#"#version 100
precision lowp float;
varying vec4 color;
varying vec2 uv;
uniform sampler2D Texture;
vec4 pack(vec2 fc) {
    vec2 quot = floor(floor(fc) / 128.0);
    vec2 frac = fract(floor(fc) / 128.0);
    return vec4(frac, quot / 128.0);
}
void main() {
        gl_FragColor = pack(gl_FragCoord.xy);
}
"#;

const JFA_STEP_FRAGMENT_SHADER: &str = r#"#version 100
precision lowp float;
varying vec4 color;
varying vec2 uv;
uniform sampler2D Texture;
vec4 pack(vec2 fc) {
    vec2 quot = floor(floor(fc) / 128.0);
    vec2 frac = fract(floor(fc) / 128.0);
    return vec4(frac, quot / 128.0);
}
float round_(float v) {
    return floor(v + 0.5);
}
vec2 unpack(vec4 t) {
    return vec2(round_(t.r * 128.0 + round_(t.b * 128.0) * 128.0), round_(t.g * 128.0 + round_(t.a * 128.0) * 128.0)) + 0.5;
}
void main() {
    vec2 current_pos;
    float current_dist;
    current_pos = unpack(texture2D(Texture, uv));
    current_dist = length(gl_FragCoord.xy - current_pos);
    int r = int(color.r * 256.0);
    //vec2 size = vec2(textureSize(Texture, 0));
    vec2 size = vec2(354.0, 234.0);
    for (int dx = -1; dx <= 1; dx += 1) {
        for (int dy = -1; dy <= 1; dy += 1) {
            vec2 newFragCoord = gl_FragCoord.xy + vec2(float(dx * r), float(dy * r));
            vec2 other_pos = unpack(texture2D(Texture, clamp(newFragCoord / size, 0.0, 1.0)));
            float len = length(gl_FragCoord.xy - other_pos);
            if (len < current_dist) {
                current_dist = len;
                current_pos = other_pos;
            }
        }
    }
    gl_FragColor = pack(current_pos);
}
"#;

const JFA_FINAL_FRAGMENT_SHADER: &str = r#"#version 100
precision lowp float;
varying vec4 color;
varying vec2 uv;
uniform sampler2D Texture;
float round_(float v) {
    return floor(v + 0.5);
}
vec2 unpack(vec4 t) {
    return vec2(round_(t.r * 128.0 + round_(t.b * 128.0) * 128.0), round_(t.g * 128.0 + round_(t.a * 128.0) * 128.0)) + 0.5;
}
void main() {
    float r = color.r * 128.0;
    float len = length(gl_FragCoord.xy - unpack(texture2D(Texture, uv)));
    if (len <= r) {
        gl_FragColor = vec4(0.0);
    } else {
        gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
    }
}
"#;

const VERTEX_SHADER: &str = "#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    color = color0 / 255.0;
    uv = texcoord;
}
";

const VERTEX_SHADER_PASS_COLOR: &str = "#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    color = color0 / 255.0;
    uv = texcoord;
}
";
