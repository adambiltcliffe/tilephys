fn init() {
    set_path("door", "door");
    set_path("dropper", "dropper");
    set_path("boulder", "boulder");
}

fn doortrigger_enter() {
    set_motion_goto("door", 1, 0.5);
}

fn bouldertrigger_enter() {
    set_motion_goto("boulder", 2, 0.5);
}

fn droppertrigger_enter() {
    set_motion_goto("dropper", 2, 3.0);
}
