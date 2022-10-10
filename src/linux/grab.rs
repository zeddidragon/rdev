// This code is awful. Good luck
use crate::{
    key_from_scancode, linux_keycode_from_key, simulate, Event, EventType, GrabError,
    Key as RdevKey,
};
use std::{
    collections::HashSet,
    mem::zeroed,
    os::raw::c_int,
    ptr,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
        Arc, Mutex,
    },
    time::SystemTime,
};
use strum::IntoEnumIterator;
use x11::xlib::{self, Display, GrabModeAsync, KeyPressMask, XUngrabKey};

const KEYPRESS_EVENT: i32 = 2;
const MODIFIERS: i32 = 0;

pub static mut GLOBAL_CALLBACK: Option<Box<dyn FnMut(Event) -> Option<Event>>> = None;
pub static mut IS_GRAB: AtomicBool = AtomicBool::new(false);

lazy_static::lazy_static! {
    pub static ref GRABED_KEYS: Arc<Mutex<HashSet<RdevKey>>> = Arc::new(Mutex::new(HashSet::<RdevKey>::new()));
    pub static ref BROADCAST_CONNECT: Arc<Mutex<Option<Sender<bool>>>> = Arc::new(Mutex::new(None));
}
pub fn init_grabed_keys() {
    for key in RdevKey::iter() {
        for press in [true, false] {
            let event = convert_event(key, press);
            unsafe {
                if let Some(callback) = &mut GLOBAL_CALLBACK {
                    if callback(event).is_none() {
                        GRABED_KEYS.lock().unwrap().insert(key);
                    }
                }
            }
        }
    }
}

fn send(event_type: &EventType) {
    match simulate(event_type) {
        Ok(()) => (),
        Err(simulate_error) => {
            println!("We could not send {:?}", simulate_error);
        }
    }
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
    for key in RdevKey::iter() {
        let keycode: i32 = linux_keycode_from_key(key).unwrap_or_default() as _;
        if is_key_grabed(key) {
            grab_key(display, grab_window, keycode);
        }
    }
}

fn ungrab_key(display: *mut Display, grab_window: u64, keycode: i32) {
    unsafe {
        XUngrabKey(display, keycode, MODIFIERS as _, grab_window);
    }
}

fn ungrab_keys(display: *mut Display, grab_window: u64) {
    for key in RdevKey::iter() {
        let keycode: i32 = linux_keycode_from_key(key).unwrap_or_default() as _;
        if is_key_grabed(key) {
            ungrab_key(display, grab_window, keycode);
        }
    }
}

fn set_key_hook() {
    unsafe {
        let display = xlib::XOpenDisplay(ptr::null());
        let screen_number = xlib::XDefaultScreen(display);
        let screen = xlib::XScreenOfDisplay(display, screen_number);
        let grab_window = xlib::XRootWindowOfScreen(screen);

        // Passing null pointers for the things we don't need results in a
        // segfault.
        let mut root_return: xlib::Window = 0;
        let mut child_return: xlib::Window = 0;
        let mut root_x_return = 0;
        let mut root_y_return = 0;
        let mut win_x_return = 0;
        let mut win_y_return = 0;
        let mut mask_return = 0;
        xlib::XQueryPointer(
            display,
            grab_window,
            &mut root_return,
            &mut child_return,
            &mut root_x_return,
            &mut root_y_return,
            &mut win_x_return,
            &mut win_y_return,
            &mut mask_return,
        );
        let numlocked = mask_return & 16 != 0;
        if numlocked {
            send(&EventType::KeyPress(RdevKey::NumLock));
            send(&EventType::KeyRelease(RdevKey::NumLock));
        };

        let (send, recv) = std::sync::mpsc::channel::<bool>();
        *BROADCAST_CONNECT.lock().unwrap() = Some(send);

        let handle = std::thread::spawn(move || {
            let display = xlib::XOpenDisplay(ptr::null());

            xlib::XSelectInput(display, grab_window, KeyPressMask);
            let mut x_event: xlib::XEvent = zeroed();
            loop {
                if IS_GRAB.load(Ordering::SeqCst) {
                    grab_keys(display, grab_window);
                    loop {
                        xlib::XNextEvent(display, &mut x_event);
                        if !IS_GRAB.load(Ordering::SeqCst) {
                            ungrab_keys(display, grab_window);
                            xlib::XNextEvent(display, &mut x_event);
                            break;
                        }

                        let key = key_from_scancode(x_event.key.keycode);
                        let is_press = x_event.type_ == KEYPRESS_EVENT;
                        let event = convert_event(key, is_press);

                        if let Some(callback) = &mut GLOBAL_CALLBACK {
                            callback(event);
                        }
                    }
                }
            }
        });

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
    if GRABED_KEYS.lock().unwrap().len() == 0 {
        init_grabed_keys();
    }
    set_key_hook();
    Ok(())
}
