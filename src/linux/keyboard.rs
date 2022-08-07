extern crate x11;
use crate::linux::keycodes::code_from_key;
use crate::rdev::{EventType, KeyboardState};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uint, c_ulong, c_void};
use std::ptr::{null, null_mut, NonNull};
use x11::xlib::{self, XKeysymToString};

#[derive(Debug)]
struct State {
    alt: bool,
    alt_gr: bool,
    ctrl: bool,
    caps_lock: bool,
    shift: bool,
    meta: bool,
    raw: u16,
}

// Inspired from https://github.com/wavexx/screenkey
// But without remitting events to custom windows, instead we recreate  XKeyEvent
// from xEvent data received via xrecord.
// Other source of inspiration https://gist.github.com/baines/5a49f1334281b2685af5dcae81a6fa8a
// Needed xproto crate as x11 does not implement _xevent.
impl State {
    fn new() -> State {
        State {
            alt: false,
            alt_gr: false,
            ctrl: false,
            caps_lock: false,
            meta: false,
            shift: false,
            raw: 0,
        }
    }

    fn value(&self) -> c_uint {
        // ignore all modiferes for name
        // note: Can't switch input method.
        let mut res: c_uint = 0;
        if self.alt {
            res += xlib::Mod1Mask;
        }
        if self.alt_gr {
            res += xlib::Mod5Mask;
        }
        if self.ctrl {
            res += xlib::ControlMask;
        }
        if self.caps_lock {
            res += xlib::LockMask;
        }
        if self.meta {
            res += xlib::Mod4Mask;
        }
        if self.shift {
            res += xlib::ShiftMask;
        }
        res
    }
}

#[derive(Debug)]
pub struct MyXIM(xlib::XIM);
unsafe impl Sync for MyXIM {}
unsafe impl Send for MyXIM {}

#[derive(Debug)]
pub struct MyXIC(xlib::XIC);
unsafe impl Sync for MyXIC {}
unsafe impl Send for MyXIC {}

#[derive(Debug)]
pub struct MyDisplay(*mut xlib::Display);
unsafe impl Sync for MyDisplay {}
unsafe impl Send for MyDisplay {}

#[derive(Debug)]
pub struct Keyboard {
    pub xim: Box<MyXIM>,
    pub xic: Box<MyXIC>,
    pub display: Box<MyDisplay>,
    window: Box<xlib::Window>,
    keysym: Box<c_ulong>,
    status: Box<i32>,
    state: State,
    serial: c_ulong,
}

impl Drop for Keyboard {
    fn drop(&mut self) {
        unsafe {
            let MyDisplay(display) = *self.display;
            xlib::XCloseDisplay(display);
        }
    }
}

impl Keyboard {
    pub fn new() -> Option<Keyboard> {
        unsafe {
            let dpy = xlib::XOpenDisplay(null());
            if dpy.is_null() {
                return None;
            }

            let string = CString::new("").expect("Can't creat CString");
            libc::setlocale(libc::LC_ALL, string.as_ptr());
            // https://stackoverflow.com/questions/18246848/get-utf-8-input-with-x11-display#
            let string = CString::new("@im=none").expect("Can't creat CString");
            let ret = xlib::XSetLocaleModifiers(string.as_ptr());
            NonNull::new(ret)?;

            let xim = xlib::XOpenIM(dpy, null_mut(), null_mut(), null_mut());
            NonNull::new(xim)?;

            let mut win_attr = xlib::XSetWindowAttributes {
                background_pixel: 0,
                background_pixmap: 0,
                border_pixel: 0,
                border_pixmap: 0,
                bit_gravity: 0,
                win_gravity: 0,
                backing_store: 0,
                backing_planes: 0,
                backing_pixel: 0,
                event_mask: 0,
                save_under: 0,
                do_not_propagate_mask: 0,
                override_redirect: 0,
                colormap: 0,
                cursor: 0,
            };

            let window = xlib::XCreateWindow(
                dpy,
                xlib::XDefaultRootWindow(dpy),
                0,
                0,
                1,
                1,
                0,
                xlib::CopyFromParent,
                xlib::InputOnly as c_uint,
                null_mut(),
                xlib::CWOverrideRedirect,
                &mut win_attr,
            );

            let input_style = CString::new(xlib::XNInputStyle).expect("CString::new failed");
            let window_client = CString::new(xlib::XNClientWindow).expect("CString::new failed");
            let style = xlib::XIMPreeditNothing | xlib::XIMStatusNothing;

            let xic = xlib::XCreateIC(
                xim,
                input_style.as_ptr(),
                style,
                window_client.as_ptr(),
                window,
                null::<c_void>(),
            );
            NonNull::new(xic)?;

            xlib::XSetICFocus(xic);

            Some(Keyboard {
                xim: Box::new(MyXIM(xim)),
                xic: Box::new(MyXIC(xic)),
                display: Box::new(MyDisplay(dpy)),
                window: Box::new(window),
                keysym: Box::new(0),
                status: Box::new(0),
                state: State::new(),
                serial: 0,
            })
        }
    }

    pub(crate) unsafe fn get_current_modifiers(&mut self) -> Option<u32> {
        let MyDisplay(display) = *self.display;
        let screen_number = xlib::XDefaultScreen(display);
        let screen = xlib::XScreenOfDisplay(display, screen_number);
        let window = xlib::XRootWindowOfScreen(screen);
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
            window,
            &mut root_return,
            &mut child_return,
            &mut root_x_return,
            &mut root_y_return,
            &mut win_x_return,
            &mut win_y_return,
            &mut mask_return,
        );
        Some(mask_return)
    }

    pub(crate) unsafe fn name_from_code(
        &mut self,
        keycode: c_uint,
        state: c_uint,
    ) -> Option<String> {
        let MyDisplay(display) = *self.display;
        let MyXIC(xic) = *self.xic;
        if display.is_null() || xic.is_null() {
            println!("We don't seem to have a display or a xic");
            return None;
        }
        const BUF_LEN: usize = 4;
        let mut buf = [0_u8; BUF_LEN];
        let MyDisplay(display) = *self.display;
        let key = xlib::XKeyEvent {
            display,
            root: 0,
            window: *self.window,
            subwindow: 0,
            x: 0,
            y: 0,
            x_root: 0,
            y_root: 0,
            state,
            keycode,
            same_screen: 0,
            send_event: 0,
            serial: self.serial,
            type_: xlib::KeyPress,
            time: xlib::CurrentTime,
        };
        self.serial += 1;

        let mut event = xlib::XEvent { key };

        // -----------------------------------------------------------------
        // XXX: This is **OMEGA IMPORTANT** This is what enables us to receive
        // the correct keyvalue from the utf8LookupString !!
        // https://stackoverflow.com/questions/18246848/get-utf-8-input-with-x11-display#
        // -----------------------------------------------------------------
        xlib::XFilterEvent(&mut event, 0);

        let MyXIC(xic) = *self.xic;
        let ret = xlib::Xutf8LookupString(
            xic,
            &mut event.key,
            buf.as_mut_ptr() as *mut c_char,
            BUF_LEN as c_int,
            &mut *self.keysym,
            &mut *self.status,
        );
        if ret == xlib::NoSymbol {
            return None;
        }

        let len = buf.iter().position(|ch| ch == &0).unwrap_or(BUF_LEN);

        // C0 controls
        if len == 1
            && matches!(
                String::from_utf8(buf[..len].to_vec())
                    .unwrap()
                    .chars()
                    .next()
                    .unwrap(),
                '\u{1}'..='\u{1f}'
            )
        {
            return None;
        }

        String::from_utf8(buf[..len].to_vec()).ok()
    }

    pub fn is_dead(&mut self) -> bool {
        unsafe {
            CStr::from_ptr(XKeysymToString(*self.keysym))
                .to_str()
                .unwrap_or_default()
                .to_owned()
                .starts_with("dead")
        }
    }
}

impl KeyboardState for Keyboard {
    fn add(&mut self, event_type: &EventType) -> Option<String> {
        match event_type {
            EventType::KeyPress(key) => {
                let keycode = code_from_key(*key)?;
                // let state = self.state.value();
                let state = unsafe { self.get_current_modifiers().unwrap_or_default() };
                // !!!: Igore Control
                let state = state & 0xFFFB;
                unsafe { self.name_from_code(keycode, state) }
            }
            EventType::KeyRelease(_key) => None,
            _ => None,
        }
    }
    fn reset(&mut self) {
        self.state = State::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    /// If the following tests run, they *will* cause a crash because xlib
    /// is *not* thread safe. Ignoring the tests for now.
    /// XCB *could* be an option but not even sure we can get dead keys again.
    /// XCB doc is sparse on the web let's say.
    fn test_thread_safety() {
        let mut keyboard = Keyboard::new().unwrap();
        let char_s = keyboard.add(&EventType::KeyPress(crate::rdev::Key::KeyS)).unwrap();
        assert_eq!(
            char_s,
            "s".to_string(),
            "This test should pass only on Qwerty layout !"
        );
    }

    #[test]
    #[ignore]
    fn test_thread_safety_2() {
        let mut keyboard = Keyboard::new().unwrap();
        let char_s = keyboard.add(&EventType::KeyPress(crate::rdev::Key::KeyS)).unwrap();
        assert_eq!(
            char_s,
            "s".to_string(),
            "This test should pass only on Qwerty layout !"
        );
    }
}
