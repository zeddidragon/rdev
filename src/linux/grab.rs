use crate::{
    key_from_scancode, linux_keycode_from_key, Event, EventType, GrabError, Key as RdevKey,
};
use std::{ffi::c_int, mem::zeroed, ptr, time::SystemTime};
use strum::IntoEnumIterator;
use x11::xlib::{self, GrabModeAsync, KeyPressMask};

const KEYPRESS_EVENT: i32 = 2;

static mut GLOBAL_CALLBACK: Option<Box<dyn FnMut(Event) -> Option<Event>>> = None;

fn convert_event(key: RdevKey, is_press: bool) -> Event {
    Event {
        event_type: if is_press {
            EventType::KeyPress(key)
        } else {
            EventType::KeyRelease(key)
        },
        time: SystemTime::now(),
        name: None,
        code: linux_keycode_from_key(key).unwrap_or_default() as _,
        scan_code: linux_keycode_from_key(key).unwrap_or_default() as _,
    }
}

fn set_key_hook() {
    unsafe {
        let display = xlib::XOpenDisplay(ptr::null());
        let screen_number = xlib::XDefaultScreen(display);
        let screen = xlib::XScreenOfDisplay(display, screen_number);
        let grab_window = xlib::XRootWindowOfScreen(screen);

        // Grab?
        for key in RdevKey::iter() {
            let event = convert_event(key, true);
            if let Some(callback) = &mut GLOBAL_CALLBACK {
                let is_report = callback(event).is_some();
                let keycode: i32 = linux_keycode_from_key(key).unwrap_or_default() as _;
                let modifiers = 0;

                if !is_report {
                    xlib::XGrabKey(
                        display,
                        keycode,
                        modifiers,
                        grab_window,
                        c_int::from(is_report),
                        GrabModeAsync,
                        GrabModeAsync,
                    );
                }
            }
        }

        xlib::XSelectInput(display, grab_window, KeyPressMask);
        let mut x_event: xlib::XEvent = zeroed();
        loop {
            xlib::XNextEvent(display, &mut x_event);
            let key = key_from_scancode(x_event.key.keycode);
            let is_press = x_event.type_ == KEYPRESS_EVENT;

            let event = convert_event(key, is_press);
            if let Some(callback) = &mut GLOBAL_CALLBACK {
                let _is_report = callback(event).is_some();
            }

            println!("{:?} {:?}", key, is_press);
        }
    }
}

pub fn grab<T>(callback: T) -> Result<(), GrabError>
where
    T: FnMut(Event) -> Option<Event> + 'static,
{
    unsafe {
        GLOBAL_CALLBACK = Some(Box::new(callback));
    }
    set_key_hook();
    Ok(())
}