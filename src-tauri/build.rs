// The client ID is a public OAuth value (not a secret) — safe to ship in the binary.
// Override via GITHUB_CLIENT_ID env var or a local .env file for forks/alternative apps.
const DEFAULT_CLIENT_ID: &str = "Ov23livwWLw53DLJYdkG";

fn main() {
    let _ = dotenvy::dotenv();
    let id = std::env::var("GITHUB_CLIENT_ID")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_CLIENT_ID.to_string());
    println!("cargo:rustc-env=GITHUB_CLIENT_ID={}", id);
    println!("cargo:rerun-if-changed=.env");
    println!("cargo:rerun-if-env-changed=GITHUB_CLIENT_ID");
    tauri_build::build()
}
