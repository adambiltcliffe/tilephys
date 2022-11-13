use std::collections::VecDeque;

const MESSAGE_FRAMES: i32 = 60;

pub struct Messages {
    entries: VecDeque<(i32, String)>,
    pub offset: u32,
}

impl Messages {
    pub const HEIGHT: u32 = 12;

    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            offset: 0,
        }
    }

    pub fn add(&mut self, message: String) {
        self.entries.push_back((MESSAGE_FRAMES, message))
    }

    pub fn update(&mut self) {
        self.entries.iter_mut().for_each(|t| t.0 -= 1);
        if self.entries.front().map_or(false, |t| t.0 <= 0) {
            self.offset += 2;
        }
        if self.offset >= Self::HEIGHT {
            self.offset -= Self::HEIGHT;
            self.entries.pop_front();
        }
    }

    pub fn iter_messages(&self) -> impl Iterator<Item = &String> + '_ {
        self.entries.iter().map(|t| &t.1)
    }
}
