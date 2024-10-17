pub fn print_main_help() {
    println!("Usage: velo <command> [args...]");
    println!("Available commands:");
    println!("  ssh      Connect via SSH");
    println!("  zellij   Manage Zellij sessions");
    println!("  add      Add a new SSH connection");
    println!("  list     List all SSH connections");
    println!("  remove   Remove an SSH connection");
    println!();
    println!("For more details, use 'velo <command> -h'");
}

pub fn print_zellij_new_help() {
    println!("Usage: velo zellij new <session_name>");
    println!("Create a new Zellij session with the given name.");
    println!("The session will be created in detached mode.");
}

pub fn print_zellij_list_help() {
    println!("Usage: velo zellij list");
    println!("List all active Zellij sessions.");
}

pub fn print_zellij_attach_help() {
    println!("Usage: velo zellij attach <session_name>");
    println!("Attach to an existing Zellij session with the given name.");
}

pub fn print_zellij_kill_help() {
    println!("Usage: velo zellij kill <session_name>");
    println!("Kill (terminate) a Zellij session with the given name.");
}

pub fn print_zellij_create_layout_help() {
    println!("Usage: velo zellij create-layout <layout_name> <layout_file_path>");
    println!("Create a new Zellij layout with the given name, using the content from the specified file.");
    println!("The layout will be saved in the Zellij layouts directory.");
}

pub fn print_zellij_list_layouts_help() {
    println!("Usage: velo zellij list-layouts");
    println!("List all available Zellij layouts in the Zellij layouts directory.");
}

pub fn print_ssh_help() {
    println!("Usage: velo ssh <connection_name>");
    println!("Connect to a stored SSH connection.");
    println!("You can manage SSH connections using 'velo add', 'velo remove', or 'velo list'.");
}

pub fn print_zellij_help() {
    println!("Usage: velo zellij <subcommand> [args...]");
    println!("Available subcommands:");
    println!("  new <session_name>             Create a new Zellij session");
    println!("  list                           List active Zellij sessions");
    println!("  attach <session_name>          Attach to a Zellij session");
    println!("  kill <session_name>            Kill a Zellij session");
    println!("  list-layouts                   List all Zellij layouts");
    println!("  create-layout <layout_name>    Create a Zellij layout");
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

pub fn print_add_key_help() {
    println!("Usage: velo add-key");
    println!("Add an SSH private key to the keyring for automatic login.");
}

pub fn print_copy_id_help() {
    println!("Usage: velo copy-id <connection_name> <key_path>");
    println!("Copy an SSH public key to a remote server.");
    println!("  <connection_name>  The name of the SSH connection (as defined in your SSH config)");
    println!("  <key_path>         The path to the public key file to copy");
}