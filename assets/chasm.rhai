fn init() {
    top_door.set_path(up3);
    slow_lift.set_path(lift_path);
    slow_lift.set_motion(forward_cycle, 1.0);
    start_lift.set_path(up7);
    start_lift.go_to(1, 2.0);
}

fn top_switch_interact() {
    top_door.go_to(1, 4.0);
}

fn exit_switch_interact() {
    context.win();
}
