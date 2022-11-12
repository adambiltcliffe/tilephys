use crate::draw::draw;
use crate::loader::TilesetInfo;
use crate::resources::Resources;
use crate::visibility::draw_visibility;
use hecs::Entity;
use macroquad::prelude::*;

const WALL_VISION_DEPTH: f32 = 16.5;

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
    width: f32,
    height: f32,
    final_width: f32,
    final_height: f32,
    draw_target: RenderTarget,
    vis_targets: [RenderTarget; 2],
    jfa_init_material: Material,
    jfa_step_material: Material,
    jfa_final_material: Material,
}

impl Renderer {
    pub fn new(final_width: u32, final_height: u32) -> Self {
        let margin = WALL_VISION_DEPTH.ceil() as u32 * 2;
        let width = final_width + margin;
        let height = final_height + margin;
        println!("{} {}", width, height);
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
        let draw_target = render_target(width, height);
        draw_target.texture.set_filter(FilterMode::Nearest);
        Self {
            width: width as f32,
            height: height as f32,
            final_width: final_width as f32,
            final_height: final_height as f32,
            draw_target,
            vis_targets: [render_target(width, height), render_target(width, height)],
            jfa_init_material,
            jfa_step_material,
            jfa_final_material,
        }
    }

    pub(crate) fn draw(
        &self,
        world: &mut hecs::World,
        eye: Vec2,
        cam: Vec2,
        tsi: &TilesetInfo,
        resources: &Resources,
        draw_order: &Vec<Entity>,
        fps: u32,
    ) {
        // draw the basic graphics
        gl_use_default_material();
        set_camera(&get_camera_for_target(
            &self.draw_target,
            cam,
            Origin::TopLeft,
        ));
        draw(world, tsi, resources, draw_order);

        // initialise the offscreen texture for jump flood algorithm
        gl_use_material(self.jfa_init_material);
        set_camera(&get_camera_for_target(
            &self.vis_targets[0],
            cam,
            Origin::TopLeft,
        ));
        draw_rectangle(
            cam.x - self.width / 2.,
            cam.y - self.height / 2.,
            self.width,
            self.height,
            WHITE,
        );

        // draw black shapes from each obscurer into an offscreen texture
        gl_use_default_material();
        let e = eye - cam;
        let r = e.x.max(self.width - e.x).max(e.y).max(self.height - e.y) + 1.;
        draw_visibility(&world, eye, r);

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

        // draw text here (it's not affected by visibility but does need scaling)
        gl_use_default_material();
        let (font_size, font_scale, font_scale_aspect) = camera_font_scale(16.0);
        draw_text_ex(
            &format!("FPS: {}", fps),
            WALL_VISION_DEPTH.ceil(),
            self.height - WALL_VISION_DEPTH.ceil() - 8.0,
            TextParams {
                font_size,
                font_scale: -font_scale,
                font_scale_aspect: -font_scale_aspect,
                ..Default::default()
            },
        );

        // finally draw to the screen
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
}

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
    vec2 size = vec2(textureSize(Texture, 0));
    //vec2 size = vec2(354.0, 234.0);
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
