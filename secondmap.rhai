fn init() {
    set_path("movingblock", "zigzag");
}

fn trigger1_enter() {
    set_motion_goto("movingblock", 6, 6.0);
}

fn trigger2_enter() {
    set_motion_goto("movingblock", 0, 6.0);
}
