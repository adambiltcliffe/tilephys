pub enum Scene {
    Play,
}

#[derive(PartialEq, Eq)]
pub enum SceneTransition {
    None,
    Restart,
}
