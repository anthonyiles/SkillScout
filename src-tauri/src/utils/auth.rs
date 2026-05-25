pub fn save_token(token: &str) -> Result<(), String> {
    keyring::Entry::new("skillscout", "github_token")
        .map_err(|e| format!("Failed to access OS keyring: {}", e))?
        .set_password(token)
        .map_err(|e| format!("Failed to store token in OS keyring: {}", e))
}

pub const NO_CREDENTIAL: &str = "Not authenticated. Please sign in with GitHub.";

pub fn load_token() -> Result<String, String> {
    let entry = keyring::Entry::new("skillscout", "github_token")
        .map_err(|e| { eprintln!("Keyring access error: {}", e); format!("OS keyring unavailable: {}", e) })?;
    entry.get_password()
        .map_err(|e| {
            eprintln!("Keyring read error: {}", e);
            if matches!(e, keyring::Error::NoEntry) {
                NO_CREDENTIAL.to_string()
            } else {
                format!("OS keyring unavailable: {}", e)
            }
        })
}
