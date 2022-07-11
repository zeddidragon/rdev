#[cfg(target_os = "windows")]
extern crate winapi;
#[cfg(target_os = "windows")]
mod common;
#[cfg(target_os = "windows")]
mod display;
#[cfg(target_os = "windows")]
#[cfg(feature = "unstable_grab")]
mod grab;
#[cfg(target_os = "windows")]
mod keyboard;
mod keycodes;
#[cfg(target_os = "windows")]
mod listen;
#[cfg(target_os = "windows")]
mod simulate;

#[cfg(target_os = "windows")]
pub use crate::windows::common::*;
#[cfg(target_os = "windows")]
pub use crate::windows::display::display_size;
#[cfg(target_os = "windows")]
#[cfg(feature = "unstable_grab")]
pub use crate::windows::grab::grab;
#[cfg(target_os = "windows")]
pub use crate::windows::keyboard::Keyboard;
pub use crate::windows::keycodes::*;
#[cfg(target_os = "windows")]
pub use crate::windows::listen::listen;
#[cfg(target_os = "windows")]
pub use crate::windows::simulate::*;
