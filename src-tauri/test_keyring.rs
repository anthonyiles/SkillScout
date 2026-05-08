use keyring::Entry;
fn main() {
    let entry = Entry::new("skillscout", "test_user").unwrap();
    match entry.set_password("test_pass") {
        Ok(_) => println!("Keyring works!"),
        Err(e) => println!("Keyring error: {}", e),
    }
}
