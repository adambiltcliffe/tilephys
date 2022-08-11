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
    // eventually, should not be pub
    render_target: RenderTarget,
    material: Material,
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> Self {
        use miniquad::graphics::{BlendFactor, BlendState, BlendValue, Equation};
        let bs = BlendState::new(
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        );
        let material = load_material(
            VERTEX_SHADER,
            FRAGMENT_SHADER,
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
            render_target: render_target(width, height),
            material,
        }
    }

    pub fn draw(&self, world: &mut hecs::World, eye: Vec2) {
        gl_use_default_material();
        set_camera(&get_screen_camera(self.width as f32, self.height as f32));
        draw(world);

        set_camera(&get_camera_for_target(&self.render_target));
        let r = eye
            .x
            .max(self.width as f32 - eye.x)
            .max(eye.y)
            .max(self.height as f32 - eye.y)
            + 1.;
        draw_visibility(&world, eye, r);

        set_camera(&get_screen_camera(self.width as f32, self.height as f32));
        gl_use_material(self.material);
        draw_texture_ex(
            self.render_target.texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(self.width as f32, self.height as f32)),
                ..Default::default()
            },
        );
        gl_use_default_material();
    }
}

const FRAGMENT_SHADER: &'static str = r#"#version 100
precision lowp float;
varying vec4 color;
varying vec2 uv;

uniform sampler2D Texture;

void main() {
    vec3 res = texture2D(Texture, uv).rgb * color.rgb;
    gl_FragColor = vec4(res, 0.5);
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
