use crate::{resources::SceneResources, stats::LevelStats};

pub enum Scene {
    PreLevel,
    PlayLevel(SceneResources),
    PostLevel(LevelStats),
}

pub enum NewScene {
    PreLevel,
    PlayLevel,
    PostLevel(LevelStats),
}
