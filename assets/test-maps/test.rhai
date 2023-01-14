fn init() {
    leftright.set_path(right6);
    updown.set_path(up10);
    dropstairs.set_path(down5);

    leftright.set_motion(forward_cycle, 1.0);
    updown.set_motion(forward_cycle, 1.0);

    moveable.set_path(moverpath);
    right_control.set_enabled(false);

    crusher.set_path(crush);
    dogpen.set_path(down5);
}

fn stair_switch_interact() {
    dropstairs.go_to(1, 1.0);
}

fn left_control_interact() {
    moveable.go_to(1, 1.0);
    right_control.set_enabled(true);
}

fn right_control_interact() {
    moveable.go_to(0, 1.0);
    left_control.set_enabled(true);
}

fn crusher_switch_interact() {
    dogpen.go_to(1, 1.0);
    crusher.set_motion(forward_cycle, 2.0);
    crusher_stop.set_enabled(true);
}

fn crusher_stop_interact() {
    crusher.go_to(0, 0.5);
    crusher_switch.set_enabled(true);
}
