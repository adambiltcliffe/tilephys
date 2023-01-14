fn init() {
    door.set_path(door_path);
}

fn left_switch_interact() {
    door.go_to(1, 4.0);
    right_switch.set_enabled(false);
    context.after_frames(30, || {
        door.go_to(0, 4.0);
        left_switch.set_enabled(true);
        right_switch.set_enabled(true);
    });
}

fn right_switch_interact() {
    door.go_to(1, 4.0);
    left_switch.set_enabled(false);
    context.after_frames(30, || {
        door.go_to(0, 4.0);
        left_switch.set_enabled(true);
        right_switch.set_enabled(true);
    });
}
