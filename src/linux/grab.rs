// This code is awful. Good luck
use crate::{
    key_from_scancode,
    linux_keycode_from_key,
    Event,
    EventType,
    GrabError,
    Key as RdevKey,
};
use mio::unix::SourceFd;
use mio::{ Events, Interest, Poll, Token };
use std::os::unix::net::UnixDatagram;
use std::os::unix::prelude::AsRawFd;
use std::path::Path;
use std::ptr;
use std::thread;
use std::time::Duration;
use std::{ mem::zeroed, os::raw::c_int, sync::{ mpsc::Sender, Arc, Mutex }, time::SystemTime };
use x11::xlib::{ self, Display, GrabModeAsync, KeyPressMask, KeyReleaseMask };

#[derive(Debug)]
pub struct MyDisplay(*mut xlib::Display);
unsafe impl Sync for MyDisplay {}
unsafe impl Send for MyDisplay {}

lazy_static::lazy_static! {
    pub static ref SENDER: Arc<Mutex<Option<Sender<GrabEvent>>>> = Arc::new(Mutex::new(None));
}

const KEYPRESS_EVENT: i32 = 2;

pub static mut GLOBAL_CALLBACK: Option<Box<dyn FnMut(Event) -> Option<Event>>> = None;
pub static FILE_PATH: &str = "/tmp/rdev_service.sock";
const GRAB_RECV: Token = Token(0);
const SERVICE_RECV: Token = Token(1);
type Window = u64;

pub enum GrabEvent {
    Grab,
    UnGrab,
    Exit,
    KeyEvent(Event),
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

fn grab_keys(display: *mut Display, grab_window: libc::c_ulong) {
    unsafe {
        xlib::XGrabKeyboard(
            display,
            grab_window,
            c_int::from(true),
            GrabModeAsync,
            GrabModeAsync,
            xlib::CurrentTime
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

fn send_to_client(grab: bool) {
    let socket = UnixDatagram::unbound().unwrap();
    if socket.connect(FILE_PATH).is_ok() {
        let message = if grab { b"1" } else { b"0" };
        socket.send(message).expect("[-] grab error in rdev: recv function failed");
    }
}

fn start_grab_service() -> Result<(), GrabError> {
    let (send, recv) = std::sync::mpsc::channel::<GrabEvent>();
    *SENDER.lock().unwrap() = Some(send);

    start_grab_thread()?;

    thread::spawn(move || {
        loop {
            if let Ok(data) = recv.recv() {
                match data {
                    GrabEvent::Grab => {
                        send_to_client(true);
                    }
                    GrabEvent::UnGrab => {
                        send_to_client(false);
                    }
                    GrabEvent::KeyEvent(event) => unsafe {
                        if let Some(callback) = &mut GLOBAL_CALLBACK {
                            callback(event);
                        }
                    }
                    GrabEvent::Exit => {
                        break;
                    }
                }
            }
        }
    });

    Ok(())
}

fn unlink_socket(path: impl AsRef<Path>) -> Result<(), GrabError> {
    let path = path.as_ref();
    if Path::new(path).exists() {
        std::fs::remove_file(path).map_err(GrabError::IoError)?;
    }
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
    screen_number: i32
) -> Result<*mut xlib::Screen, GrabError> {
    let screen = unsafe { xlib::XScreenOfDisplay(*display, screen_number) };
    if screen.is_null() {
        return Err(GrabError::MissScreenError);
    }
    Ok(screen)
}

fn get_root_window(screen: &*mut xlib::Screen) -> xlib::Window {
    unsafe { xlib::XRootWindowOfScreen(*screen) }
}

fn listen_to_keyboard_events(display: &*mut xlib::Display, window: &xlib::Window) {
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

fn get_grab_fd(display: *mut xlib::Display) -> i32 {
    unsafe { xlib::XConnectionNumber(display) }
}

fn get_socket() -> Result<UnixDatagram, GrabError> {
    unlink_socket(FILE_PATH)?;
    let socket = UnixDatagram::bind(FILE_PATH).map_err(GrabError::IoError)?;
    socket.set_nonblocking(true).ok();
    Ok(socket)
}

fn create_poll_instance() -> Result<Poll, GrabError> {
    Ok(Poll::new().map_err(GrabError::IoError).ok().unwrap())
}

fn poll_register_fd(
    poll: &Poll,
    source_fd: i32,
    token: Token,
    interests: Interest
) -> Result<(), GrabError> {
    poll
        .registry()
        .register(&mut SourceFd(&source_fd), token, interests)
        .map_err(GrabError::IoError)?;
    Ok(())
}

fn change_grab_state(
    socket: &UnixDatagram,
    display: *mut xlib::Display,
    grab_window: Window
) -> Result<(), GrabError> {
    // if recv "1"=> Ascii(49): grab key
    let mut buf = [0; 1];
    socket.recv(buf.as_mut_slice()).map_err(GrabError::IoError)?;
    if buf[0] == 49 {
        grab_keys(display, grab_window);
    } else {
        ungrab_keys(display);
    }
    Ok(())
}

fn read_x_event(x_event: &mut xlib::XEvent, display: *mut xlib::Display) {
    while (unsafe { xlib::XPending(display) }) > 0 {
        unsafe {
            xlib::XNextEvent(display, x_event);
        }
        let keycode = unsafe { x_event.key.keycode };
        let event_type = unsafe { x_event.type_ };

        let key = key_from_scancode(keycode);
        let is_press = event_type == KEYPRESS_EVENT;
        let event = convert_event(key, is_press);
        if let Some(tx) = &*SENDER.lock().unwrap() {
            tx.send(GrabEvent::KeyEvent(event)).ok();
        }
    }
}

fn create_event_loop() -> Result<(), GrabError> {
    let (display, grab_window) = grab_keyboard_events()?;
    let grab_fd = get_grab_fd(display);
    let socket = get_socket()?;
    let socket_fd = socket.as_raw_fd();

    let mut poll = create_poll_instance()?;
    poll_register_fd(&poll, grab_fd, GRAB_RECV, Interest::READABLE)?;
    poll_register_fd(&poll, socket_fd, SERVICE_RECV, Interest::READABLE)?;

    let mut events = Events::with_capacity(128);
    let mut x_event: xlib::XEvent = unsafe { zeroed() };

    loop {
        poll.poll(&mut events, None).map_err(GrabError::IoError)?;
        for event in &events {
            match event.token() {
                SERVICE_RECV => {
                    change_grab_state(&socket, display, grab_window)?;
                }
                GRAB_RECV => {
                    read_x_event(&mut x_event, display);
                }
                _ => {}
            }
        }
    }
}

fn start_grab_thread() -> Result<(), GrabError> {
    let (error_sender, error_recv) = std::sync::mpsc::channel::<GrabError>();

    thread::spawn(move || {
        if let Err(err) = create_event_loop() {
            error_sender.send(err).ok();
        }
    });
    thread::sleep(Duration::from_millis(100));
    if let Ok(err) = error_recv.try_recv() {
        Err(err)
    } else {
        Ok(())
    }
}

pub fn enable_grab() {
    if let Some(tx) = &*SENDER.lock().unwrap() {
        tx.send(GrabEvent::Grab).ok();
        /* if too fast: poll cannot perceive events */
        thread::sleep(Duration::from_millis(50));
    }
}

pub fn disable_grab() {
    if let Some(tx) = &*SENDER.lock().unwrap() {
        tx.send(GrabEvent::UnGrab).ok();
        thread::sleep(Duration::from_millis(50));
    }
}

pub fn start_grab_listen<T>(callback: T) -> Result<(), GrabError>
    where T: FnMut(Event) -> Option<Event> + 'static
{
    unsafe {
        GLOBAL_CALLBACK = Some(Box::new(callback));
    }
    start_grab_service()?;
    thread::sleep(Duration::from_millis(100));
    Ok(())
}

pub fn exit_grab_listen() {
    if let Some(tx) = &*SENDER.lock().unwrap() {
        tx.send(GrabEvent::Exit).ok();
    }
}