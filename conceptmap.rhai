fn init() {
    set_path("door", "door");
    set_path("dropper", "dropper");
    set_path("boulder", "boulder");
    set_path("door2", "smalldoors");
    set_path("door3", "smalldoors");
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

fn door2trigger_enter() {
    set_motion_goto("door2", 1, 2.0);
}

fn door3trigger_enter() {
    set_motion_goto("door3", 1, 2.0);
}
