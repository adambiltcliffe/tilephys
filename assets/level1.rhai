fn init() {
    set_path("door", "door");
    set_path("door2", "door");
    set_path("door3", "door");
    set_path("finaldoor", "door");
    set_path("hatchleft", "hatchleft");
    set_path("hatchright", "hatchright");
    set_path("door2", "door");
    set_path("lift", "lift");

}

fn door_interact() {
    set_motion_goto("door", 1, 1.0)
}

fn door2_interact() {
    set_motion_goto("door2", 1, 1.0)
}

fn door3_interact() {
    set_motion_goto("door3", 1, 1.0);
}

fn hatch_trigger_enter() {
    set_motion_goto("hatchleft", 2, 4.0);
    set_motion_goto("hatchright", 2, 4.0);
    set_motion_goto("door2", 1, 4.0);
}

fn lift_trigger_enter() {
    set_motion("lift", ForwardCycle, 0.5);
}

fn final_switch_interact() {
    win();
}
