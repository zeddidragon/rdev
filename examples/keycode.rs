use rdev;

fn main() {
    let keycode = rdev::linux_keycode_from_key(rdev::Key::Clear);
    dbg!(keycode);
}
