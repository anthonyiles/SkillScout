use serde::Serialize;

#[derive(Debug)]
pub enum SkillScoutError {
    RepoUrlInvalid(String),
    GitOperationFailed(String),
    DatabaseBusy,
    DatabaseError(String),
    FileSystemError(String),
    NetworkError(String),
    AuthError(String),
    PathTraversalAttempt,
}

impl std::fmt::Display for SkillScoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RepoUrlInvalid(msg) => write!(f, "{}", msg),
            Self::GitOperationFailed(msg) => write!(f, "Git operation failed: {}", msg),
            Self::DatabaseBusy => write!(f, "Database busy. Please try again."),
            Self::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            Self::FileSystemError(msg) => write!(f, "File system error: {}", msg),
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Self::AuthError(msg) => write!(f, "{}", msg),
            Self::PathTraversalAttempt => write!(f, "Invalid path: path traversal components are not allowed"),
        }
    }
}

impl Serialize for SkillScoutError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<rusqlite::Error> for SkillScoutError {
    fn from(err: rusqlite::Error) -> Self {
        if let rusqlite::Error::SqliteFailure(ref e, _) = err {
            if e.code == rusqlite::ErrorCode::DatabaseBusy {
                return Self::DatabaseBusy;
            }
        }
        Self::DatabaseError(err.to_string())
    }
}

impl From<std::io::Error> for SkillScoutError {
    fn from(err: std::io::Error) -> Self {
        Self::FileSystemError(err.to_string())
    }
}

impl From<String> for SkillScoutError {
    fn from(msg: String) -> Self {
        Self::GitOperationFailed(msg)
    }
}
