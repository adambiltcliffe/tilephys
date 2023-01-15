use crate::loader::load_level;
use crate::stats::LevelNumber;
use crate::{resources::SceneResources, stats::LevelStats};
use macroquad::experimental::coroutines::{start_coroutine, Coroutine};

// eventually there will be variants whose names don't end in "...Level"
#[allow(clippy::enum_variant_names)]
pub enum Scene {
    PreLevel(LevelNumber, Coroutine<Result<Scene, String>>, bool),
    PlayLevel(SceneResources),
    PostLevel(LevelStats),
}

pub async fn new_prelevel(n: LevelNumber, name: String, fast: bool) -> Scene {
    let coro: Coroutine<Result<Scene, String>> = start_coroutine(load_level(n, name));
    if coro.is_done() {
        let res = coro.retrieve();
        assert!(res.is_some());
        assert!(res.unwrap().is_ok());
    }
    Scene::PreLevel(n, coro, fast)
}
