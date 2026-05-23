pub fn save_token(token: &str) -> Result<(), String> {
    keyring::Entry::new("skillscout", "github_token")
        .map_err(|e| format!("Failed to access OS keyring: {}", e))?
        .set_password(token)
        .map_err(|e| format!("Failed to store token in OS keyring: {}", e))
}

pub fn load_token() -> Result<String, String> {
    keyring::Entry::new("skillscout", "github_token")
        .map_err(|e| { eprintln!("Keyring access error: {}", e); "Not authenticated. Please sign in with GitHub.".to_string() })?
        .get_password()
        .map_err(|e| { eprintln!("Keyring read error: {}", e); "Not authenticated. Please sign in with GitHub.".to_string() })
}
