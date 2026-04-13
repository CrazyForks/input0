//! Native macOS Fn key monitor.
//!
//! The `tauri-plugin-global-shortcut` plugin (and its underlying
//! `global-hotkey` crate) cannot register the Fn key as a shortcut because
//! macOS reports Fn as a modifier flag change rather than as a regular key
//! press. To support "hold Fn to dictate", we install an `NSEvent` monitor
//! for `flagsChanged` events and dispatch press/release edges of the
//! `NSEventModifierFlagFunction` bit to our pipeline trigger handlers.
//!
//! Global and local monitors are used in tandem so the key is detected both
//! when our app is focused and when it is in the background.

#![cfg(target_os = "macos")]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use block::ConcreteBlock;
use cocoa::base::{id, nil};

use crate::commands::window::gcd_run_on_main_sync;

/// `NSEventModifierFlagFunction` — set when Fn is held.
const NS_EVENT_MODIFIER_FLAG_FUNCTION: u64 = 1 << 23;
/// `NSEventMaskFlagsChanged` — mask for modifier-flag-change events.
const NS_EVENT_MASK_FLAGS_CHANGED: u64 = 1 << 12;

type Callback = Arc<dyn Fn(bool) + Send + Sync + 'static>;

struct MonitorHandles {
    global_monitor: id,
    local_monitor: id,
}

// NSEvent retains the monitor objects internally; we store their `id`
// pointers behind a Mutex and only touch them through main-thread dispatch.
unsafe impl Send for MonitorHandles {}

static MONITORS: Mutex<Option<MonitorHandles>> = Mutex::new(None);
static FN_PRESSED: AtomicBool = AtomicBool::new(false);

pub fn start<F>(callback: F) -> Result<(), String>
where
    F: Fn(bool) + Send + Sync + 'static,
{
    let cb: Callback = Arc::new(callback);
    gcd_run_on_main_sync(move || start_on_main(cb))
}

pub fn stop() -> Result<(), String> {
    gcd_run_on_main_sync(stop_on_main)
}

fn start_on_main(callback: Callback) -> Result<(), String> {
    let mut guard = MONITORS
        .lock()
        .map_err(|e| format!("Fn monitor mutex poisoned: {}", e))?;

    if guard.is_some() {
        teardown(&mut guard);
    }

    FN_PRESSED.store(false, Ordering::SeqCst);

    let cb_global = callback.clone();
    let block_global = ConcreteBlock::new(move |event: id| {
        handle_event(event, &cb_global);
    })
    .copy();

    let cb_local = callback;
    let block_local = ConcreteBlock::new(move |event: id| -> id {
        handle_event(event, &cb_local);
        event
    })
    .copy();

    unsafe {
        let global_monitor: id = msg_send![
            class!(NSEvent),
            addGlobalMonitorForEventsMatchingMask: NS_EVENT_MASK_FLAGS_CHANGED
            handler: &*block_global
        ];
        let local_monitor: id = msg_send![
            class!(NSEvent),
            addLocalMonitorForEventsMatchingMask: NS_EVENT_MASK_FLAGS_CHANGED
            handler: &*block_local
        ];

        if global_monitor == nil && local_monitor == nil {
            return Err("NSEvent returned nil for both Fn monitors".to_string());
        }

        *guard = Some(MonitorHandles {
            global_monitor,
            local_monitor,
        });
    }

    // The blocks are retained by NSEvent's internal copy of them; our local
    // RcBlock handles drop here but that only releases our own reference.
    drop(block_global);
    drop(block_local);

    Ok(())
}

fn stop_on_main() -> Result<(), String> {
    let mut guard = MONITORS
        .lock()
        .map_err(|e| format!("Fn monitor mutex poisoned: {}", e))?;
    teardown(&mut guard);
    Ok(())
}

fn teardown(guard: &mut Option<MonitorHandles>) {
    if let Some(handles) = guard.take() {
        unsafe {
            if handles.global_monitor != nil {
                let _: () = msg_send![class!(NSEvent), removeMonitor: handles.global_monitor];
            }
            if handles.local_monitor != nil {
                let _: () = msg_send![class!(NSEvent), removeMonitor: handles.local_monitor];
            }
        }
    }
    FN_PRESSED.store(false, Ordering::SeqCst);
}

fn handle_event(event: id, callback: &Callback) {
    if event == nil {
        return;
    }
    let flags: u64 = unsafe { msg_send![event, modifierFlags] };
    let pressed = (flags & NS_EVENT_MODIFIER_FLAG_FUNCTION) != 0;
    let previous = FN_PRESSED.swap(pressed, Ordering::SeqCst);
    if previous != pressed {
        callback(pressed);
    }
}
