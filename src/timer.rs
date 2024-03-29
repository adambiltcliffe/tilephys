use macroquad::time::get_time;

const FPS: f64 = 30.0;

pub struct Timer {
    accumulator: f64,
    last_frame_time: f64,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            accumulator: 0.0,
            last_frame_time: get_time(),
        }
    }
    pub fn get_num_updates(&mut self) -> u32 {
        let new_time = get_time();

        let mut frames = (new_time - self.last_frame_time) * FPS;

        if (frames - 0.25).abs() < 0.006 {
            frames = 0.25;
        } else if (frames - 0.5).abs() < 0.006 {
            frames = 0.5;
        } else if (frames - 1.0).abs() < 0.006 {
            frames = 1.0;
        }

        frames = frames.min(5.0);

        self.last_frame_time = new_time;
        self.accumulator += frames;
        self.accumulator = self.accumulator.min(5.0);

        let result = self.accumulator.floor();
        self.accumulator -= result;
        result as u32
    }
}
