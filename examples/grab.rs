use rdev::{grab, Event, EventType, Key};
#[allow(unused)]
#[cfg(target_os = "linux")]
use rdev::{BROADCAST_CONNECT, GRABED_KEYS};

fn callback(event: Event) -> Option<Event> {
    match event.event_type {
        EventType::KeyPress(Key::Tab) | EventType::KeyRelease(Key::Tab) => {
            println!("{:?}", event);
            None
        }
        EventType::KeyPress(Key::ControlLeft) | EventType::KeyRelease(Key::ControlLeft) => {
            println!("{:?}", event);
            None
        }
        _ => Some(event),
    }
}

fn main() {
    #[cfg(target_os = "linux")]
    {
        std::thread::spawn(|| {
            let delay = core::time::Duration::from_millis(10);
            std::thread::sleep(delay);
            if let Some(sender) = BROADCAST_CONNECT.lock().unwrap().as_ref() {
                (*sender).send(true);
            }
        });

        GRABED_KEYS.lock().unwrap().insert(Key::Tab);
        GRABED_KEYS.lock().unwrap().insert(Key::ControlLeft);
    }
    // This will block.
    if let Err(error) = grab(callback) {
        println!("Error: {:?}", error)
    }
}
