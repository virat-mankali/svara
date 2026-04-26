use std::process::Command;
use std::thread;
use std::time::Duration;

#[cfg(target_os = "macos")]
mod macos {
    use std::ffi::c_void;

    const CG_HID_EVENT_TAP: u32 = 0;
    const CG_EVENT_FLAG_MASK_COMMAND: u64 = 0x0010_0000;
    const KEY_CODE_V: u16 = 9;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn CGEventCreateKeyboardEvent(
            source: *const c_void,
            virtual_key: u16,
            key_down: bool,
        ) -> *mut c_void;
        fn CGEventSetFlags(event: *mut c_void, flags: u64);
        fn CGEventPost(tap: u32, event: *mut c_void);
        fn CFRelease(cf: *const c_void);
    }

    pub fn paste_shortcut() -> anyhow::Result<()> {
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

pub fn frontmost_bundle_identifier() -> Option<String> {
    let output = Command::new("osascript")
        .args([
            "-e",
            "tell application \"System Events\" to get bundle identifier of first application process whose frontmost is true",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let bundle_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if bundle_id.is_empty() || bundle_id == "com.viratmankali.svara" {
        None
    } else {
        Some(bundle_id)
    }
}

pub fn inject_text(text: &str, target_bundle_id: Option<&str>) -> anyhow::Result<()> {
    let mut clipboard = arboard::Clipboard::new()?;
    let previous_clipboard = clipboard.get_text().ok();

    clipboard.set_text(text.to_string())?;

    if let Some(bundle_id) = target_bundle_id {
        let _ = Command::new("open").args(["-b", bundle_id]).status();
    }

    thread::sleep(Duration::from_millis(140));
    paste_shortcut()?;
    thread::sleep(Duration::from_millis(220));

    if let Some(previous_clipboard) = previous_clipboard {
        let _ = clipboard.set_text(previous_clipboard);
    }

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
