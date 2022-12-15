use crate::rdev::Key;

macro_rules! decl_keycodes {
    ($($key:ident, $code:literal),*) => {
        //TODO: make const when rust lang issue #49146 is fixed
        pub fn code_from_key(key: Key) -> Option<u32> {
            match key {
                $(
                    Key::$key => Some($code),
                )*
                Key::Unknown(code) => Some(code),
                _ => None,
            }
        }

        //TODO: make const when rust lang issue #49146 is fixed
        #[allow(dead_code)]
        pub fn key_from_code(code: u32) -> Key {
            match code {
                $(
                    $code => Key::$key,
                )*
                _ => Key::Unknown(code)
            }
        }
    };
}

#[rustfmt::skip]
decl_keycodes!(
    Alt, 56,
    AltGr, 100,
    Backspace, 14,
    CapsLock, 58,
    ControlLeft, 29,
    ControlRight, 97,
    Delete, 111,
    DownArrow, 108,
    End, 107,
    Escape, 1,
    F1, 59,
    F10, 68,
    F11, 87,
    F12, 88,
    F2, 60,
    F3, 61,
    F4, 62,
    F5, 63,
    F6, 64,
    F7, 65,
    F8, 66,
    F9, 67,
    Home, 102,
    LeftArrow, 105,
    MetaLeft, 125,
    PageDown, 109,
    PageUp, 104,
    Return, 28,
    RightArrow, 106,
    ShiftLeft, 42,
    ShiftRight, 54,
    Space, 57,
    Tab, 15,
    UpArrow, 103,
    PrintScreen, 99,
    ScrollLock, 70,
    Pause, 119,
    NumLock, 69,
    BackQuote, 41,
    Num1, 2,
    Num2, 3,
    Num3, 4,
    Num4, 5,
    Num5, 6,
    Num6, 7,
    Num7, 8,
    Num8, 9,
    Num9, 10,
    Num0, 11,
    Minus, 12,
    Equal, 13,
    KeyQ, 16,
    KeyW, 17,
    KeyE, 18,
    KeyR, 19,
    KeyT, 20,
    KeyY, 21,
    KeyU, 22,
    KeyI, 23,
    KeyO, 24,
    KeyP, 25,
    LeftBracket, 26,
    RightBracket, 27,
    KeyA, 30,
    KeyS, 31,
    KeyD, 32,
    KeyF, 33,
    KeyG, 34,
    KeyH, 35,
    KeyJ, 36,
    KeyK, 37,
    KeyL, 38,
    SemiColon, 39,
    Quote, 40,
    BackSlash, 43,
    IntlBackslash, 86,
    KeyZ, 44,
    KeyX, 45,
    KeyC, 46,
    KeyV, 47,
    KeyB, 48,
    KeyN, 49,
    KeyM, 50,
    Comma, 51,
    Dot, 52,
    Slash, 53,
    Insert, 110,
    KpDecimal, 83,
    KpReturn, 96,
    KpMinus, 74,
    KpPlus, 78,
    KpMultiply, 55,
    KpDivide, 98,
    Kp0, 82,
    Kp1, 79,
    Kp2, 80,
    Kp3, 81,
    Kp4, 75,
    Kp5, 76,
    Kp6, 77,
    Kp7, 71,
    Kp8, 72,
    Kp9, 73,
    MetaRight, 126,
    Apps, 127
);

#[cfg(test)]
mod test {
    use super::{code_from_key, key_from_code};
    #[test]
    fn test_reversible() {
        for code in 0..65636 {
            let key = key_from_code(code);
            match code_from_key(key) {
                Some(code2) => assert_eq!(code, code2),
                None => panic!("Could not convert back code: {:?}", code),
            }
        }
    }
}
