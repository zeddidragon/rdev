use rdev::{listen, Event, EventType::*, Key as RdevKey};
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref MUTEX_SPECIAL_KEYS: Mutex<HashMap<RdevKey, bool>> = {
        let mut m = HashMap::new();
        // m.insert(RdevKey::PrintScreen, false); // 无反应

        m.insert(RdevKey::ShiftLeft, false);
        m.insert(RdevKey::ShiftRight, false);

        m.insert(RdevKey::ControlLeft, false);
        m.insert(RdevKey::ControlRight, false);

        m.insert(RdevKey::Alt, false);
        m.insert(RdevKey::AltGr, false);

        Mutex::new(m)
    };
}

fn main() {
    // This will block.
    std::env::set_var("KEYBOARD_ONLY", "y");

    let func = move |evt: Event| {
        let (key, down) = match evt.event_type {
            KeyPress(k) => {
                if MUTEX_SPECIAL_KEYS.lock().unwrap().contains_key(&k) {
                    if *MUTEX_SPECIAL_KEYS.lock().unwrap().get(&k).unwrap() {
                        return;
                    }
                    MUTEX_SPECIAL_KEYS.lock().unwrap().insert(k, true);
                }
                println!("keydown {:?} {:?} {:?}", k, evt.code, evt.scan_code);
                (k, 1)
            }
            KeyRelease(k) => {
                if MUTEX_SPECIAL_KEYS.lock().unwrap().contains_key(&k) {
                    MUTEX_SPECIAL_KEYS.lock().unwrap().insert(k, false);
                }
                println!("keyup {:?} {:?} {:?}", k, evt.code, evt.scan_code);
                (k, 0)
            }
            _ => return,
        };

        // todo: clear key
        #[cfg(target_os = "windows")]
        let scancode_key = rdev::key_from_scancode(evt.scan_code);
        #[cfg(target_os = "windows")]
        let key: RdevKey = if key == RdevKey::AltGr {
            // note: alt and altgr have same keycode.
            RdevKey::AltGr
        } else if scancode_key != RdevKey::Unknown(evt.scan_code) {
            // note: numpad should use keycode directly.
            rdev::key_from_scancode(evt.scan_code)
        } else {
            key
        };

        // todo: up down left right in numpad
        // #[cfg(target_os = "linux")]
        
        dbg!(key);
        println!("--------------");
    };
    if let Err(error) = rdev::listen(func) {
        dbg!("{:?}", error);
    }
}
