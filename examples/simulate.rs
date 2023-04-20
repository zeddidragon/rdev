use rdev::{simulate, EventType, Key, SimulateError};
use std::{
    thread::{self, sleep_ms},
    time,
};

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

#[cfg(target_os = "macos")]
fn test_macos_keys() {
    let virtual_input = rdev::VirtualInput::new(
        rdev::CGEventSourceStateID::Private,
        rdev::CGEventTapLocation::Session,
    )
    .unwrap();

    let key_caps = rdev::Key::RawKey(rdev::RawKey::MacVirtualKeycode(rdev::kVK_CapsLock));
    let key_ansi_a = rdev::Key::RawKey(rdev::RawKey::MacVirtualKeycode(rdev::kVK_ANSI_A));
    {
        println!(
            "caps lock satte 1 {}",
            rdev::VirtualInput::get_key_state(
                rdev::CGEventSourceStateID::CombinedSessionState,
                rdev::kVK_CapsLock
            )
        );

        virtual_input
            .simulate(&rdev::EventType::KeyPress(key_caps))
            .unwrap();
        thread::sleep(time::Duration::from_millis(20));

        println!(
            "caps lock satte 2 {}",
            rdev::VirtualInput::get_key_state(
                rdev::CGEventSourceStateID::CombinedSessionState,
                rdev::kVK_CapsLock
            )
        );

        virtual_input
            .simulate(&rdev::EventType::KeyPress(key_ansi_a))
            .unwrap();
        thread::sleep(time::Duration::from_millis(20));
        virtual_input
            .simulate(&rdev::EventType::KeyRelease(key_ansi_a))
            .unwrap();
        thread::sleep(time::Duration::from_millis(20));

        virtual_input
            .simulate(&rdev::EventType::KeyRelease(key_caps))
            .unwrap();
        thread::sleep(time::Duration::from_millis(20));

        println!(
            "caps lock satte 3 {}",
            rdev::VirtualInput::get_key_state(
                rdev::CGEventSourceStateID::CombinedSessionState,
                rdev::kVK_CapsLock
            )
        );
    }

    let command_tab = rdev::Key::RawKey(rdev::RawKey::MacVirtualKeycode(rdev::kVK_Command));
    let key_tab = rdev::Key::RawKey(rdev::RawKey::MacVirtualKeycode(rdev::kVK_CapsLock));
    {
        virtual_input
            .simulate(&rdev::EventType::KeyPress(command_tab))
            .unwrap();
        thread::sleep(time::Duration::from_millis(200));

        virtual_input
            .simulate(&rdev::EventType::KeyPress(key_tab))
            .unwrap();
        thread::sleep(time::Duration::from_millis(200));
        virtual_input
            .simulate(&rdev::EventType::KeyRelease(key_tab))
            .unwrap();
        thread::sleep(time::Duration::from_millis(200));

        virtual_input
            .simulate(&rdev::EventType::KeyPress(key_tab))
            .unwrap();
        thread::sleep(time::Duration::from_millis(200));
        virtual_input
            .simulate(&rdev::EventType::KeyRelease(key_tab))
            .unwrap();
        thread::sleep(time::Duration::from_millis(200));

        virtual_input
            .simulate(&rdev::EventType::KeyRelease(command_tab))
            .unwrap();
        thread::sleep(time::Duration::from_millis(200));
    }
}

#[cfg(windows)]
fn test_simulate_vk() {
    rdev::simulate_code(Some(0xA2), None, true);
    rdev::simulate_code(Some(0x4F), None, true);
    rdev::simulate_code(Some(0x4F), None, false);
    rdev::simulate_code(Some(0xA2), None, false);
}

#[cfg(windows)]
fn test_simulate_char() {
    println!("{:?}", rdev::simulate_char('A', false));
    println!("{:?}", rdev::simulate_char('€', false));
    println!("{:?}", rdev::simulate_char('€', true));
}

fn main() {
    // Windows: LeftBracket
    // scancode 26 => [
    // in us: [
    // in fr: ^(dead key)

    // send(&EventType::KeyPress(Key::Unknown(219)));
    // send(&EventType::KeyRelease(Key::Unknown(219)));

    // send(&EventType::KeyPress(Key::LeftBracket));
    // send(&EventType::KeyRelease(Key::LeftBracket));

    // // Combination
    // send(&EventType::KeyPress(Key::ControlLeft));
    // rdev::simulate_char('€', true);
    // rdev::simulate_char('€', false);
    // // send_char('a', false);
    // send(&EventType::KeyRelease(Key::ControlLeft));
    //send(&EventType::KeyPress(Key::AltGr));
    // send(&EventType::KeyPress(Key::Num3));

    // send(&EventType::KeyRelease(Key::Num3));
    // send(&EventType::KeyRelease(Key::AltGr));

    #[cfg(windows)]
    test_simulate_vk();

    #[cfg(windows)]
    test_simulate_char();

    #[cfg(target_os = "macos")]
    test_macos_keys();
}
