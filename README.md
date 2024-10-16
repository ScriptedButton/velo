# Velo CLI

Velo is a command-line interface (CLI) tool designed to simplify and streamline SSH connections and (planned) tmux session management. It allows users to store and manage connection details securely, making it easier to connect to remote servers without remembering complex connection strings.

## Features

- Securely store and manage SSH connection details
- Encrypt configuration file for added security
- Easy-to-use commands for adding, removing, and listing connections
- Seamless SSH connections using stored details
- Optional password storage for connections
- Compatible with systems with or without `sshpass`

## Installation

### Prerequisites

- Rust programming language (https://www.rust-lang.org/tools/install)
- OpenSSL development libraries

### Building from source

1. Clone the repository:
   ```
   git clone https://github.com/ScriptedButton/velo.git
   cd velo
   ```

2. Build the project:
   ```
   cargo build --release
   ```

3. The binary will be available at `target/release/velo`

4. (Optional) Move the binary to a directory in your PATH:
   ```
   sudo mv target/release/velo /usr/local/bin/
   ```

## Usage

### Adding a new connection

```
velo add <name> <host> <user> <port>
```

Example:
```
velo add myserver 192.168.1.100 admin 22
```

You will be prompted if you want to store the SSH password and for the configuration encryption password.

### Listing stored connections

```
velo list
```

### Removing a connection

```
velo remove <name>
```

or

```
velo rm <name>
```

### Connecting to a stored server

```
velo ssh <name>
```

Example:
```
velo ssh myserver
```

## Security

- All connection details are stored in an encrypted configuration file.
- The configuration file is encrypted using AES-256-GCM.
- SSH passwords, if stored, are kept in the encrypted configuration file.
- Users are always prompted for the configuration encryption password when accessing or modifying the configuration.

## Planned Features

- TMux session management
- Support for SSH key authentication
- Custom SSH options

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Disclaimer

This tool stores sensitive information. While we make every effort to ensure the security of your data, please use it at your own risk and always follow best practices for SSH security.