use std::process::Command;
use std::fs;
use std::path::PathBuf;

pub fn create_layout(layout_name: &str, layout_content: &str) -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let layout_dir = home_dir.join(".config").join("zellij").join("layouts");
    fs::create_dir_all(&layout_dir).map_err(|e| format!("Failed to create layout directory: {}", e))?;
    
    let layout_path = layout_dir.join(format!("{}.kdl", layout_name));
    fs::write(&layout_path, layout_content).map_err(|e| format!("Failed to write layout file: {}", e))?;
    
    Ok(())
}

pub fn list_layouts() -> Result<Vec<String>, String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let layout_dir = home_dir.join(".config").join("zellij").join("layouts");
    
    let layouts: Vec<String> = fs::read_dir(layout_dir)
        .map_err(|e| format!("Failed to read layout directory: {}", e))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "kdl" {
                Some(path.file_stem()?.to_string_lossy().into_owned())
            } else {
                None
            }
        })
        .collect();

    Ok(layouts)
}

pub fn create_session(session_name: &str) -> Result<(), String> {
    let output = Command::new("zellij")
        .args(&["--session", session_name])
        .output()
        .map_err(|e| format!("Failed to execute zellij: {}", e))?;

    // Check if the command was successful (exit status 0)
    if output.status.success() {
        Ok(())
    } else {
        // If the command wasn't successful, check if the session was still created
        let list_output = Command::new("zellij")
            .args(&["list-sessions"])
            .output()
            .map_err(|e| format!("Failed to list Zellij sessions: {}", e))?;

        let sessions = String::from_utf8_lossy(&list_output.stdout);
        if sessions.contains(session_name) {
            Ok(()) // Session was created despite non-zero exit status
        } else {
            Err(format!("Failed to create session. Error: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }
}

pub fn list_sessions() -> Result<Vec<String>, String> {
    let output = Command::new("zellij")
        .args(&["list-sessions"])
        .output()
        .map_err(|e| format!("Failed to list Zellij sessions: {}", e))?;

    if output.status.success() {
        let sessions = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect();
        Ok(sessions)
    } else {
        Err(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

pub fn attach_session(session_name: &str) -> Result<(), String> {
    let status = Command::new("zellij")
        .args(&["attach", session_name])
        .status()
        .map_err(|e| format!("Failed to execute zellij attach: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("Failed to attach to session '{}'. Make sure the session exists and try again.", session_name))
    }
}

pub fn kill_session(session_name: &str) -> Result<(), String> {
    let output = Command::new("zellij")
        .args(&["kill-session", session_name])
        .output()
        .map_err(|e| format!("Failed to kill Zellij session: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}