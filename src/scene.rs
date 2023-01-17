use crate::loader::load_level;
use crate::{resources::SceneResources, stats::LevelStats};
use macroquad::experimental::coroutines::{start_coroutine, Coroutine};

pub enum Scene {
    PreLevel(Coroutine<Result<Scene, String>>, bool),
    PlayLevel(SceneResources),
    PostLevel(LevelStats),
}

pub async fn new_prelevel(name: String, fast: bool) -> Scene {
    let coro: Coroutine<Result<Scene, String>> = start_coroutine(load_level(name));
    //println!("was it done at the beginning? {}", coro.is_done());
    if coro.is_done() {
        let res = coro.retrieve();
        assert!(res.is_some());
        assert!(res.unwrap().is_ok());
    }
    //let d = format!("{:?}", coro).to_string();
    //println!("id is {:?}", d);
    Scene::PreLevel(coro, fast)
}
