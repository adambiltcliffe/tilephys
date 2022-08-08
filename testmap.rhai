fn init() {
    print("hello from rhai");
    set_constant_motion("layer2", 1, 0);
    set_path("cross", "diamondpath");
    set_motion("cross", ForwardCycle, 3.0);
    set_path("slider","slidertrack");
    set_path("lift", "lift_path");
}

fn testtrigger_enter() {
    print("here we go here we go");
    set_motion_goto("slider", 3, 6.0);
    set_motion("lift", ForwardCycle, 3.5);
}

fn testtrigger2_enter() {
    print("re-e-wind");
    set_motion_goto("slider", 0, 6.0);
    set_motion("lift", Static, 3.5);
}

fn trap_trigger_enter() {
    set_path("trap_floor", "trap_floor_path");
    set_motion("trap_floor", ForwardOnce, 6.0);
    set_path("trap_hatch", "trap_hatch_path");
    set_motion("trap_hatch", ForwardOnce, 2.0);
}
