use crate::rdev::Key;
use std::convert::TryInto;

macro_rules! decl_keycodes {
    ($($key:ident, $code:literal, $scancode:literal),*) => {
        //TODO: make const when rust lang issue #49146 is fixed
        pub fn code_from_key(key: Key) -> Option<u32> {
            match key {
                $(
                    Key::$key => Some($code),
                )*
                Key::Unknown(code) => Some(code.try_into().ok()?),
                _ => None,
            }
        }

        //TODO: make const when rust lang issue #49146 is fixed
        pub fn key_from_code(code: u32) -> Key {
            match code {
                $(
                    $code => Key::$key,
                )*
                _ => Key::Unknown(code.into())
            }
        }

        pub fn scancode_from_key(key: Key) -> Option<u32> {
            match key {
                $(
                    Key::$key => Some($scancode),
                )*
                Key::Unknown(code) => Some(code.try_into().ok()?),
                _ => None,
            }
        }

        pub fn key_from_scancode(scancode: u32) -> Key{
            match scancode {
                0 => Key::Unknown(0),
                $(
                    $scancode => Key::$key,
                )*
                _ => Key::Unknown(scancode.into())
            }
        }
    };
}

// TODO: 0

// https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
// We redefined here for Letter and number keys which are not in winapi crate (and don't have a name either in win32)
decl_keycodes! {
    Alt, 164, 56,
    AltGr, 165, 56,
    Backspace, 0x08, 14,
    CapsLock, 20, 58,
    ControlLeft, 162, 29,
    ControlRight, 163, 29,
    Delete, 46, 0,
    UpArrow, 38, 0,
    DownArrow, 40, 0,
    LeftArrow, 37, 0,
    RightArrow, 39, 0,
    End, 35, 0,
    Escape, 27, 1,
    F1, 112, 59,
    F2, 113, 60,
    F3, 114, 61,
    F4, 115, 62,
    F5, 116, 63,
    F6, 117, 64,
    F7, 118, 65,
    F8, 119, 66,
    F9, 120, 67,
    F10, 121, 68,
    F11, 122, 87,
    F12, 123, 88,
    Home, 36, 0,
    MetaLeft, 91, 91,
    PageDown, 34, 0,
    PageUp, 33, 0,
    Return, 13, 28,
    ShiftLeft, 160, 42,
    ShiftRight, 161, 54,
    Space, 32, 57,
    Tab, 0x09, 15,
    PrintScreen, 44, 0,
    ScrollLock, 145, 70,
    NumLock, 144, 0,
    BackQuote, 192, 41,
    Num1, 49, 2,
    Num2, 50, 3,
    Num3, 51, 4,
    Num4, 52, 5,
    Num5, 53, 6,
    Num6, 54, 7,
    Num7, 55, 8,
    Num8, 56, 9,
    Num9, 57, 10,
    Num0, 48, 11,
    Minus, 189, 12,
    Equal, 187, 13,
    KeyQ, 81, 16,
    KeyW, 87, 17,
    KeyE, 69, 18,
    KeyR, 82, 19,
    KeyT, 84, 20,
    KeyY, 89, 21,
    KeyU, 85, 22,
    KeyI, 73, 23,
    KeyO, 79, 24,
    KeyP, 80, 25,
    LeftBracket, 219, 26,
    RightBracket, 221, 27,
    BackSlash, 220, 43,
    KeyA, 65, 30,
    KeyS, 83, 31,
    KeyD, 68, 32,
    KeyF, 70, 33,
    KeyG, 71, 34,
    KeyH, 72, 35,
    KeyJ, 74, 36,
    KeyK, 75, 37,
    KeyL, 76, 38,
    SemiColon, 186,  39,
    Quote, 222, 40 ,
    IntlBackslash, 226, 43,
    KeyZ, 90, 44,
    KeyX, 88, 45,
    KeyC, 67, 46,
    KeyV, 86, 47,
    KeyB, 66, 48,
    KeyN, 78, 49,
    KeyM, 77, 50,
    Comma, 188, 51,
    Dot, 190, 52,
    Slash, 191, 53,
    Insert, 45, 0,
    KpMinus, 109, 0,
    KpPlus, 107, 0,
    KpMultiply, 106, 0,
    KpDivide, 111, 0,
    KpDecimal, 110, 0,
    Kp0, 96, 0,
    Kp1, 97, 0,
    Kp2, 98, 0,
    Kp3, 99, 0,
    Kp4, 100, 0,
    Kp5, 101, 0,
    Kp6, 102, 0,
    Kp7, 103, 0,
    Kp8, 104, 0,
    Kp9, 105, 0,
    MetaRight, 92, 0,
    Apps, 93, 93,
    Cancel, 0x03, 0,
    Clear, 12, 0,
    Kana, 0x15, 0,
    Junja, 0x17, 0,
    Final, 0x18, 0,
    Hanja, 0x19, 0,
    Convert, 0x1C, 0,
    Select, 0x29, 0,
    Print, 0x2A, 0,
    Execute, 0x2B, 0,
    Help, 0x2F, 0,
    Sleep, 0x5F, 0,
    Separator, 0x6C, 0,
    Pause, 19, 0
}

#[cfg(test)]
mod test {
    use super::{code_from_key, key_from_code};
    #[test]
    fn test_reversible() {
        for code in 0..65535 {
            let key = key_from_code(code);
            if let Some(code2) = code_from_key(key) {
                assert_eq!(code, code2)
            } else {
                assert!(false, "We could not convert back code: {:?}", code);
            }
        }
    }
}
