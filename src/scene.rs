use crate::level::LevelInfo;
use crate::loader::load_level;
use crate::resources::Inventory;
use crate::{resources::SceneResources, stats::LevelStats};
use macroquad::experimental::coroutines::{start_coroutine, Coroutine};

pub enum Scene {
    Title(u8),
    PreLevel(LevelInfo, Coroutine<anyhow::Result<Scene>>, bool),
    PlayLevel(SceneResources),
    PostLevel(LevelStats, Inventory),
    Error(String),
}

pub async fn new_prelevel(info: LevelInfo, inv: Inventory, fast: bool) -> Scene {
    let coro: Coroutine<anyhow::Result<Scene>> = start_coroutine(load_level(info.clone(), inv));
    if coro.is_done() {
        let res = coro.retrieve();
        assert!(res.is_some());
        assert!(res.unwrap().is_ok());
    }
    Scene::PreLevel(info, coro, fast)
}
