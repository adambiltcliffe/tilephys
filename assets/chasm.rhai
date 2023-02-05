fn init() {
    top_door.set_path(up3);
    enemy_doors.set_path(up3);
    dogpile_door.set_path(dogpile_path);
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
    enemy_doors.go_to(1, 4.0);
    context.after_frames(900, || {
        top_door.go_to(0, 4.0);
        enemy_doors.go_to(0, 4.0);
        top_switch.set_enabled(true);
    });
}

fn exit_switch_interact() {
    context.win();
}

fn side_switch_interact() {
    chasm_bottom2.go_to(1, 1.0);
}

fn dogpiler_interact() {
    dogpile_door.go_to(2, 4.0);
    chasm_bottom.go_to(1, 1.0);
}

fn tutorial_cycle_enter() {
    context.popup("Press A and S to cycle weapons.");
}

fn tutorial_backup1_enter() {
    context.popup("Backup laser is restored if ammo runs out.");
}

fn tutorial_backup2_enter() {
    context.popup("Backup laser is restored if ammo runs out.");
}
