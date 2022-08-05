fn init() {
    print("hello from rhai");
    set_constant_motion("layer1", -1, 0);
    set_constant_motion("layer2", 1, 0);
    set_constant_motion("layer3", 0, -1);
    set_path_motion("layer4", "orbit", 1.0);
    set_path_motion("cross", "diamondpath", 3.0);
}
