// This code is awful. Good luck
use crate::{
    key_from_scancode, linux_keycode_from_key, Event, EventType, GrabError, Key as RdevKey,
};
use mio::unix::SourceFd;
use mio::{Events, Interest, Poll, Token};
use std::os::unix::net::UnixDatagram;
use std::os::unix::prelude::AsRawFd;
use std::path::Path;
use std::ptr;
use std::thread;
use std::{
    collections::HashSet,
    mem::zeroed,
    os::raw::c_int,
    sync::{mpsc::Sender, Arc, Mutex},
    time::SystemTime,
};
use strum::IntoEnumIterator;
use x11::xlib::{self, AnyModifier, Display, GrabModeAsync, KeyPressMask};

#[derive(Debug)]
pub struct MyDisplay(*mut xlib::Display);
unsafe impl Sync for MyDisplay {}
unsafe impl Send for MyDisplay {}

lazy_static::lazy_static! {
    pub static ref KEYS: Arc<Mutex<HashSet<RdevKey>>> = Arc::new(Mutex::new(HashSet::<RdevKey>::new()));
    pub static ref SENDER: Arc<Mutex<Option<Sender<GrabEvent>>>> = Arc::new(Mutex::new(None));
}

const KEYPRESS_EVENT: i32 = 2;

pub static mut GLOBAL_CALLBACK: Option<Box<dyn FnMut(Event) -> Option<Event>>> = None;
pub static FILE_PATH: &str = "/tmp/rdev_service.sock";
const GRAB_KEY: Token = Token(0);
const SERVICE_CLIENT: Token = Token(1);

pub enum GrabEvent {
    Grab,
    UnGrab,
    Exit,
}

pub fn init_grabed_keys() {
    for key in RdevKey::iter() {
        for press in [true, false] {
            let event = convert_event(key, press);
            unsafe {
                if let Some(callback) = &mut GLOBAL_CALLBACK {
                    if callback(event).is_none() {
                        KEYS.lock().unwrap().insert(key);
                    }
                }
            }
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
    KEYS.lock().unwrap().get(&key).is_some()
}

fn grab_key(display: *mut Display, grab_window: u64, keycode: i32) {
    unsafe {
        xlib::XGrabKey(
            display,
            keycode,
            AnyModifier,
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

pub fn grab() -> Result<(), GrabError> {
    if let Some(tx) = &*SENDER.lock().unwrap() {
        tx.send(GrabEvent::Grab).ok();
    } else {
        return Err(GrabError::ListenError);
    };
    Ok(())
}

pub fn ungrab() -> Result<(), GrabError> {
    if let Some(tx) = &*SENDER.lock().unwrap() {
        tx.send(GrabEvent::UnGrab).ok();
    } else {
        return Err(GrabError::ListenError);
    };
    Ok(())
}

pub fn start_grab_listen<T>(callback: T)
where
    T: FnMut(Event) -> Option<Event> + 'static,
{
    unsafe {
        GLOBAL_CALLBACK = Some(Box::new(callback));
    }
    init_grabed_keys();

    if let Err(error) = grab_service() {
        println!("[-] grab error in rdev: {:?}", error);
        if error != GrabError::SimulateError {
            unsafe {
                libc::exit(1);
            }
        }
    }
}

pub fn exit_grab_listen() -> Result<(), GrabError> {
    if let Some(tx) = &*SENDER.lock().unwrap() {
        if tx.send(GrabEvent::Exit).is_err() {
            return Err(GrabError::ListenError);
        };
    } else {
        return Err(GrabError::ListenError);
    };
    Ok(())
}

fn grab_service() -> Result<(), GrabError> {
    let (send, recv) = std::sync::mpsc::channel::<GrabEvent>();
    *SENDER.lock().unwrap() = Some(send);

    thread::spawn(move || loop {
        if let Ok(data) = recv.recv() {
            match data {
                GrabEvent::Grab => {
                    start_grab_thread();
                }
                GrabEvent::UnGrab => {
                    let socket = UnixDatagram::unbound().unwrap();
                    if let Err(e) = socket.connect(FILE_PATH) {
                        println!("[-] grab error in rdev: Couldn't connect: {:?}", e);
                    } else {
                        socket
                            .send(b"")
                            .expect("[-] grab error in rdev: recv function failed");
                    };
                }
                GrabEvent::Exit => {
                    break;
                }
            }
        }
    });

    Ok(())
}

fn unlink_socket(path: impl AsRef<Path>) {
    let path = path.as_ref();
    if Path::new(path).exists() {
        let result = std::fs::remove_file(path);
        if let Err(e) = result {
            println!("[-] grab error in rdev: Couldn't remove the file: {:?}", e);
        }
    }
}

fn start_grab_thread() {
    thread::spawn(move || {
        let display = unsafe { xlib::XOpenDisplay(ptr::null()) };
        /* todo! how to get  Exception*/
        if display.is_null() {
            panic!("[-] grab error in rdev: MissingDisplayError");
        }
        let screen_number = unsafe { xlib::XDefaultScreen(display) };
        /* todo! Multiple Display */
        let screen = unsafe { xlib::XScreenOfDisplay(display, screen_number) };
        if screen.is_null() {
            panic!("[-] grab error in rdev: XScreenOfDisplay Error");
        }
        let grab_window = unsafe { xlib::XRootWindowOfScreen(screen) };

        unsafe {
            xlib::XSelectInput(display, grab_window, KeyPressMask);
        }
        grab_keys(display, grab_window);
        unsafe { xlib::XFlush(display) };

        let grab_fd = unsafe { xlib::XConnectionNumber(display) };
        unlink_socket(FILE_PATH);
        let socket = match UnixDatagram::bind(FILE_PATH) {
            Ok(socket) => socket,
            Err(err) => panic!("[-] grab error in rdev: {:?}", err),
        };
        if socket.set_nonblocking(true).is_err() {
            panic!("[-] grab error in rdev: set_nonblocking");
        };

        if let Ok(mut poll) = Poll::new() {
            let mut events = Events::with_capacity(128);
            if poll
                .registry()
                .register(&mut SourceFd(&grab_fd), GRAB_KEY, Interest::READABLE)
                .is_err()
            {
                panic!("[-] grab error in rdev: Poll register grab fd failed");
            };
            if poll
                .registry()
                .register(
                    &mut SourceFd(&socket.as_raw_fd()),
                    SERVICE_CLIENT,
                    Interest::READABLE,
                )
                .is_err()
            {
                panic!("[-] grab error in rdev: Poll register socket fd failed");
            };

            let mut x_event: xlib::XEvent = unsafe { zeroed() };

            loop {
                if poll.poll(&mut events, None).is_err() {
                    println!("[-] grab error in rdev: Poll poll failed");
                };
                for event in &events {
                    if event.token() == SERVICE_CLIENT {
                        println!("Exit grab thread");
                        unsafe {
                            xlib::XCloseDisplay(display);
                        }
                        break;
                    }
                    unsafe { xlib::XNextEvent(display, &mut x_event) };

                    let key = unsafe { key_from_scancode(x_event.key.keycode) };
                    let is_press = unsafe { x_event.type_ == KEYPRESS_EVENT };
                    let event = convert_event(key, is_press);

                    unsafe {
                        if let Some(callback) = &mut GLOBAL_CALLBACK {
                            callback(event);
                        }
                    }
                }
            }
        } else {
            panic!("[-] grab error in rdev: Create Poll failed");
        }
    });
}
