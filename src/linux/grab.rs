use crate::rdev::UnicodeInfo;
// This code is awful. Good luck
use crate::{key_from_code, Event, EventType, GrabError, Keyboard, KeyboardState};
use log::error;
use mio::{unix::SourceFd, Events, Interest, Poll, Token};
use std::{
    io::{Error, ErrorKind},
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

#[derive(Debug)]
pub struct MyDisplay(*mut xlib::Display);
unsafe impl Sync for MyDisplay {}
unsafe impl Send for MyDisplay {}

lazy_static::lazy_static! {
    static ref GRAB_KEY_EVENT_SENDER: Arc<Mutex<Option<Sender<GrabEvent>>>> = Arc::new(Mutex::new(None));
    static ref GRAB_CONTROL_SENDER: Arc<Mutex<Option<Sender<GrabEvent>>>> = Arc::new(Mutex::new(None));
}

const KEYPRESS_EVENT: i32 = 2;

static mut GLOBAL_CALLBACK: Option<Box<dyn FnMut(Event) -> Option<Event>>> = None;
const GRAB_RECV: Token = Token(0);

pub enum GrabEvent {
    Grab,
    UnGrab,
    Exit,
    KeyEvent(Event),
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

fn start_grab_service() -> Result<(), GrabError> {
    let (send, recv) = std::sync::mpsc::channel::<GrabEvent>();
    *GRAB_KEY_EVENT_SENDER.lock().unwrap() = Some(send);

    unsafe {
        KEYBOARD = Keyboard::new();
    }
    start_grab_thread()?;

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
                _ => {}
            }
        }
    });

    Ok(())
}

fn open_display() -> Result<*mut Display, GrabError> {
    let display = unsafe { xlib::XOpenDisplay(ptr::null()) };
    if display.is_null() {
        return Err(GrabError::MissingDisplayError);
    }
    Ok(display)
}

fn get_default_screen_number(display: &*mut xlib::Display) -> i32 {
    unsafe { xlib::XDefaultScreen(*display) }
}

fn get_screen(
    display: &*mut xlib::Display,
    screen_number: i32,
) -> Result<*mut xlib::Screen, GrabError> {
    let screen = unsafe { xlib::XScreenOfDisplay(*display, screen_number) };
    if screen.is_null() {
        return Err(GrabError::MissScreenError);
    }
    Ok(screen)
}

fn get_root_window(screen: &*mut xlib::Screen) -> Window {
    unsafe { xlib::XRootWindowOfScreen(*screen) }
}

fn listen_to_keyboard_events(display: &*mut xlib::Display, window: &Window) {
    unsafe {
        xlib::XSelectInput(*display, *window, KeyPressMask | KeyReleaseMask);
    }
}

fn grab_keyboard_events() -> Result<(*mut Display, Window), GrabError> {
    let display = open_display()?;
    let screen_number = get_default_screen_number(&display);
    let screen = get_screen(&display, screen_number)?;
    let grab_window = get_root_window(&screen);
    listen_to_keyboard_events(&display, &grab_window);

    Ok((display, grab_window))
}

fn read_x_event(x_event: &mut xlib::XEvent, display: *mut xlib::Display) {
    while (unsafe { xlib::XPending(display) }) > 0 {
        unsafe {
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
    grab_window: u64,
    exit_clone: Arc<Mutex<bool>>,
    rx: Receiver<GrabEvent>,
) {
    std::thread::spawn(move || {
        let display = display as *mut xlib::Display;
        loop {
            match rx.recv() {
                Ok(evt) => match evt {
                    GrabEvent::Exit => {
                        *exit_clone.lock().unwrap() = true;
                        break;
                    }
                    GrabEvent::Grab => {
                        grab_keys(display, grab_window);
                    }
                    GrabEvent::UnGrab => {
                        ungrab_keys(display);
                    }
                    _ => {}
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

fn start_poll_x_event(display: u64, exit: Arc<Mutex<bool>>, mut poll: Poll) {
    thread::spawn(move || {
        let display = display as *mut xlib::Display;
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
                    // to-do: Are there any errors that need to be passed?
                    log::error!("Failed to poll event, {}", e);
                    break;
                }
            }
        }
    });
}

fn create_event_loop() -> Result<(), GrabError> {
    let (display, grab_window) = grab_keyboard_events()?;
    let grab_fd = unsafe { xlib::XConnectionNumber(display) };

    let poll = Poll::new().map_err(GrabError::IoError)?;
    poll.registry()
        .register(&mut SourceFd(&grab_fd), GRAB_RECV, Interest::READABLE)
        .map_err(GrabError::IoError)?;

    let (tx, rx) = channel();
    GRAB_CONTROL_SENDER.lock().unwrap().replace(tx);

    let exit = Arc::new(Mutex::new(false));
    start_grab_control_thread(display as u64, grab_window as u64, exit.clone(), rx);
    start_poll_x_event(display as u64, exit, poll);
    Ok(())
}

fn start_grab_thread() -> Result<(), GrabError> {
    let (tx, rx) = std::sync::mpsc::channel::<Option<GrabError>>();
    let _tx_holder = tx.clone();

    thread::spawn(move || match create_event_loop() {
        Err(err) => tx.send(Some(err)).ok(),
        Ok(_) => tx.send(None).ok(),
    });
    match rx.recv_timeout(Duration::from_millis(1000)) {
        Ok(Some(err)) => Err(err),
        Ok(None) => Ok(()),
        Err(_) => {
            // Wait too long
            Err(GrabError::IoError(Error::new(
                ErrorKind::TimedOut,
                "Wait more than 1000 milliseconds",
            )))
        }
    }
}

fn send_grab_control(data: GrabEvent) {
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
    send_grab_control(GrabEvent::Grab);
}

#[inline]
pub fn disable_grab() {
    send_grab_control(GrabEvent::UnGrab);
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
    if let Some(tx) = GRAB_KEY_EVENT_SENDER.lock().unwrap().as_ref() {
        tx.send(GrabEvent::Exit).ok();
    }
    send_grab_control(GrabEvent::Exit);
}
