use enum_map::MaybeUninit;
#[cfg(target_os = "linux")]
use libc::{c_char, c_int, c_ulong, setlocale, LC_ALL};
#[cfg(target_os = "windows")]
use rdev::get_win_key;
use rdev::{Event, EventType::*, Key as RdevKey};
use std::ffi::CString;
use std::ptr::{null_mut, NonNull};
use std::sync::Mutex;
use std::{collections::HashMap, ptr::null};
#[cfg(target_os = "linux")]
use x11::xlib::{
    self, ButtonPressMask, KeyPressMask, KeyReleaseMask, StructureNotifyMask, XBufferOverflow,
    XCreateIC, XCreateSimpleWindow, XDefaultRootWindow, XFilterEvent, XIMPreeditNothing,
    XIMStatusNothing, XLookupBoth, XLookupKeySym, XMapWindow, XNClientWindow, XNInputStyle,
    XNextEvent, XOpenDisplay, XOpenIM, XSelectInput, XSetICFocus, XSetLocaleModifiers,
    Xutf8LookupString,
};

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

#[cfg(target_os = "linux")]
#[warn(dead_code)]
fn listen_for_char() -> Option<()> {
    unsafe {
        let display = XOpenDisplay(null());
        if display.is_null() {
            panic!("cannot open display");
        }

        // Fix modifiers
        setlocale(LC_ALL, CString::new("").unwrap().as_ptr());
        XSetLocaleModifiers(CString::new("@im=none").unwrap().as_ptr());

        // create windows to get Window ID.
        let width = 400;
        let height = 2200;
        let win = XCreateSimpleWindow(
            display,
            XDefaultRootWindow(display), /* display, parent */
            0,
            0, /* x, y: the window manager will place the window elsewhere */
            width,
            height, /* width, height */
            2,
            100, /* border width & colour, unless you have a window manager */
            100,
        ); /* background colour */
        println!("Windows id: {:?}", win);

        /* tell the display server what kind of events we would like to see */
        XSelectInput(
            display,
            win,
            ButtonPressMask | StructureNotifyMask | KeyPressMask | KeyReleaseMask,
        );
        /* okay, put the window on the screen, please */
        XMapWindow(display, win);

        println!("Open IM");
        let im = XOpenIM(display, null_mut(), null_mut(), null_mut());
        NonNull::new(im)?;

        println!("Create IC");
        let input_style = CString::new(xlib::XNInputStyle).expect("CString::new failed");
        let window_client = CString::new(xlib::XNClientWindow).expect("CString::new failed");
        let style = xlib::XIMPreeditNothing | xlib::XIMStatusNothing;
        let ic = XCreateIC(im, input_style, style, window_client, win, null_mut::<()>());
        NonNull::new(ic);

        println!("Select IC");
        XSetICFocus(*Box::new(ic));

        loop {
            let event = {
                let mut event = MaybeUninit::uninit();
                XNextEvent(display, event.as_mut_ptr());
                &mut event.assume_init()
            };
            if XFilterEvent(event, win) != 0 {
                continue;
            };
            match event.type_ {
                2 => {
                    // Key Down
                    let mut keysym: Box<c_ulong> = Box::new(0);

                    const BUF_LEN: usize = 20;
                    let mut buf = [0_u8; BUF_LEN];
                    let mut status: Box<i32> = Box::new(0);

                    let count = Xutf8LookupString(
                        *Box::new(ic),
                        &mut event.key,
                        buf.as_mut_ptr() as *mut c_char,
                        BUF_LEN as c_int,
                        &mut *keysym,
                        &mut *status,
                    );

                    println!("count: {:?}", count);
                    if status == Box::new(XBufferOverflow) {
                        println!("BufferOverflow\n")
                    };

                    if count != 0 {
                        let len = buf.iter().position(|ch| ch == &0).unwrap_or(BUF_LEN);
                        println!(
                            "key down -> buffer: {:?} {:?}",
                            count,
                            String::from_utf8(buf[..len].to_vec()).ok()
                        )
                    };

                    if status == Box::new(XLookupKeySym) || status == Box::new(XLookupBoth) {
                        println!("status: {:?}", status)
                    };
                    println!("pressed KEY: {:?}", keysym);
                }
                3 => {
                    // Key Up
                    println!("Key up");
                    println!("-------------------------");
                }
                _ => {}
            }
        }
    }
}

fn main() {
    // This will block.
    std::env::set_var("KEYBOARD_ONLY", "y");

    let func = move |evt: Event| {
        let (_key, _down) = match evt.event_type {
            KeyPress(k) => {
                if MUTEX_SPECIAL_KEYS.lock().unwrap().contains_key(&k) {
                    if *MUTEX_SPECIAL_KEYS.lock().unwrap().get(&k).unwrap() {
                        return;
                    }
                    MUTEX_SPECIAL_KEYS.lock().unwrap().insert(k, true);
                }
                let s = evt.name.unwrap_or_default();
                println!("keydown {:?} {:?} {:?} {:?}", k, evt.code, evt.scan_code, s);

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

        #[cfg(target_os = "windows")]
        let _key = get_win_key(evt.code.into(), evt.scan_code);
        #[cfg(target_os = "windows")]
        println!("{:?}", _key);

        let linux_keycode = rdev::linux_keycode_from_key(_key).unwrap();
        // Mac/Linux Numpad -> Windows ArrawKey
        // https://github.com/asur4s/rustdesk/blob/fe9923109092827f543560a7af42dff6c3135117/src/ui/remote.rs#L968
        let windwos_keycode = rdev::win_keycode_from_key(_key).unwrap();
        let macos_keycode = rdev::macos_keycode_from_key(_key).unwrap();
        println!("Linux keycode {:?}", linux_keycode);
        println!("Windows keycode {:?}", windwos_keycode);
        println!("Mac OS keycode {:?}", macos_keycode);

        println!("--------------");
    };
    if let Err(error) = rdev::listen(func) {
        // rdev::listen
        dbg!("{:?}", error);
    }

    // listen_for_char();
}
