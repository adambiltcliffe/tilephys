#[cfg(debug_assertions)]
use crate::console::{ConsoleEntryType, CONSOLE};

#[cfg(debug_assertions)]
pub fn info(msg: &str) {
    CONSOLE
        .lock()
        .unwrap()
        .add(msg.to_string(), ConsoleEntryType::Info);
}

#[cfg(debug_assertions)]
pub fn warn(msg: &str) {
    let mut c = CONSOLE.lock().unwrap();
    c.add(msg.to_string(), ConsoleEntryType::Warning);
    c.force_visible();
    println!("{}", msg);
}

#[cfg(not(debug_assertions))]
pub fn info(_msg: &str) {}

#[cfg(not(debug_assertions))]
pub fn warn(msg: &str) {
    println!("{}", msg);
}
