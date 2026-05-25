fn main() {
    let _ = dotenvy::dotenv();
    if let Ok(id) = std::env::var("GITHUB_CLIENT_ID") {
        println!("cargo:rustc-env=GITHUB_CLIENT_ID={}", id);
    }
    println!("cargo:rerun-if-changed=.env");
    println!("cargo:rerun-if-env-changed=GITHUB_CLIENT_ID");
    tauri_build::build()
}
