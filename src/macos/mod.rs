#[cfg(target_os = "macos")]
mod common;
#[cfg(target_os = "macos")]
mod display;
#[cfg(target_os = "macos")]
#[cfg(feature = "unstable_grab")]
mod grab;
#[cfg(target_os = "macos")]
mod keyboard;
#[cfg(target_os = "macos")]
mod listen;
#[cfg(target_os = "macos")]
mod simulate;
mod keycodes;

#[cfg(target_os = "macos")]
pub use crate::macos::display::display_size;
#[cfg(target_os = "macos")]
#[cfg(feature = "unstable_grab")]
pub use crate::macos::grab::grab;
#[cfg(target_os = "macos")]
pub use crate::macos::keyboard::Keyboard;
#[cfg(target_os = "macos")]
pub use crate::macos::listen::listen;
#[cfg(target_os = "macos")]
pub use crate::macos::simulate::simulate;
pub use crate::macos::keycodes::*;