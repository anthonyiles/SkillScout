
# Rust: No serde(default) on Required API Fields

Do not mark fields `#[serde(default)]` if those fields are required by the external API contract (e.g. GitHub API responses). When a required field is absent, deserialization should fail loudly — not silently produce a zero value or empty string that propagates as valid data.

**Wrong — silently accepts a malformed GitHub device flow response:**
```rust
#[derive(Deserialize)]
pub struct DeviceFlowResponse {
    #[serde(default)]
    pub device_code: String,
    #[serde(default)]
    pub user_code: String,
    #[serde(default)]
    pub verification_uri: String,
    #[serde(default)]
    pub expires_in: u64,
    #[serde(default)]
    pub interval: u64,
}
```

**Right — fails on malformed response:**
```rust
#[derive(Deserialize)]
pub struct DeviceFlowResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}
```

Reserve `#[serde(default)]` for fields that are genuinely optional in the API spec, or for internal structs where a sensible default is correct by design.
