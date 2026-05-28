use serde::Serialize;

#[derive(Debug)]
#[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_repo_url_invalid() {
        let e = SkillScoutError::RepoUrlInvalid("bad url".to_string());
        assert_eq!(e.to_string(), "bad url");
    }

    #[test]
    fn display_git_operation_failed() {
        let e = SkillScoutError::GitOperationFailed("clone failed".to_string());
        assert_eq!(e.to_string(), "Git operation failed: clone failed");
    }

    #[test]
    fn display_database_busy() {
        assert_eq!(SkillScoutError::DatabaseBusy.to_string(), "Database busy. Please try again.");
    }

    #[test]
    fn display_database_error() {
        let e = SkillScoutError::DatabaseError("constraint violation".to_string());
        assert_eq!(e.to_string(), "Database error: constraint violation");
    }

    #[test]
    fn display_filesystem_error() {
        let e = SkillScoutError::FileSystemError("permission denied".to_string());
        assert_eq!(e.to_string(), "File system error: permission denied");
    }

    #[test]
    fn display_network_error() {
        let e = SkillScoutError::NetworkError("timeout".to_string());
        assert_eq!(e.to_string(), "Network error: timeout");
    }

    #[test]
    fn display_auth_error() {
        let e = SkillScoutError::AuthError("token expired".to_string());
        assert_eq!(e.to_string(), "token expired");
    }

    #[test]
    fn display_path_traversal_attempt() {
        assert_eq!(
            SkillScoutError::PathTraversalAttempt.to_string(),
            "Invalid path: path traversal components are not allowed"
        );
    }

    #[test]
    fn from_io_error_produces_filesystem_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let e = SkillScoutError::from(io_err);
        assert!(matches!(e, SkillScoutError::FileSystemError(_)));
        assert!(e.to_string().contains("file not found"));
    }

    #[test]
    fn from_string_produces_git_operation_failed() {
        let e = SkillScoutError::from("something broke".to_string());
        assert!(matches!(e, SkillScoutError::GitOperationFailed(_)));
    }

    #[test]
    fn serialize_produces_plain_string() {
        let e = SkillScoutError::DatabaseBusy;
        let json = serde_json::to_string(&e).unwrap();
        assert_eq!(json, "\"Database busy. Please try again.\"");
    }
}
