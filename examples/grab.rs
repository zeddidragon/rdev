use core::time::Duration;
use rdev::Event;
use rdev::EventType;
use rdev::Key as RdevKey;
use rdev::{disable_grab, enable_grab, exit_grab_listen, start_grab_listen};
use std::collections::HashSet;
use std::thread;

fn callback(event: Event) -> Option<Event> {
    match event.event_type {
        EventType::KeyPress(RdevKey::ControlLeft) | EventType::KeyRelease(RdevKey::ControlLeft) => {
            /*  */
            println!("{:?}", event.event_type);
            None
        }
        _ => Some(event),
    }
}

#[cfg(target_os = "linux")]
fn main() {
    let delay = Duration::from_secs(5);

    println!("[*] starting grab listen...");
    let mut keys: HashSet<RdevKey> = HashSet::<RdevKey>::new();
    keys.insert(RdevKey::ControlLeft);
    start_grab_listen(callback, keys);

    enable_grab();
    println!("[*] grab keys(5s), try to press Ctrl+C, won't work on other applications");
    thread::sleep(delay);

    disable_grab();
    println!("[*] grab keys(5s), try to press Ctrl+C");
    thread::sleep(delay);

    enable_grab();
    println!("[*] grab keys(5s), try to press Ctrl+C, won't work on other applications");
    thread::sleep(delay);

    exit_grab_listen();
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn main() {
    // This will block.
    if let Err(error) = grab(callback) {
        println!("Error: {:?}", error)
    }
}
