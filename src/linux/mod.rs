#[cfg(target_os = "linux")]
extern crate libc;
#[cfg(target_os = "linux")]
extern crate x11;
#[cfg(target_os = "linux")]
mod common;
#[cfg(target_os = "linux")]
mod display;
#[cfg(target_os = "linux")]
mod grab;
#[cfg(target_os = "linux")]
mod keyboard;
#[cfg(target_os = "linux")]
mod listen;
#[cfg(target_os = "linux")]
mod simulate;
mod keycodes;

#[cfg(target_os = "linux")]
pub use crate::linux::display::display_size;
#[cfg(target_os = "linux")]
pub use crate::linux::grab::grab;
#[cfg(target_os = "linux")]
pub use crate::linux::keyboard::Keyboard;
#[cfg(target_os = "linux")]
pub use crate::linux::listen::listen;
#[cfg(target_os = "linux")]
pub use crate::linux::simulate::simulate;
pub use crate::linux::keycodes::*;
