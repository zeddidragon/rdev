use crate::rdev::UnicodeInfo;
// This code is awful. Good luck
use crate::{key_from_code, Event, EventType, GrabError, Keyboard, KeyboardState};
use log::error;
use mio::{unix::SourceFd, Events, Interest, Poll, Token};
use std::{
    mem::zeroed,
    os::raw::c_int,
    ptr,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::{Duration, SystemTime},
};
use x11::xlib::{self, Display, GrabModeAsync, KeyPressMask, KeyReleaseMask, Window};

use super::common::KEYBOARD;

enum GrabEvent {
    Exit,
    KeyEvent(Event),
}

enum GrabControl {
    Grab,
    UnGrab,
    Exit,
}

struct KeyboardGrabber {
    display: *mut xlib::Display,
    screen: *mut xlib::Screen,
    window: Window,
    grab_fd: c_int,
}

unsafe impl Send for KeyboardGrabber {}
unsafe impl Sync for KeyboardGrabber {}

lazy_static::lazy_static! {
    static ref GRAB_KEY_EVENT_SENDER: Arc<Mutex<Option<Sender<GrabEvent>>>> = Arc::new(Mutex::new(None));
    static ref GRAB_CONTROL_SENDER: Arc<Mutex<Option<Sender<GrabControl>>>> = Arc::new(Mutex::new(None));
}

const KEYPRESS_EVENT: i32 = 2;

static mut EXIT_GRAB: bool = false;
static mut GLOBAL_CALLBACK: Option<Box<dyn FnMut(Event) -> Option<Event>>> = None;
const GRAB_RECV: Token = Token(0);

impl KeyboardGrabber {
    fn create() -> Result<Self, GrabError> {
        let mut grabber = Self {
            display: ptr::null_mut(),
            screen: ptr::null_mut(),
            window: 0,
            grab_fd: 0,
        };
        grabber.display = unsafe { xlib::XOpenDisplay(ptr::null()) };
        if grabber.display.is_null() {
            return Err(GrabError::MissingDisplayError);
        }

        let screen_number = unsafe { xlib::XDefaultScreen(grabber.display) };
        grabber.screen = unsafe { xlib::XScreenOfDisplay(grabber.display, screen_number) };
        if grabber.screen.is_null() {
            return Err(GrabError::MissScreenError);
        }

        grabber.window = unsafe { xlib::XRootWindowOfScreen(grabber.screen) };
        unsafe {
            // to-do: check the result.
            // No documentation on the return value of this function
            // https://tronche.com/gui/x/xlib/event-handling/XSelectInput.html
            xlib::XSelectInput(
                grabber.display,
                grabber.window,
                KeyPressMask | KeyReleaseMask,
            );
        }

        grabber.grab_fd = unsafe { xlib::XConnectionNumber(grabber.display) };

        Ok(grabber)
    }

    fn start(&self, exit: Arc<Mutex<bool>>) -> Result<(), GrabError> {
        let poll = Poll::new().map_err(GrabError::IoError)?;
        poll.registry()
            .register(&mut SourceFd(&self.grab_fd), GRAB_RECV, Interest::READABLE)
            .map_err(GrabError::IoError)?;

        let (tx, rx) = channel();
        GRAB_CONTROL_SENDER.lock().unwrap().replace(tx);

        start_grab_control_thread(self.display as u64, self.window, exit.clone(), rx);
        loop_poll_x_event(self.display, exit, poll);
        Ok(())
    }
}

impl Drop for KeyboardGrabber {
    fn drop(&mut self) {
        if !self.display.is_null() {
            ungrab_keys(self.display);
            let _ignore = unsafe { xlib::XCloseDisplay(self.display) };
        }
    }
}

#[inline]
fn is_control(unicode_info: &Option<UnicodeInfo>) -> bool {
    unicode_info.as_ref().map_or(false, |unicode_info| {
        unicode_info.name.as_ref().map_or(false, |seq| {
            for chr in seq.chars() {
                if chr.is_control() {
                    return true;
                }
            }
            false
        })
    })
}

fn convert_event(code: u32, is_press: bool) -> Event {
    let key = key_from_code(code);
    let event_type = if is_press {
        EventType::KeyPress(key)
    } else {
        EventType::KeyRelease(key)
    };

    let (unicode, platform_code) = unsafe {
        if let Some(kbd) = &mut KEYBOARD {
            // delete -> \u{7f}
            let unicode_info = kbd.add(&event_type);
            if is_control(&unicode_info) {
                (None, kbd.keysym())
            } else {
                (unicode_info, kbd.keysym())
            }
        } else {
            (None, 0)
        }
    };

    Event {
        event_type,
        time: SystemTime::now(),
        unicode,
        platform_code,
        position_code: code as _,
    }
}

fn grab_keys(display: *mut Display, grab_window: libc::c_ulong) {
    unsafe {
        xlib::XGrabKeyboard(
            display,
            grab_window,
            c_int::from(true),
            GrabModeAsync,
            GrabModeAsync,
            xlib::CurrentTime,
        );
        xlib::XFlush(display);
        thread::sleep(Duration::from_millis(50));
    }
}

fn ungrab_keys(display: *mut Display) {
    unsafe {
        xlib::XUngrabKeyboard(display, xlib::CurrentTime);
        xlib::XFlush(display);
        thread::sleep(Duration::from_millis(50));
    }
}

fn start_callback_event_thread(recv: Receiver<GrabEvent>) {
    thread::spawn(move || loop {
        if let Ok(data) = recv.recv() {
            match data {
                GrabEvent::KeyEvent(event) => unsafe {
                    if let Some(callback) = &mut GLOBAL_CALLBACK {
                        callback(event);
                    }
                },
                GrabEvent::Exit => {
                    break;
                }
            }
        }
    });
}

fn start_grab_service() -> Result<(), GrabError> {
    let (tx, rx) = channel::<GrabEvent>();
    *GRAB_KEY_EVENT_SENDER.lock().unwrap() = Some(tx);

    unsafe {
        // to-do: is display pointer in keyboard always valid?
        // KEYBOARD usage is very confusing and error prone.
        KEYBOARD = Keyboard::new();
        if KEYBOARD.is_none() {
            return Err(GrabError::KeyboardError);
        }
    }

    start_grab_thread();
    start_callback_event_thread(rx);
    Ok(())
}

fn read_x_event(x_event: &mut xlib::XEvent, display: *mut xlib::Display) {
    while (unsafe { xlib::XPending(display) }) > 0 {
        unsafe {
            // to-do: check the result.
            // No documentation on the return value of this function
            // https://linux.die.net/man/3/xnextevent
            xlib::XNextEvent(display, x_event);
        }
        let keycode = unsafe { x_event.key.keycode };
        let is_press = unsafe { x_event.type_ == KEYPRESS_EVENT };
        let event = convert_event(keycode, is_press);
        if let Some(tx) = GRAB_KEY_EVENT_SENDER.lock().unwrap().as_ref() {
            tx.send(GrabEvent::KeyEvent(event)).ok();
        }
    }
}

fn start_grab_control_thread(
    display: u64,
    grab_window: Window,
    exit_clone: Arc<Mutex<bool>>,
    rx: Receiver<GrabControl>,
) {
    std::thread::spawn(move || {
        let display = display as *mut xlib::Display;
        loop {
            match rx.recv() {
                Ok(evt) => match evt {
                    GrabControl::Exit => {
                        *exit_clone.lock().unwrap() = true;
                        break;
                    }
                    GrabControl::Grab => {
                        grab_keys(display, grab_window);
                    }
                    GrabControl::UnGrab => {
                        ungrab_keys(display);
                    }
                },
                Err(e) => {
                    // unreachable
                    log::error!("Failed to receive event, {}", e);
                    break;
                }
            }
        }
    });
}

fn loop_poll_x_event(display: *mut xlib::Display, exit: Arc<Mutex<bool>>, mut poll: Poll) {
    let mut x_event: xlib::XEvent = unsafe { zeroed() };
    let mut events = Events::with_capacity(128);
    loop {
        if *exit.lock().unwrap() {
            break;
        }

        match poll.poll(&mut events, Some(Duration::from_millis(300))) {
            Ok(_) => {
                for event in &events {
                    match event.token() {
                        GRAB_RECV => {
                            read_x_event(&mut x_event, display);
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to poll event, {}", e);
                break;
            }
        }
    }
}

#[inline]
fn start_grab(exit: Arc<Mutex<bool>>) -> Result<(), GrabError> {
    let grabber = KeyboardGrabber::create()?;
    grabber.start(exit)
}

fn start_grab_thread() {
    thread::spawn(|| {
        let mut c = 0;
        loop {
            if unsafe { EXIT_GRAB } {
                break;
            }
            let exit = Arc::new(Mutex::new(false));
            if let Err(err) = start_grab(exit.clone()) {
                log::debug!("Failed to start grab keyboard, {:?}", err);
                if c <= 3 {
                    c += 1;
                    thread::sleep(Duration::from_millis(100));
                }
                if c > 3 && c < 10 {
                    thread::sleep(Duration::from_millis(c * 100));
                } else {
                    thread::sleep(Duration::from_millis(1000));
                }
            } else {
                c = 0;
            }
            if exit.lock().unwrap().clone() {
                break;
            }
        }
    });
}

fn send_grab_control(data: GrabControl) {
    match GRAB_CONTROL_SENDER.lock().unwrap().as_ref() {
        Some(sender) => {
            if let Err(e) = sender.send(data) {
                error!("Failed to send grab command, {e}");
            }
        }
        None => {
            error!("Failed to send grab command, no sender");
        }
    }
    thread::sleep(Duration::from_millis(50));
}

#[inline]
pub fn enable_grab() {
    send_grab_control(GrabControl::Grab);
}

#[inline]
pub fn disable_grab() {
    send_grab_control(GrabControl::UnGrab);
}

pub fn start_grab_listen<T>(callback: T) -> Result<(), GrabError>
where
    T: FnMut(Event) -> Option<Event> + 'static,
{
    unsafe {
        GLOBAL_CALLBACK = Some(Box::new(callback));
    }
    start_grab_service()?;
    thread::sleep(Duration::from_millis(100));
    Ok(())
}

pub fn exit_grab_listen() {
    unsafe {
        EXIT_GRAB = true;
    }
    if let Some(tx) = GRAB_KEY_EVENT_SENDER.lock().unwrap().as_ref() {
        tx.send(GrabEvent::Exit).ok();
    }
    send_grab_control(GrabControl::Exit);
}
