use std::process::Command;

pub fn handle_tmux(args: &[String]) {
    if args.is_empty() {
        println!("Usage: velo tmux <command> [args...]");
        return;
    }

    let subcommand = &args[0];
    let rest_args = &args[1..];

    match subcommand.as_str() {
        "new" => handle_tmux_new_session(rest_args),
        "list" | "ls" => handle_tmux_list_sessions(),
        "attach" => handle_tmux_attach_session(rest_args),
        "kill" => handle_tmux_kill_session(rest_args),
        _ => println!("Unknown tmux subcommand: {}", subcommand),
    }
}

fn handle_tmux_new_session(args: &[String]) {
    if args.len() != 1 {
        println!("Usage: velo tmux new <session_name>");
        return;
    }

    let session_name = &args[0];

    let status = Command::new("tmux")
        .arg("new-session")
        .arg("-d") // Detached mode
        .arg("-s")
        .arg(session_name)
        .status()
        .expect("Failed to create new tmux session");

    if status.success() {
        println!("Created new tmux session '{}'", session_name);
    } else {
        println!("Failed to create new tmux session");
    }
}

fn handle_tmux_list_sessions() {
    let output = Command::new("tmux")
        .arg("list-sessions")
        .output()
        .expect("Failed to list tmux sessions");

    if output.status.success() {
        let sessions = String::from_utf8_lossy(&output.stdout);
        if sessions.trim().is_empty() {
            println!("No active tmux sessions found.");
        } else {
            println!("Active tmux sessions:");
            println!("{}", sessions);
        }
    } else {
        println!("Failed to list tmux sessions");
    }
}

fn handle_tmux_attach_session(args: &[String]) {
    if args.len() != 1 {
        println!("Usage: velo tmux attach <session_name>");
        return;
    }

    let session_name = &args[0];

    let status = Command::new("tmux")
        .arg("attach-session")
        .arg("-t")
        .arg(session_name)
        .status()
        .expect("Failed to attach to tmux session");

    if status.success() {
        println!("Attached to tmux session '{}'", session_name);
    } else {
        println!("Failed to attach to tmux session '{}'", session_name);
    }
}

fn handle_tmux_kill_session(args: &[String]) {
    if args.len() != 1 {
        println!("Usage: velo tmux kill <session_name>");
        return;
    }

    let session_name = &args[0];

    let status = Command::new("tmux")
        .arg("kill-session")
        .arg("-t")
        .arg(session_name)
        .status()
        .expect("Failed to kill tmux session");

    if status.success() {
        println!("Killed tmux session '{}'", session_name);
    } else {
        println!("Failed to kill tmux session '{}'", session_name);
    }
}
