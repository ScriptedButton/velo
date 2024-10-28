use std::fs;
use std::process::Command;

pub fn handle_zellij(args: &[String]) -> std::io::Result<()> {
    if args.is_empty() {
        println!("Usage: velo zellij <subcommand> [args...]");
        println!("Subcommands: new, list, attach, kill, create-layout, list-layouts");
        return Ok(());
    }

    let subcommand = &args[0];
    let rest_args = &args[1..];

    match subcommand.as_str() {
        "new" => {
            if rest_args.is_empty() {
                println!("Usage: velo zellij new <session_name>");
                return Ok(());
            }
            match create_session(&rest_args[0]) {
                Ok(_) => {
                    println!("Zellij session '{}' created successfully.", rest_args[0]);
                    println!(
                        "To attach to this session, run: velo zellij attach {}",
                        rest_args[0]
                    );
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        "list" => match list_sessions() {
            Ok(sessions) => {
                println!("Zellij sessions:");
                for session in sessions {
                    println!("  {}", session);
                }
            }
            Err(e) => eprintln!("Error listing Zellij sessions: {}", e),
        },
        "attach" => {
            if rest_args.is_empty() {
                println!("Usage: velo zellij attach <session_name>");
                return Ok(());
            }
            match attach_session(&rest_args[0]) {
                Ok(_) => println!("Attached to Zellij session: {}", rest_args[0]),
                Err(e) => eprintln!("Error attaching to Zellij session: {}", e),
            }
        }
        "kill" => {
            if rest_args.is_empty() {
                println!("Usage: velo zellij kill <session_name>");
                return Ok(());
            }
            match kill_session(&rest_args[0]) {
                Ok(_) => println!("Killed Zellij session: {}", rest_args[0]),
                Err(e) => eprintln!("Error killing Zellij session: {}", e),
            }
        }
        "create-layout" => {
            if rest_args.len() != 2 {
                println!("Usage: velo zellij create-layout <layout_name> <layout_file_path>");
                return Ok(());
            }
            let layout_name = &rest_args[0];
            let layout_file_path = &rest_args[1];
            match fs::read_to_string(layout_file_path) {
                Ok(content) => match create_layout(layout_name, &content) {
                    Ok(_) => println!("Layout '{}' created successfully.", layout_name),
                    Err(e) => eprintln!("Error creating layout: {}", e),
                },
                Err(e) => eprintln!("Error reading layout file: {}", e),
            }
        }
        "list-layouts" => match list_layouts() {
            Ok(layouts) => {
                println!("Available Zellij layouts:");
                for layout in layouts {
                    println!("  {}", layout);
                }
            }
            Err(e) => eprintln!("Error listing layouts: {}", e),
        },
        _ => println!(
            "Unknown Zellij subcommand: {}. Use 'velo zellij' for usage information.",
            subcommand
        ),
    }

    Ok(())
}

pub fn create_layout(layout_name: &str, layout_content: &str) -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let layout_dir = home_dir.join(".config").join("zellij").join("layouts");
    fs::create_dir_all(&layout_dir)
        .map_err(|e| format!("Failed to create layout directory: {}", e))?;

    let layout_path = layout_dir.join(format!("{}.kdl", layout_name));
    fs::write(&layout_path, layout_content)
        .map_err(|e| format!("Failed to write layout file: {}", e))?;

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
            Err(format!(
                "Failed to create session. Error: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
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
        Err(format!(
            "Failed to attach to session '{}'. Make sure the session exists and try again.",
            session_name
        ))
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
