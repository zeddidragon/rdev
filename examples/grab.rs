use rdev::{grab, Event, EventType, Key};
#[allow(unused)]
#[cfg(target_os = "linux")]
use rdev::{BROADCAST_CONNECT, GRABED_KEYS, IS_GRAB};

fn callback(event: Event) -> Option<Event> {
    match event.event_type {
        EventType::KeyPress(Key::Tab) | EventType::KeyRelease(Key::Tab) => {
            println!("{:?}", event.event_type);
            None
        }
        _ => Some(event),
    }
}
/* Notice: XGrabKey need without NUMLOCK */
fn main() {
    #[cfg(target_os = "linux")]
    {
        std::thread::spawn(|| {
            let delay = core::time::Duration::from_millis(100);
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
