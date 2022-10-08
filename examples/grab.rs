use rdev::{grab, Event, EventType, Key};

fn callback(event: Event) -> Option<Event> {
    match event.event_type{
        EventType::KeyPress(Key::Tab) => None,
        _ => Some(event),
    }
}

fn main(){
    // This will block.
    if let Err(error) = grab(callback) {
        println!("Error: {:?}", error)
    }
}