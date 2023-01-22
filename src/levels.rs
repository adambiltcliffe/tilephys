use enum_iterator::{Sequence, next};
use crate::scene::{new_prelevel, Scene};

#[derive(Sequence)]
pub enum Level {
    Intro,
    Chasm,
    Level1,
}

impl Level {
    pub fn as_map_name(&self) -> &str {
        match self {
            Self::Intro => {
                return "intro";
            },
            Self::Chasm => {
                return "chasm";
            },
            Self::Level1 => {
                return "level1";
            },
        }
    }

    pub fn as_level_name(&self) -> &str {
        match self {
            Self::Intro => {
                return "Entryway";
            },
            Self::Chasm => {
                return "Chasm"
            },
            Self::Level1 => {
                return "Run!";
            },
        }
    }

    pub fn next(&self) -> Self {
        // At some point this should be updated for a proper end game screen/quit application
        next(self).unwrap_or(Level::Intro)
    }

    pub async fn init_scene(&self, fast: bool) -> Scene {
        new_prelevel(self.as_map_name().to_string().clone(), fast).await
    }
}