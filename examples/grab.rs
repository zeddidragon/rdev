use rdev::{grab, start_grab_listen, exit_grab_listen, ungrab};
use core::time::Duration;
use std::thread;
use rdev::Event;
use rdev::EventType;
use rdev::Key;


fn callback(event: Event) -> Option<Event> {
    match event.event_type {
        EventType::KeyPress(Key::ControlLeft) | EventType::KeyRelease(Key::ControlLeft) => {
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

    grab();
    println!("[*] grab keys(5s), try to press Ctrl+C, won't work on other applications");
    thread::sleep(delay);

    ungrab();
    println!("[*] grab keys(5s), try to press Ctrl+C");
    thread::sleep(delay);

    grab();
    println!("[*] grab keys(5s), try to press Ctrl+C, won't work on other applications");
    thread::sleep(delay);

    exit_grab_listen();
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn main(){
    // This will block.
    if let Err(error) = grab(callback) {
        println!("Error: {:?}", error)
    }
}