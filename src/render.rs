use crate::draw::draw;
use crate::loader::TilesetInfo;
use crate::visibility::draw_visibility;
use macroquad::prelude::*;

const WALL_VISION_DEPTH: f32 = 8.5;

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

fn get_screen_camera(width: f32, height: f32, camera: Vec2, o: Origin) -> Camera2D {
    Camera2D {
        zoom: (vec2(2. / width, zoom_coeff(o) / height)),
        target: camera,
        ..Default::default()
    }
}

fn get_camera_for_target(target: &RenderTarget, camera: Vec2, o: Origin) -> Camera2D {
    let width = target.texture.width() as f32;
    let height = target.texture.height() as f32;
    Camera2D {
        render_target: Some(*target),
        zoom: (vec2(2. / width, zoom_coeff(o) / height)),
        target: camera,
        ..Default::default()
    }
}

pub struct Renderer {
    width: u32,
    height: u32,
    render_targets: [RenderTarget; 2],
    jfa_init_material: Material,
    jfa_step_material: Material,
    jfa_final_material: Material,
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> Self {
        use miniquad::graphics::{BlendFactor, BlendState, BlendValue, Equation};
        let bs = BlendState::new(
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        );
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
        Self {
            width,
            height,
            render_targets: [render_target(width, height), render_target(width, height)],
            jfa_init_material,
            jfa_step_material,
            jfa_final_material,
        }
    }

    pub(crate) fn draw(&self, world: &mut hecs::World, eye: Vec2, cam: Vec2, tsi: &TilesetInfo) {
        // draw the basic graphics
        gl_use_default_material();
        set_camera(&get_screen_camera(
            self.width as f32,
            self.height as f32,
            cam,
            Origin::TopLeft,
        ));
        draw(world, tsi);

        // initialise the offscreen texture for jump flood algorithm
        gl_use_material(self.jfa_init_material);
        set_camera(&get_camera_for_target(
            &self.render_targets[0],
            cam,
            Origin::TopLeft,
        ));
        draw_rectangle(
            cam.x - self.width as f32 / 2.,
            cam.y - self.height as f32 / 2.,
            self.width as f32,
            self.height as f32,
            WHITE,
        );

        // draw black shapes from each obscurer into an offscreen texture
        gl_use_default_material();
        let e = eye - cam;
        let r =
            e.x.max(self.width as f32 - e.x)
                .max(e.y)
                .max(self.height as f32 - e.y)
                + 1.;
        draw_visibility(&world, eye, r);

        let mut current_rt = 1;
        let mut step_size = 2_u32.pow(WALL_VISION_DEPTH.log2().ceil() as u32);
        loop {
            // do the shader pass to process the visibility texture
            gl_use_material(self.jfa_step_material);
            set_camera(&get_camera_for_target(
                &self.render_targets[current_rt],
                vec2(self.width as f32 / 2., self.height as f32 / 2.),
                Origin::BottomLeft,
            ));
            // we don't use the actual colour but we use it to encode some other info
            // easier than setting up custom shader inputs!
            let c = Color::new(step_size as f32 / 256.0, 0.0, 0.0, 0.0);
            draw_texture_ex(
                self.render_targets[1 - current_rt].texture,
                0.,
                0.,
                c,
                DrawTextureParams {
                    dest_size: Some(vec2(self.width as f32, self.height as f32)),
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
        set_camera(&get_screen_camera(
            self.width as f32,
            self.height as f32,
            vec2(self.width as f32 / 2., self.height as f32 / 2.),
            Origin::BottomLeft,
        ));
        let c = Color::new(WALL_VISION_DEPTH / 128.0, 0.0, 0.0, 0.0);
        draw_texture_ex(
            self.render_targets[1 - current_rt].texture,
            0.,
            0.,
            c,
            DrawTextureParams {
                dest_size: Some(vec2(self.width as f32, self.height as f32)),
                ..Default::default()
            },
        );
    }
}

const JFA_INIT_FRAGMENT_SHADER: &'static str = r#"#version 100
precision lowp float;
varying vec4 color;
varying vec2 uv;
uniform sampler2D Texture;
vec4 pack(vec2 fc) {
    vec2 quot;
    vec2 frac = modf(floor(fc) / 128.0, quot);
    return vec4(frac, quot / 128.0);
}
void main() {
        gl_FragColor = pack(gl_FragCoord.xy);
}
"#;

const JFA_STEP_FRAGMENT_SHADER: &'static str = r#"#version 100
precision lowp float;
varying vec4 color;
varying vec2 uv;
uniform sampler2D Texture;
vec4 pack(vec2 fc) {
    vec2 quot;
    vec2 frac = modf(floor(fc) / 128.0, quot);
    return vec4(frac, quot / 128.0);
}
vec2 unpack(vec4 t) {
    return vec2(round(t.r * 128.0 + round(t.b * 128.0) * 128.0), round(t.g * 128.0 + round(t.a * 128.0) * 128.0)) + 0.5;
}
void main() {
    vec2 current_pos;
    float current_dist;
    current_pos = unpack(texture2D(Texture, uv));
    current_dist = length(gl_FragCoord.xy - current_pos);
    int r = int(color.r * 256.0);
    vec2 size = vec2(textureSize(Texture, 0));
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

const JFA_FINAL_FRAGMENT_SHADER: &'static str = r#"#version 100
precision lowp float;
varying vec4 color;
varying vec2 uv;
uniform sampler2D Texture;
vec2 unpack(vec4 t) {
    return vec2(round(t.r * 128.0 + round(t.b * 128.0) * 128.0), round(t.g * 128.0 + round(t.a * 128.0) * 128.0)) + 0.5;
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

const VERTEX_SHADER: &'static str = "#version 100
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

const VERTEX_SHADER_PASS_COLOR: &'static str = "#version 100
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
