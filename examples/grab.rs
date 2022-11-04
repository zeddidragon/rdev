use core::time::Duration;
use rdev::Event;
use rdev::EventType;
use rdev::{disable_grab, enable_grab, exit_grab_listen, start_grab_listen};
use std::thread;

fn callback(event: Event) -> Option<Event> {
    match event.event_type {
        EventType::KeyPress(_key) | EventType::KeyRelease(_key) => {
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
    start_grab_listen(callback);

    println!("[*] grab keys(5s), try to press Ctrl+C, won't work on other applications");
    enable_grab();
    thread::sleep(delay);

    println!("[*] ungrab keys(5s), try to press Ctrl+C");
    disable_grab();
    thread::sleep(delay);

    println!("[*] grab keys(5s), try to press Ctrl+C, won't work on other applications");
    enable_grab();
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
