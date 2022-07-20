use rdev::{grab, Event, EventType, Key};

fn main() {
    // This will block.
    if let Err(error) = grab(callback) {
        println!("Error: {:?}", error)
    }
}

fn callback(event: Event) -> Option<Event> {
    println!("My callback {:?}", event);
    match event.event_type {
        EventType::KeyPress(Key::F1) => {
            println!("Cancelling F1 !");
            None
        }
        _ => Some(event),
    }
}
