use rdev::{simulate, EventType, Key, SimulateError};
use std::{thread::{self, sleep_ms}, time};

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

// fn send_char(chr: char, pressed: bool) {
//     let delay = time::Duration::from_millis(20);
//     match simulate_char(chr, pressed) {
//         Ok(()) => (),
//         Err(SimulateError) => {
//             println!("We could not send {:?}", chr);
//         }
//     }
//     // Let ths OS catchup (at least MacOS)
//     thread::sleep(delay);
// }

fn main() {
    // Windows: LeftBracket
    // scancode 26 => [
    // in us: [
    // in fr: ^(dead key)

    // send(&EventType::KeyPress(Key::Unknown(219)));
    // send(&EventType::KeyRelease(Key::Unknown(219)));

    // send(&EventType::KeyPress(Key::LeftBracket));
    // send(&EventType::KeyRelease(Key::LeftBracket));

    // // Conbination
    // send(&EventType::KeyPress(Key::ControlLeft));
    // rdev::simulate_char('€', true);
    // rdev::simulate_char('€', false);
    // // send_char('a', false);
    // send(&EventType::KeyRelease(Key::ControlLeft));
    //send(&EventType::KeyPress(Key::AltGr));
    // send(&EventType::KeyPress(Key::Num3));

    // send(&EventType::KeyRelease(Key::Num3));
    //send(&EventType::KeyRelease(Key::AltGr));

    rdev::simulate_code(Some(0xA2), None, true);
    rdev::simulate_code(Some(0x4F), None, true);
    rdev::simulate_code(Some(0x4F), None, false);
    rdev::simulate_code(Some(0xA2), None, false);
}
