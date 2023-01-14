fn init() {
    door.set_path(door_path);
    finaldoor.set_path(door_path);
    hatchleft.set_path(hatchleft_path);
    hatchright.set_path(hatchright_path);
    towerhatch.set_path(right3);
    releaser.set_path(releaser_path);
    releaser2.set_path(right3);
    secretdoors.set_path(door_path);
    secretplat.set_path(right3);
    falsewall.set_path(drop3);

    doodler.set_path(doodle);
    doodler.set_motion(forward_cycle, 0.5);
    platform.set_path(platform_circuit);
    platform.set_motion(forward_cycle, 1.0);
    piston.set_path(piston_path);
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
    context.win();
}
