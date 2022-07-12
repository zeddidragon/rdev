use rdev::{simulate, Button, EventType, Key, SimulateError};
use std::{thread, time};

fn send(event_type: &EventType) {
    let delay = time::Duration::from_millis(20);
    match simulate(event_type) {
        Ok(()) => (),
        Err(SimulateError) => {
            println!("We could not send {:?}", event_type);
        }
    }
    // Let ths OS catchup (at least MacOS)
    thread::sleep(delay);
}

fn main() {
    // Windows: LeftBracket
    // scancode 26 => [
    // in us: [
    // in fr: ^(dead key)

    send(&EventType::KeyPress(Key::Unknown(219)));
    send(&EventType::KeyRelease(Key::Unknown(219)));

    send(&EventType::KeyPress(Key::LeftBracket));
    send(&EventType::KeyRelease(Key::LeftBracket));
}
