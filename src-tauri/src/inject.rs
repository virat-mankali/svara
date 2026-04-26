use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct InsertionTarget {
    pid: i32,
}

#[cfg(target_os = "macos")]
mod macos {
    use std::ffi::{c_int, c_void};

    const CG_HID_EVENT_TAP: u32 = 0;
    const CG_EVENT_FLAG_MASK_COMMAND: u64 = 0x0010_0000;
    const KEY_CODE_V: u16 = 9;
    const NO_ERR: i32 = 0;

    #[repr(C)]
    struct ProcessSerialNumber {
        high_long_of_psn: u32,
        low_long_of_psn: u32,
    }

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
        fn CGEventCreateKeyboardEvent(
            source: *const c_void,
            virtual_key: u16,
            key_down: bool,
        ) -> *mut c_void;
        fn CGEventSetFlags(event: *mut c_void, flags: u64);
        fn CGEventPost(tap: u32, event: *mut c_void);
        fn CFRelease(cf: *const c_void);
    }

    #[link(name = "Carbon", kind = "framework")]
    extern "C" {
        fn GetFrontProcess(psn: *mut ProcessSerialNumber) -> c_int;
        fn GetProcessPID(psn: *const ProcessSerialNumber, pid: *mut c_int) -> c_int;
        fn GetProcessForPID(pid: c_int, psn: *mut ProcessSerialNumber) -> c_int;
        fn SetFrontProcess(psn: *const ProcessSerialNumber) -> c_int;
    }

    pub fn frontmost_pid() -> Option<i32> {
        let mut psn = ProcessSerialNumber {
            high_long_of_psn: 0,
            low_long_of_psn: 0,
        };
        let mut pid = 0;

        unsafe {
            if GetFrontProcess(&mut psn) != NO_ERR {
                return None;
            }
            if GetProcessPID(&psn, &mut pid) != NO_ERR {
                return None;
            }
        }

        Some(pid)
    }

    pub fn activate_pid(pid: i32) -> anyhow::Result<()> {
        let mut psn = ProcessSerialNumber {
            high_long_of_psn: 0,
            low_long_of_psn: 0,
        };

        unsafe {
            if GetProcessForPID(pid, &mut psn) != NO_ERR {
                anyhow::bail!("could not find the app that had focus before recording");
            }
            if SetFrontProcess(&psn) != NO_ERR {
                anyhow::bail!("could not reactivate the app that had focus before recording");
            }
        }

        Ok(())
    }

    pub fn paste_shortcut() -> anyhow::Result<()> {
        if unsafe { !AXIsProcessTrusted() } {
            anyhow::bail!("Svara needs Accessibility permission to paste into other apps");
        }

        unsafe {
            let key_down = CGEventCreateKeyboardEvent(std::ptr::null(), KEY_CODE_V, true);
            let key_up = CGEventCreateKeyboardEvent(std::ptr::null(), KEY_CODE_V, false);

            if key_down.is_null() || key_up.is_null() {
                if !key_down.is_null() {
                    CFRelease(key_down);
                }
                if !key_up.is_null() {
                    CFRelease(key_up);
                }
                anyhow::bail!("failed to create paste keyboard event");
            }

            CGEventSetFlags(key_down, CG_EVENT_FLAG_MASK_COMMAND);
            CGEventSetFlags(key_up, CG_EVENT_FLAG_MASK_COMMAND);
            CGEventPost(CG_HID_EVENT_TAP, key_down);
            CGEventPost(CG_HID_EVENT_TAP, key_up);
            CFRelease(key_down);
            CFRelease(key_up);
        }

        Ok(())
    }
}

pub fn capture_insertion_target() -> Option<InsertionTarget> {
    let pid = frontmost_pid()?;
    if pid as u32 == std::process::id() {
        None
    } else {
        Some(InsertionTarget { pid })
    }
}

pub fn inject_text(text: &str, target: Option<InsertionTarget>) -> anyhow::Result<()> {
    let mut clipboard = arboard::Clipboard::new()?;
    let previous_clipboard = clipboard.get_text().ok();

    clipboard.set_text(text.to_string())?;

    if let Some(target) = target {
        activate_pid(target.pid)?;
    }

    thread::sleep(Duration::from_millis(140));
    paste_shortcut()?;
    thread::sleep(Duration::from_millis(900));

    if let Some(previous_clipboard) = previous_clipboard {
        let _ = clipboard.set_text(previous_clipboard);
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn frontmost_pid() -> Option<i32> {
    macos::frontmost_pid()
}

#[cfg(not(target_os = "macos"))]
fn frontmost_pid() -> Option<i32> {
    None
}

#[cfg(target_os = "macos")]
fn activate_pid(pid: i32) -> anyhow::Result<()> {
    macos::activate_pid(pid)
}

#[cfg(not(target_os = "macos"))]
fn activate_pid(_pid: i32) -> anyhow::Result<()> {
    Ok(())
}

#[cfg(target_os = "macos")]
fn paste_shortcut() -> anyhow::Result<()> {
    macos::paste_shortcut().map_err(|error| {
        anyhow::anyhow!("paste automation failed; Svara may need Accessibility permission: {error}")
    })
}

#[cfg(not(target_os = "macos"))]
fn paste_shortcut() -> anyhow::Result<()> {
    anyhow::bail!("paste automation is only implemented on macOS")
}
