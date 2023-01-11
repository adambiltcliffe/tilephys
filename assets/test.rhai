fn init() {
    set_path("leftright", "right6");
    set_path("updown", "up10");
    set_path("dropstairs", "down5");

    set_motion("leftright", ForwardCycle, 1.0);
    set_motion("updown", ForwardCycle, 1.0);

    set_path("moveable", "moverpath");
    set_switch_enabled("right_control", false);

    set_path("crusher", "crush");
    set_path("dogpen", "down5");
}

fn stair_switch_interact() {
    set_motion_goto("dropstairs", 1, 1.0);
}

fn left_control_interact() {
    set_motion_goto("moveable", 1, 1.0);
    set_switch_enabled("right_control", true);
}

fn right_control_interact() {
    set_motion_goto("moveable", 0, 1.0);
    set_switch_enabled("left_control", true);
}

fn crusher_switch_interact() {
    set_motion_goto("dogpen", 1, 1.0);
    set_motion("crusher", ForwardCycle, 2.0);
}
