use crate::rdev::Key;

macro_rules! decl_keycodes {
    ($($key:ident, $code:literal),*) => {
        //TODO: make const when rust lang issue #49146 is fixed
        pub fn code_from_key(key: Key) -> Option<u32> {
            match key {
                $(
                    Key::$key => Some($code),
                )*
                Key::Unknown(code) => Some(code as _),
                _ => None,
            }
        }

        //TODO: make const when rust lang issue #49146 is fixed
        pub fn key_from_code(code: u32) -> Key {
            match code {
                $(
                    $code => Key::$key,
                )*
                _ => Key::Unknown(code as _)
            }
        }

        pub fn scancode_from_key(key:Key) -> Option<u32> {
            // keycode
            code_from_key(key)
        }

        pub fn key_from_scancode(scancode: u32) -> Key {
            // keycode
            key_from_scancode(scancode)
        }
    };
}

#[rustfmt::skip]
decl_keycodes!(
   Alt, 58,
   AltGr, 61,
   Backspace, 51,
   CapsLock, 57,
   ControlLeft, 59,
   ControlRight, 62,
   DownArrow, 125,
   Escape, 53,
   F1, 122,
   F10, 109,
   F11, 103,
   F12, 111,
   F2, 120,
   F3, 99,
   F4, 118,
   F5, 96,
   F6, 97,
   F7, 98,
   F8, 100,
   F9, 101,
   Function, 63,
   LeftArrow, 123,
   MetaLeft, 55,
   MetaRight, 54,
   Return, 36,
   RightArrow, 124,
   ShiftLeft, 56,
   ShiftRight, 60,
   Space, 49,
   Tab, 48,
   UpArrow, 126,
   BackQuote, 50,
   Num1, 18,
   Num2, 19,
   Num3, 20,
   Num4, 21,
   Num5, 23,
   Num6, 22,
   Num7, 26,
   Num8, 28,
   Num9, 25,
   Num0, 29,
   Minus, 27,
   Equal, 24,
   KeyQ, 12,
   KeyW, 13,
   KeyE, 14,
   KeyR, 15,
   KeyT, 17,
   KeyY, 16,
   KeyU, 32,
   KeyI, 34,
   KeyO, 31,
   KeyP, 35,
   LeftBracket, 33,
   RightBracket, 30,
   KeyA, 0,
   KeyS, 1,
   KeyD, 2,
   KeyF, 3,
   KeyG, 5,
   KeyH, 4,
   KeyJ, 38,
   KeyK, 40,
   KeyL, 37,
   SemiColon, 41,
   Quote, 39,
   BackSlash, 42,
   KeyZ, 6,
   KeyX, 7,
   KeyC, 8,
   KeyV, 9,
   KeyB, 11,
   KeyN, 45,
   KeyM, 46,
   Comma, 43,
   Dot, 47,
   Slash, 44,
   Delete, 117,
   Insert, 114,
   PageUp, 116,
   PageDown, 121,
   Home, 115,
   End, 119,
   PrintScreen, 105,
   ScrollLock, 107,
   Pause, 113,
   KpReturn, 76,
   KpMinus, 78,
   KpPlus, 69,
   KpMultiply, 67,
   KpDivide, 75,
   KpDecimal, 65,
   Kp0, 82,
   Kp1, 83,
   Kp2, 84,
   Kp3, 85,
   Kp4, 86,
   Kp5, 87,
   Kp6, 88,
   Kp7, 89,
   Kp8, 91,
   Kp9, 92,
   Apps, 110,
   NumLock, 71
);

#[cfg(test)]
mod test {
    use super::{code_from_key, key_from_code};
    #[test]
    fn test_reversible() {
        for code in 0..=65535 {
            let key = key_from_code(code);
            match code_from_key(key) {
                Some(code2) => assert_eq!(code, code2),
                None => panic!("Could not convert back code: {:?}", code),
            }
        }
    }
}
