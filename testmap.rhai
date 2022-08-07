fn init() {
    print("hello from rhai");
    set_constant_motion("layer1", -1, 0);
    set_constant_motion("layer2", 1, 0);
    set_constant_motion("layer3", 0, -1);
    set_path_motion("layer4", "orbit", 1.0, true);
    set_path_motion("cross", "diamondpath", 3.0, true);
}

fn testtrigger_enter() {
    print("here we go here we go");
    set_path_motion("slider","slidertrack", 2.5, false);
}
