use crate::level::LevelInfo;
use crate::loader::load_level;
use crate::{resources::SceneResources, stats::LevelStats};
use macroquad::experimental::coroutines::{start_coroutine, Coroutine};

// eventually there will be variants whose names don't end in "...Level"
#[allow(clippy::enum_variant_names)]
pub enum Scene {
    PreLevel(LevelInfo, Coroutine<Result<Scene, String>>, bool),
    PlayLevel(SceneResources),
    PostLevel(LevelStats),
}

pub async fn new_prelevel(info: LevelInfo, fast: bool) -> Scene {
    let coro: Coroutine<Result<Scene, String>> = start_coroutine(load_level(info.clone()));
    if coro.is_done() {
        let res = coro.retrieve();
        assert!(res.is_some());
        assert!(res.unwrap().is_ok());
    }
    Scene::PreLevel(info, coro, fast)
}
