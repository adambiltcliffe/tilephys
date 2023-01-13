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
    doodler.set_motion(forward_cycle, 0.5);
    set_path("platform", "platform");
    platform.set_motion(forward_cycle, 1.0);
    set_path("piston", "piston");
    piston.set_motion(forward_cycle, 0.5);
}

fn door_switch_interact() {
    door.go_to(1, 2.0);
}

fn hatch_trigger_enter() {
    hatchleft.go_to(2, 4.0);
    hatchright.go_to(2, 4.0);
}

fn unlock_switch_interact() {
    finaldoor.go_to(1, 1.0);
    towerhatch.go_to(1, 4.0);
}

fn releaser_trigger_enter() {
    releaser.go_to(1, 4.0);
    releaser2.go_to(1, 4.0);
    secretdoors.go_to(1, 4.0);
    secretplat.go_to(1, 0.5);
}

fn compartment_enter() {
    falsewall.go_to(1, 4.0);
}

fn final_switch_interact() {
    flags.win();
}
