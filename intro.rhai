fn init() {
    set_path("door", "door");
    set_path("finaldoor", "door");
    set_path("hatchleft", "hatchleft");
    set_path("hatchright", "hatchright");
    set_path("towerhatch", "right2");
}

fn door_trigger_enter() {
    set_motion_goto("door", 1, 0.5);
}

fn hatch_trigger_enter() {
    set_motion_goto("hatchleft", 2, 4.0);
    set_motion_goto("hatchright", 2, 4.0);
}

fn unlock_trigger_enter() {
    set_motion_goto("finaldoor", 1, 0.5);
    set_motion_goto("towerhatch", 1, 4.0);
}
