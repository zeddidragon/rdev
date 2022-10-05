#[cfg(target_os = "windows")]
use rdev::{grab, Event, EventType, Key};

#[cfg(target_os = "windows")]
fn callback(event: Event) -> Option<Event> {
    println!("My callback {:?}", event);
    match event.event_type{
        EventType::KeyPress(Key::Tab) => None,
        _ => Some(event),
    }
}

fn main(){
    // This will block.
    #[cfg(target_os = "windows")]
    if let Err(error) = grab(callback) {
        println!("Error: {:?}", error)
    }
}