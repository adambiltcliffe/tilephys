fn init() {
    set_path("leftright", "right6");
    set_path("updown", "up10");
    set_path("dropstairs", "down5");

    leftright.set_motion(forward_cycle, 1.0);
    updown.set_motion(forward_cycle, 1.0);

    set_path("moveable", "moverpath");
    right_control.set_enabled(false);

    set_path("crusher", "crush");
    set_path("dogpen", "down5");
}

fn stair_switch_interact() {
    set_motion_goto("dropstairs", 1, 1.0);
}

fn left_control_interact() {
    set_motion_goto("moveable", 1, 1.0);
    right_control.set_enabled(true);
}

fn right_control_interact() {
    set_motion_goto("moveable", 0, 1.0);
    left_control.set_enabled(true);
}

fn crusher_switch_interact() {
    set_motion_goto("dogpen", 1, 1.0);
    crusher.set_motion(forward_cycle, 2.0);
    crusher_stop.set_enabled(true);
}

fn crusher_stop_interact() {
    set_motion_goto("crusher", 0, 0.5);
    crusher_switch.set_enabled(true);
}
