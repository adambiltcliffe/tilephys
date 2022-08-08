fn init() {
    print("hello from rhai");
    set_constant_motion("layer1", -1, 0);
    set_constant_motion("layer2", 1, 0);
    set_constant_motion("layer3", 0, -1);
    set_path("layer4", "orbit");
    set_motion("layer4", ForwardCycle, 1.0);
    set_path("cross", "diamondpath");
    set_motion("cross", ForwardCycle, 3.0);
    set_path("slider","slidertrack");
    //set_motion("slider", ForwardOnce, 5.0);
}

fn testtrigger_enter() {
    print("here we go here we go");
    set_motion_goto("slider", 3, 6.0);
}

fn testtrigger2_enter() {
    print("re-e-wind");
    set_motion_goto("slider", 0, 6.0);
}
