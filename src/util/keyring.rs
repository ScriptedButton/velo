use keyring::{Entry, Result};

// Store passphrase securely
pub fn store_passphrase(service: &str, username: &str, passphrase: &str) -> Result<()> {
    let entry = Entry::new(service, username)?;
    entry.set_password(passphrase)?;
    println!("Passphrase securely stored.");
    Ok(())
}

// Retrieve passphrase securely
pub fn retrieve_passphrase(service: &str, username: &str) -> Option<String> {
    match Entry::new(service, username) {
        Ok(entry) => match entry.get_password() {
            Ok(pass) => Some(pass),
            Err(_) => {
                println!("No passphrase found, or error retrieving it.");
                None
            }
        },
        Err(_) => {
            println!("Error initializing keyring entry.");
            None
        }
    }
}

// Delete passphrase securely
pub fn delete_passphrase(service: &str, username: &str) {
    match Entry::new(service, username) {
        Ok(entry) => match entry.delete_credential() {
            Ok(_) => println!("Passphrase deleted successfully."),
            Err(_) => println!("Failed to delete passphrase or it doesn't exist."),
        },
        Err(_) => println!("Error initializing keyring entry."),
    }
}
