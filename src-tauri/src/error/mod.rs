use serde::Serialize;

/// The single application error type. Every fallible operation in `app`, `config`,
/// `skills`, `installer`, and `process` returns `Result<T, AppError>`, so both the
/// Tauri command layer and the CLI layer can convert it into a user-facing message
/// (and, for the CLI, an exit code) in exactly one place.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("invalid input: {0}")]
    Validation(String),

    #[error("{0}")]
    MissingDependency(String),

    #[error("process error: {0}")]
    Process(String),

    #[error("installation failed: {0}")]
    Installation(String),

    #[error("filesystem error: {0}")]
    Io(String),

    #[error("could not parse skill URL: {0}")]
    UrlParse(String),

    #[error("preferences error: {0}")]
    Preferences(String),

    #[error("operation was cancelled")]
    Cancelled,

    #[error("not found: {0}")]
    NotFound(String),
}

impl AppError {
    /// Exit code used by the CLI layer only. The GUI never touches this — it just
    /// sees the serialized message via the IPC error channel.
    pub fn exit_code(&self) -> i32 {
        match self {
            AppError::Validation(_) | AppError::UrlParse(_) | AppError::NotFound(_) => 2,
            AppError::MissingDependency(_) => 3,
            AppError::Cancelled => 130,
            AppError::Config(_)
            | AppError::Process(_)
            | AppError::Installation(_)
            | AppError::Io(_)
            | AppError::Preferences(_) => 1,
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err.to_string())
    }
}

/// Tauri commands return `Result<T, AppError>`; Tauri serializes the `Err` side to
/// the frontend via this impl rather than exposing Rust internals (source chains,
/// backtraces, path internals) across the IPC boundary.
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
