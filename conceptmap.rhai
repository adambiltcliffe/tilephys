fn init() {
    set_path("door", "door");
}

fn doortrigger_enter() {
    set_motion_goto("door", 1, 0.5);
}
