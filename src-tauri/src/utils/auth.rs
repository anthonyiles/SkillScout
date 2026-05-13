pub fn save_token(token: &str) -> Result<(), String> {
    match keyring::Entry::new("skillscout", "github_token") {
        Ok(entry) if entry.set_password(token).is_ok() => Ok(()),
        _ => Err(
            "Your system keyring is unavailable. GitHub authentication cannot be stored securely. \
             Please ensure a keyring service (e.g. GNOME Keyring, KWallet, or Windows Credential Manager) is running."
                .to_string(),
        ),
    }
}

pub fn load_token() -> Result<String, String> {
    keyring::Entry::new("skillscout", "github_token")
        .ok()
        .and_then(|entry| entry.get_password().ok())
        .ok_or_else(|| "Not authenticated. Please connect your GitHub account.".to_string())
}
