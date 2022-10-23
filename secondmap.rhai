fn init() {
    set_path("movingblock", "zigzag");
    set_path("hatch", "hatch_drop");
}

fn trigger1_enter() {
    set_motion_goto("movingblock", 6, 6.0);
}

fn trigger2_enter() {
    set_motion_goto("movingblock", 0, 6.0);
    set_motion_goto("hatch", 1, 3.0);
}
