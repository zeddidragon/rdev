// This code is awful. Good luck
use crate::{
    key_from_scancode, linux_keycode_from_key, simulate, Event, EventType, GrabError,
    Key as RdevKey,
};
pub static mut IS_GRAB: bool = true;
use core::time;
use std::{
    collections::HashSet,
    ffi::c_int,
    mem::zeroed,
    ptr,
    sync::{mpsc::Sender, Arc, Mutex},
    time::SystemTime,
};
use strum::IntoEnumIterator;
use x11::xlib::{self, Display, GrabModeAsync, KeyPressMask, XUngrabKey};

const KEYPRESS_EVENT: i32 = 2;
const MODIFIERS: i32 = 0;

static mut GLOBAL_CALLBACK: Option<Box<dyn FnMut(Event) -> Option<Event>>> = None;

lazy_static::lazy_static! {
    pub static ref GRABED_KEYS: Arc<Mutex<HashSet<RdevKey>>> = Arc::new(Mutex::new(HashSet::<RdevKey>::new()));
    pub static ref BROADCAST_CONNECT: Arc<Mutex<Option<Sender<bool>>>> = Arc::new(Mutex::new(None));
}

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

fn is_key_grabed(key: RdevKey) -> bool {
    GRABED_KEYS.lock().unwrap().get(&key).is_some()
}

fn grab_key(display: *mut Display, grab_window: u64, keycode: i32) {
    println!("{:?} {:?}", "grab key", keycode);
    unsafe {
        xlib::XGrabKey(
            display,
            keycode,
            MODIFIERS as _,
            grab_window,
            c_int::from(true),
            GrabModeAsync,
            GrabModeAsync,
        );
    }
}

fn grab_keys(display: *mut Display, grab_window: u64) {
    println!("{:?}", "grab");
    for key in RdevKey::iter() {
        let event = convert_event(key, true);

        unsafe {
            if let Some(callback) = &mut GLOBAL_CALLBACK {
                let grab = callback(event).is_none();
                let keycode: i32 = linux_keycode_from_key(key).unwrap_or_default() as _;

                if grab && !is_key_grabed(key) {
                    grab_key(display, grab_window, keycode);
                    GRABED_KEYS.lock().unwrap().insert(key);
                }
            }
        }
    }
}

fn ungrab_key(display: *mut Display, grab_window: u64, keycode: i32) {
    println!("{:?} {:?}", "ungrab key", keycode);
    unsafe {
        XUngrabKey(display, keycode, MODIFIERS as _, grab_window);
    }
}

fn ungrab_keys(display: *mut Display, grab_window: u64) {
    for key in RdevKey::iter() {
        let keycode: i32 = linux_keycode_from_key(key).unwrap_or_default() as _;
        if is_key_grabed(key) {
            ungrab_key(display, grab_window, keycode);
            GRABED_KEYS.lock().unwrap().remove(&key);
        }
    }
}

fn set_key_hook() {
    unsafe {
        let display = xlib::XOpenDisplay(ptr::null());
        let screen_number = xlib::XDefaultScreen(display);
        let screen = xlib::XScreenOfDisplay(display, screen_number);
        let grab_window = xlib::XRootWindowOfScreen(screen);
        let my_grab_window = grab_window;

        let (send, recv) = std::sync::mpsc::channel::<bool>();
        *BROADCAST_CONNECT.lock().unwrap() = Some(send);

        let handle = std::thread::spawn(move || {
            let display = xlib::XOpenDisplay(ptr::null());

            xlib::XSelectInput(display, grab_window, KeyPressMask);
            let mut x_event: xlib::XEvent = zeroed();
            loop {
                if let Ok(is_grab) = recv.recv() {
                    if is_grab {
                        grab_keys(display, grab_window);
                        loop {
                            let mut should_quit = false;
                            if let Ok(is_grab) = recv.try_recv() {
                                println!("{:?} {:?}", "recv", is_grab);
                                if !is_grab {
                                    ungrab_keys(display, grab_window);
                                    should_quit = true;
                                }
                            }
                            xlib::XNextEvent(display, &mut x_event);

                            let key = key_from_scancode(x_event.key.keycode);
                            let is_press = x_event.type_ == KEYPRESS_EVENT;

                            println!("{:?} {:?}", key, is_press);
                            if should_quit{
                                break;
                            }
                        }
                    }
                }
            }
        });

        if let Some(sender) = BROADCAST_CONNECT.lock().unwrap().as_ref() {
            (*sender).send(true);
        }

        if let Err(e) = handle.join() {
            println!("Create thread failed {:?}", e);
        };
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
