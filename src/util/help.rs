pub fn print_main_help() {
    println!("Usage: velo <command> [args...]");
    println!("Available commands:");
    println!("  ssh      Connect via SSH");
    println!("  tmux     Manage tmux sessions");
    println!("  add      Add a new SSH connection");
    println!("  list     List all SSH connections");
    println!("  remove   Remove an SSH connection");
    println!();
    println!("For more details, use 'velo <command> -h'");
}

pub fn print_tmux_help() {
    println!("Usage: velo tmux <subcommand> [args...]");
    println!("Available subcommands:");
    println!("  new <session_name>     Create a new tmux session");
    println!("  list                  List active tmux sessions");
    println!("  attach <session_name>  Attach to a tmux session");
    println!("  kill <session_name>    Kill a tmux session");
}

pub fn print_ssh_help() {
    println!("Usage: velo ssh <connection_name>");
    println!("Connect to a stored SSH connection.");
    println!("You can manage SSH connections using 'velo add', 'velo remove', or 'velo list'.");
}

pub fn print_add_help() {
    println!("Usage: velo add <name> <host> <user> <port>");
    println!("Add a new SSH connection.");
    println!("Optionally, you can store the SSH password for automatic login.");
}

pub fn print_list_help() {
    println!("Usage: velo list");
    println!("List all stored SSH connections.");
}

pub fn print_remove_help() {
    println!("Usage: velo remove <connection_name>");
    println!("Remove a stored SSH connection.");
}
