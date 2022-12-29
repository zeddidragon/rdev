use super::virtual_keycodes::{
    kVK_End, kVK_ForwardDelete, kVK_Help, kVK_Home, kVK_PageDown, kVK_PageUp,
};
use crate::macos::keycodes::code_from_key;
use crate::rdev::{Button, EventType, RawKey, SimulateError};
use core_graphics::{
    event::{
        CGEvent, CGEventFlags, CGEventTapLocation, CGEventType, CGMouseButton, ScrollEventUnit,
    },
    event_source::{CGEventSource, CGEventSourceStateID},
    geometry::CGPoint,
};
use std::convert::TryInto;

#[allow(non_upper_case_globals)]
fn workaround_fn(event: CGEvent, keycode: u32) -> CGEvent {
    match keycode {
        kVK_Help | kVK_ForwardDelete | kVK_Home | kVK_End | kVK_PageDown | kVK_PageUp => {
            let flags = event.get_flags();
            event.set_flags(flags & (!(CGEventFlags::CGEventFlagSecondaryFn)));
        }
        _ => {}
    }
    event
}

unsafe fn convert_native_with_source(
    event_type: &EventType,
    source: CGEventSource,
) -> Option<CGEvent> {
    match event_type {
        EventType::KeyPress(key) => match key {
            crate::Key::RawKey(rawkey) => {
                if let RawKey::MacVirtualKeycode(keycode) = rawkey {
                    CGEvent::new_keyboard_event(source, *keycode as _, true)
                        .and_then(|event| Ok(workaround_fn(event, *keycode)))
                        .ok()
                } else {
                    None
                }
            }
            _ => {
                let code = code_from_key(*key)?;
                CGEvent::new_keyboard_event(source, code as _, true)
                    .and_then(|event| Ok(workaround_fn(event, code as _)))
                    .ok()
            }
        },
        EventType::KeyRelease(key) => match key {
            crate::Key::RawKey(rawkey) => {
                if let RawKey::MacVirtualKeycode(keycode) = rawkey {
                    CGEvent::new_keyboard_event(source, *keycode as _, false)
                        .and_then(|event| Ok(workaround_fn(event, *keycode)))
                        .ok()
                } else {
                    None
                }
            }
            _ => {
                let code = code_from_key(*key)?;
                CGEvent::new_keyboard_event(source, code as _, false)
                    .and_then(|event| Ok(workaround_fn(event, code as _)))
                    .ok()
            }
        },
        EventType::ButtonPress(button) => {
            let point = get_current_mouse_location()?;
            let event = match button {
                Button::Left => CGEventType::LeftMouseDown,
                Button::Right => CGEventType::RightMouseDown,
                _ => return None,
            };
            CGEvent::new_mouse_event(
                source,
                event,
                point,
                CGMouseButton::Left, // ignored because we don't use OtherMouse EventType
            )
            .ok()
        }
        EventType::ButtonRelease(button) => {
            let point = get_current_mouse_location()?;
            let event = match button {
                Button::Left => CGEventType::LeftMouseUp,
                Button::Right => CGEventType::RightMouseUp,
                _ => return None,
            };
            CGEvent::new_mouse_event(
                source,
                event,
                point,
                CGMouseButton::Left, // ignored because we don't use OtherMouse EventType
            )
            .ok()
        }
        EventType::MouseMove { x, y } => {
            let point = CGPoint { x: (*x), y: (*y) };
            CGEvent::new_mouse_event(source, CGEventType::MouseMoved, point, CGMouseButton::Left)
                .ok()
        }
        EventType::Wheel { delta_x, delta_y } => {
            let wheel_count = 2;
            CGEvent::new_scroll_event(
                source,
                ScrollEventUnit::PIXEL,
                wheel_count,
                (*delta_y).try_into().ok()?,
                (*delta_x).try_into().ok()?,
                0,
            )
            .ok()
        }
    }
}

unsafe fn convert_native(event_type: &EventType) -> Option<CGEvent> {
    // https://developer.apple.com/documentation/coregraphics/cgeventsourcestateid#:~:text=kCGEventSourceStatePrivate
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).ok()?;
    convert_native_with_source(event_type, source)
}

unsafe fn get_current_mouse_location() -> Option<CGPoint> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).ok()?;
    let event = CGEvent::new(source).ok()?;
    Some(event.location())
}

pub fn simulate(event_type: &EventType) -> Result<(), SimulateError> {
    unsafe {
        if let Some(cg_event) = convert_native(event_type) {
            cg_event.post(CGEventTapLocation::HID);
            Ok(())
        } else {
            Err(SimulateError)
        }
    }
}
