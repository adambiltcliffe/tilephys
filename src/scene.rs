use hecs::World;
use std::cell::RefCell;
use std::rc::Rc;

pub enum Scene {
    PreLevel,
    PlayLevel(Rc<RefCell<World>>),
    PostLevel,
}

#[derive(PartialEq, Eq)]
pub enum NewScene {
    PreLevel,
    PlayLevel,
    PostLevel,
}
