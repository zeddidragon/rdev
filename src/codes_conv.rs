use crate::Key;
use crate::linux::{code_from_key as linux_code_from_keycode, key_from_code as linux_keycode_from_code};
use crate::macos::{code_from_key as macos_code_from_keycode, key_from_code as macos_keycode_from_code};
use crate::windows::{
    key_from_scancode as win_key_from_scancode, scancode_from_key as win_scancode_from_key,
};

macro_rules! conv_keycodes {
    ($fnname:ident, $key_from_code:ident, $code_from_key:ident) => {
        pub fn $fnname(code: u32) -> Option<u32> {
            let key = $key_from_code(code);
            match key {
                Key::Unknown(..) => None,
                Key::RawKey(..) => None,
                _ => $code_from_key(key),
            }
        }
    };
}

conv_keycodes!(
    win_scancode_to_linux_code,
    win_key_from_scancode,
    linux_code_from_keycode
);
conv_keycodes!(
    win_scancode_to_macos_code,
    win_key_from_scancode,
    macos_code_from_keycode
);
conv_keycodes!(
    linux_code_to_win_scancode,
    linux_keycode_from_code,
    win_scancode_from_key
);
conv_keycodes!(
    linux_code_to_macos_code,
    linux_keycode_from_code,
    macos_code_from_keycode
);
conv_keycodes!(
    macos_code_to_win_scancode,
    macos_keycode_from_code,
    win_scancode_from_key
);
conv_keycodes!(
    macos_code_to_linux_code,
    macos_keycode_from_code,
    linux_code_from_keycode
);
