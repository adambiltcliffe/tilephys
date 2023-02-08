fn init() {
    top_door.set_path(up3);
    left_door.set_path(up3);
    mini_door.set_path(up2);
    enemy_doors.set_path(up3);
    dogpile_door.set_path(dogpile_path);
    slow_lift.set_path(lift_path);
    slow_lift.set_motion(forward_cycle, 1.0);
    start_lift.set_path(up7);
    start_secret.set_path(left3);
    chasm_bottom.set_path(down3);
    chasm_bottom2.set_path(down3);
    shutters.set_path(shutter_open);
    jiggler.set_path(up2);
    jiggler.set_motion(forward_cycle, 1.0);
    drop_secret.set_path(down4);
}

fn start_lift_trigger_enter() {
    start_lift.go_to(1, 2.0);
    recall_switch.set_enabled(true);
}

fn start_lift_top_exit() {
    start_lift.go_to(0, 2.0);
}

fn recall_switch_interact() {
    start_lift.go_to(0, 2.0);
}

fn shutter_trigger_enter() {
    shutters.go_to(1, 4.0);
}

fn top_switch_interact() {
    escape_switch.set_enabled(false);
    top_door.go_to(1, 4.0);
    left_door.go_to(1, 4.0);
    enemy_doors.go_to(1, 4.0);
    context.after_frames(900, || {
        top_door.go_to(0, 4.0);
        left_door.go_to(0, 4.0);
        enemy_doors.go_to(0, 4.0);
        top_switch.set_enabled(true);
        escape_switch.set_enabled(true);
    });
}

fn escape_switch_interact() {
    left_door.go_to(1, 4.0);
}

fn md_left_interact() {
    mini_door.go_to(1, 4.0);
    md_right.set_enabled(false);
    context.after_frames(30, || {
        mini_door.go_to(0, 4.0);
        md_left.set_enabled(true);
        md_right.set_enabled(true);
    });
}

fn md_right_interact() {
    mini_door.go_to(1, 4.0);
    md_left.set_enabled(false);
    context.after_frames(30, || {
        mini_door.go_to(0, 4.0);
        md_left.set_enabled(true);
        md_right.set_enabled(true);
    });
}

fn exit_switch_interact() {
    context.win();
}

fn side_switch_interact() {
    chasm_bottom2.go_to(1, 1.0);
}

fn secret_switch_interact() {
    start_secret.go_to(1, 1.0);
}

fn dogpiler_interact() {
    dogpile_door.go_to(2, 4.0);
    chasm_bottom.go_to(1, 1.0);
}

fn drop_trigger_enter() {
    drop_secret.go_to(1, 4.0);
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
