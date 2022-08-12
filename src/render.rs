use crate::draw::draw;
use crate::visibility::draw_visibility;
use macroquad::prelude::*;

fn get_screen_camera(width: f32, height: f32) -> Camera2D {
    Camera2D {
        zoom: (vec2(2. / width, -2. / height)),
        target: vec2(width / 2., height / 2.),
        ..Default::default()
    }
}

fn get_camera_for_target(target: &RenderTarget) -> Camera2D {
    let width = target.texture.width() as f32;
    let height = target.texture.height() as f32;
    Camera2D {
        render_target: Some(*target),
        zoom: (vec2(2. / width, 2. / height)),
        target: vec2(width / 2., height / 2.),
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
            VERTEX_SHADER,
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

    pub fn draw(&self, world: &mut hecs::World, eye: Vec2) {
        // draw the basic graphics
        gl_use_default_material();
        set_camera(&get_screen_camera(self.width as f32, self.height as f32));
        draw(world);

        // initialise the offscreen texture for jump flood algorithm
        gl_use_material(self.jfa_init_material);
        set_camera(&get_camera_for_target(&self.render_targets[0]));
        draw_rectangle(0., 0., self.width as f32, self.height as f32, PINK);

        // draw black shapes from each obscurer into an offscreen texture
        gl_use_default_material();
        let r = eye
            .x
            .max(self.width as f32 - eye.x)
            .max(eye.y)
            .max(self.height as f32 - eye.y)
            + 1.;
        draw_visibility(&world, eye, r);

        // do the shader pass to process the visibility texture
        gl_use_material(self.jfa_step_material);
        set_camera(&get_camera_for_target(&self.render_targets[1]));
        // we don't use the actual colour but we use it to encode some other info
        let c = Color::new(2.0 / self.width as f32, 2.0 / self.height as f32, 0.0, 0.0);
        draw_texture_ex(
            self.render_targets[0].texture,
            0.,
            0.,
            c,
            DrawTextureParams {
                dest_size: Some(vec2(self.width as f32, self.height as f32)),
                ..Default::default()
            },
        );

        // draw the visibility texture over the main texture
        set_camera(&get_screen_camera(self.width as f32, self.height as f32));
        gl_use_material(self.jfa_final_material);
        draw_texture_ex(
            self.render_targets[1].texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(self.width as f32, self.height as f32)),
                ..Default::default()
            },
        );
    }
}

/*const FRAGMENT_SHADER: &'static str = r#"#version 100
precision lowp float;
varying vec4 color;
varying vec2 uv;

uniform sampler2D Texture;

void main() {
    vec3 res = texture2D(Texture, uv).rgb * color.rgb;
    gl_FragColor = vec4(res, 0.5);
}
"#;*/

const JFA_INIT_FRAGMENT_SHADER: &'static str = r#"#version 100
precision lowp float;
varying vec4 color;
varying vec2 uv;

uniform sampler2D Texture;

void main() {
    gl_FragColor = vec4(uv.x, uv.y, 0.0, 0.0);
}
"#;

const JFA_STEP_FRAGMENT_SHADER: &'static str = r#"#version 100
precision lowp float;
varying vec4 color;
varying vec2 uv;

uniform sampler2D Texture;

void main() {
    vec4 res = texture2D(Texture, uv).rgba;
    vec2 size = vec2(textureSize(Texture, 0));
    for (int dx = -3; dx < 4; dx += 1) {
        for (int dy = -3; dy < 4; dy += 1) {
            vec2 newFragCoord = gl_FragCoord.xy + vec2(float(dx), float(dy));
            vec2 newuv = newFragCoord / size;
            if (texture2D(Texture, newuv).a == 0.0) {
                res.a = 0.0;
            }
        }
    }
    gl_FragColor = res;
}
"#;

const JFA_FINAL_FRAGMENT_SHADER: &'static str = r#"#version 100
precision lowp float;
varying vec4 color;
varying vec2 uv;

uniform sampler2D Texture;

void main() {
    // remove the 0.5 on next line once we've got it working
    float level = texture2D(Texture, uv).a * 0.5;
    gl_FragColor = vec4(0.0, 0.0, 0.0, level);
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
