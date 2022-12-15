fn init() {
    set_path("door", "door");
    set_path("finaldoor", "door");
    set_path("hatchleft", "hatchleft");
    set_path("hatchright", "hatchright");
    set_path("towerhatch", "right3");
    set_path("releaser", "releaser");
    set_path("releaser2", "right3");
    set_path("secretdoors", "door");
    set_path("secretplat", "right3");
    set_path("falsewall", "drop3");

    set_path("doodler", "doodler");
    set_motion("doodler", ForwardCycle, 0.5);
    set_path("platform", "platform");
    set_motion("platform", ForwardCycle, 1.0);
    set_path("piston", "piston");
    set_motion("piston", ForwardCycle, 0.5);
}

fn door_switch_interact() {
    set_motion_goto("door", 1, 2.0);
}

fn hatch_trigger_enter() {
    set_motion_goto("hatchleft", 2, 4.0);
    set_motion_goto("hatchright", 2, 4.0);
}

fn unlock_switch_interact() {
    set_motion_goto("finaldoor", 1, 1.0);
    set_motion_goto("towerhatch", 1, 4.0);
}

fn releaser_trigger_enter() {
    set_motion_goto("releaser", 1, 4.0);
    set_motion_goto("releaser2", 1, 4.0);
    set_motion_goto("secretdoors", 1, 4.0);
    set_motion_goto("secretplat", 1, 0.5);
}

fn compartment_enter() {
    set_motion_goto("falsewall", 1, 4.0);
}

fn finish_enter() {
    win();
}

fn final_switch_interact() {
    win();
}
