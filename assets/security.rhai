fn init() {
    extra_floor.set_path(up2);
    first_block.set_path(first_block_path);
    secret_slider.set_path(right3);
}

fn first_switch_interact() {
    first_block.go_to(2, 2.0);
    extra_floor.go_to(1, 2.0);
}

fn slider_trigger_enter() {
    if context.first("slider") {
        secret_slider.go_to(1, 4.0);
        context.after_frames(20, || {
            secret_slider.go_to(0, 4.0);
        })
    }
}
