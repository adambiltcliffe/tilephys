fn init() {
    set_path("leftright", "right6");
    set_path("updown", "up10");
    set_path("dropstairs", "down5");

    set_motion("leftright", ForwardCycle, 1.0);
    set_motion("updown", ForwardCycle, 1.0);
}

fn stair_switch_interact() {
    set_motion_goto("dropstairs", 1, 1.0);
}
