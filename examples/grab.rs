#[allow(unused)]
#[cfg(target_os = "linux")]
use rdev::BROADCAST_CONNECT;
use rdev::{grab, Event, EventType, Key};

/*

#[cfg(target_os = "linux")]
if let Some(sender) = BROADCAST_CONNECT.lock().unwrap().as_ref() {
    (*sender).send(false);
}

*/
fn callback(event: Event) -> Option<Event> {
    match event.event_type {
        EventType::KeyPress(Key::Tab) => None,
        EventType::KeyPress(Key::ControlLeft) => None,
        _ => Some(event),
    }
}

fn main() {
    // This will block.
    if let Err(error) = grab(callback) {
        println!("Error: {:?}", error)
    }
}
