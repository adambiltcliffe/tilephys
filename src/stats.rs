use crate::level::LevelInfo;

#[derive(Clone)]
pub struct LevelStats {
    pub info: LevelInfo,
    pub frames: u32,
    pub kills: u32,
    pub max_kills: u32,
    pub items: u32,
    pub max_items: u32,
    pub secrets: u32,
    pub max_secrets: u32,
}

impl LevelStats {
    pub fn new(info: LevelInfo, max_kills: u32, max_items: u32, max_secrets: u32) -> Self {
        Self {
            info,
            frames: 0,
            kills: 0,
            max_kills,
            items: 0,
            max_items,
            secrets: 0,
            max_secrets,
        }
    }

    pub fn pretty_time(&self) -> String {
        let m = self.frames / (30 * 60);
        let s = (self.frames % (30 * 60)) as f32 / 30.0;
        format_args!("{:02}:{:05.2}", m, s).to_string()
    }
}
