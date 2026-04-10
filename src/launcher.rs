use std::process::Command;
use crate::models::Browser;

pub fn launch_browser(browser: &Browser, launch_args: &str) -> Result<(), String> {
    let mut cmd = Command::new(&browser.exe_path);

    let args: Vec<&str> = launch_args.split_whitespace().collect();
    for arg in &args {
        cmd.arg(arg);
    }

    cmd.spawn().map_err(|e| format!("Failed to launch {}: {}", browser.name, e))?;
    Ok(())
}
