fn main() {
    let keycode = rdev::wayland_keycode_from_key(rdev::Key::Num1);
    dbg!(keycode);
}
