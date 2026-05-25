fn main() {
    let _ = dotenvy::dotenv();
    let id = std::env::var("GITHUB_CLIENT_ID")
        .expect("GITHUB_CLIENT_ID must be set — copy .env.example to .env and fill in the value");
    println!("cargo:rustc-env=GITHUB_CLIENT_ID={}", id);
    println!("cargo:rerun-if-changed=.env");
    println!("cargo:rerun-if-env-changed=GITHUB_CLIENT_ID");
    tauri_build::build()
}
