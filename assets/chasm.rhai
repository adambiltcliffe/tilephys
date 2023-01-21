fn init() {
    top_door.set_path(up3);
    slow_lift.set_path(lift_path);
    slow_lift.set_motion(forward_cycle, 1.0);
    start_lift.set_path(up7);
    chasm_bottom.set_path(down3);
    chasm_bottom2.set_path(down3);
    shutters.set_path(shutter_open);
}

fn start_lift_trigger_enter() {
    start_lift.go_to(1, 2.0);
}

fn start_lift_top_exit() {
    start_lift.go_to(0, 2.0);
}

fn shutter_trigger_enter() {
    shutters.go_to(1, 4.0);
}

fn top_switch_interact() {
    top_door.go_to(1, 4.0);
}

fn exit_switch_interact() {
    context.win();
}

fn side_switch_interact() {
    chasm_bottom2.go_to(1, 1.0);
}

fn temp_switch_interact() {
    chasm_bottom.go_to(1, 1.0);
}
