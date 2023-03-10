fn init() {
    door.set_path(door_path);
}

fn left_switch_interact() {
    door.go_to(1, 4.0);
}
