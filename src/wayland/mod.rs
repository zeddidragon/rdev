mod keycodes;
#[cfg(target_os = "linux")]
mod simulate;

pub use keycodes::*;
#[cfg(target_os = "linux")]
pub use simulate::*;