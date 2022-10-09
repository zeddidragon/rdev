use rdev::{grab, Event, EventType, Key};
#[allow(unused)]
#[cfg(target_os = "linux")]
use rdev::{BROADCAST_CONNECT, GRABED_KEYS, IS_GRAB};

fn callback(event: Event) -> Option<Event> {
    match event.event_type {
        EventType::KeyPress(key) | EventType::KeyRelease(key) => {
            println!("{:?}", key);
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

            unsafe {
                IS_GRAB.store(true, std::sync::atomic::Ordering::SeqCst);
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
