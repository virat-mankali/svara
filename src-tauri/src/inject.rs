use std::process::Command;
use std::thread;
use std::time::Duration;

pub fn inject_text(text: &str) -> anyhow::Result<()> {
    let mut clipboard = arboard::Clipboard::new()?;
    clipboard.set_text(text.to_string())?;

    thread::sleep(Duration::from_millis(70));

    let status = Command::new("osascript")
        .arg("-e")
        .arg("tell application \"System Events\" to keystroke \"v\" using command down")
        .status()?;

    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("paste automation failed; Svara may need Accessibility permission")
    }
}
