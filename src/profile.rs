use enum_map::{Enum, EnumMap};

#[cfg(debug_assertions)]
use macroquad::color::Color;
#[cfg(debug_assertions)]
use macroquad::time::get_time;
#[cfg(debug_assertions)]
use std::collections::VecDeque;

#[cfg(debug_assertions)]
const WINDOW: usize = 60;

#[cfg(debug_assertions)]
const X_POS: f32 = 80.0;

#[cfg(debug_assertions)]
const PHASES: [(Phase, &'static str, Color); 8] = [
    (Phase::Motion, "motion", macroquad::color::RED),
    (Phase::Pickups, "pickups", macroquad::color::ORANGE),
    (Phase::Player, "player", macroquad::color::YELLOW),
    (Phase::Enemies, "enemies", macroquad::color::GREEN),
    (Phase::Actor, "actor", macroquad::color::SKYBLUE),
    (Phase::Projectile, "projectile", macroquad::color::BLUE),
    (Phase::Vfx, "vfx", macroquad::color::PURPLE),
    (Phase::Render, "render", macroquad::color::PINK),
];

#[derive(Copy, Clone, Enum)]
pub enum Phase {
    Motion,
    Pickups,
    Player,
    Enemies,
    Actor,
    Projectile,
    Vfx,
    Render,
}

#[cfg(debug_assertions)]
pub struct Profiler {
    times: EnumMap<Phase, VecDeque<f64>>,
    start: f64,
    phase: Option<Phase>,
}

#[cfg(debug_assertions)]
impl Profiler {
    pub fn new() -> Self {
        Self {
            times: EnumMap::default(),
            start: 0.0,
            phase: None,
        }
    }
    pub fn start(&mut self, new_phase: Phase) {
        self.stop();
        self.phase = Some(new_phase);
        self.start = get_time();
    }
    pub fn stop(&mut self) {
        if let Some(old_phase) = self.phase {
            let times = &mut self.times[old_phase];
            times.push_back(get_time() - self.start);
            if times.len() > WINDOW {
                times.pop_front();
            }
        }
    }
    pub fn draw(&self) {
        use macroquad::camera::set_default_camera;
        use macroquad::color::WHITE;
        use macroquad::shapes::draw_line;
        use macroquad::text::draw_text;
        set_default_camera();
        let mut y = 9.0; // why is it 9?
        for (p, pname, c) in PHASES {
            draw_text(&format!("{}", pname), 0.0, y, 16.0, c);
            self.draw_box(y, p, c);
            y += 12.0;
        }
        draw_line(X_POS, 0.0, X_POS, 96.0, 1.0, WHITE);
    }
    fn draw_box(&self, y: f32, phase: Phase, c: Color) {
        use macroquad::shapes::{draw_line, draw_rectangle_lines};
        let mut times_us: Vec<i32> = self.times[phase]
            .iter()
            .map(|t| (*t * 1000000.) as i32)
            .collect();
        times_us.sort();
        if times_us.len() > 0 {
            let n = times_us.len();
            let qs = [
                times_us[0],
                times_us[(n - 1) / 4],
                times_us[(n - 1) / 2],
                times_us[(n - 1) * 3 / 4],
                times_us[n - 1],
            ];
            let xs = qs.map(|t| t as f32 / 10.0);
            draw_line(X_POS + xs[0], y - 1.0, X_POS + xs[0], y - 7.0, 1.0, c);
            draw_line(X_POS + xs[2], y - 1.0, X_POS + xs[2], y - 7.0, 1.0, c);
            draw_line(X_POS + xs[4], y - 1.0, X_POS + xs[4], y - 7.0, 1.0, c);
            draw_rectangle_lines(X_POS + xs[1], y - 8.0, xs[3] - xs[1], 8.0, 1.0, c);
            draw_line(X_POS + xs[0], y - 4.0, X_POS + xs[1], y - 4.0, 1.0, c);
            draw_line(X_POS + xs[3], y - 4.0, X_POS + xs[4], y - 4.0, 1.0, c);
        }
    }
}

#[cfg(not(debug_assertions))]
pub struct Profiler {}

#[cfg(not(debug_assertions))]
impl Profiler {
    pub fn new() -> Self {
        Self {}
    }
    pub fn start(&mut self, phase: Phase) {}
    pub fn stop(&mut self) {}
    pub fn draw(&self) {}
}
