use std::fmt;

/// Chyba volání Workeru. `code` odpovídá stabilním kódům z `spec/PROTOCOL.md` §1.4.
#[derive(Debug, thiserror::Error)]
pub enum WorkerError {
    #[error("worker answered {status} {code}: {message}")]
    Api { status: u16, code: String, message: String, retry_after: Option<u64> },
    #[error("transport: {0}")]
    Transport(String),
    #[error("protocol: {0}")]
    Protocol(String),
}

impl WorkerError {
    pub fn code(&self) -> Option<&str> {
        match self {
            Self::Api { code, .. } => Some(code),
            _ => None,
        }
    }

    /// Má smysl zkusit znovu později (síť, přetížení, probíhající provisioning)?
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Transport(_) => true,
            Self::Api { status, code, .. } => {
                *status >= 500 || *status == 429 || (*status == 409 && code == "conflict")
            }
            Self::Protocol(_) => false,
        }
    }

    pub fn retry_after(&self) -> Option<u64> {
        match self {
            Self::Api { retry_after, .. } => *retry_after,
            _ => None,
        }
    }
}

impl From<reqwest::Error> for WorkerError {
    fn from(e: reqwest::Error) -> Self {
        Self::Transport(e.to_string())
    }
}

/// Obecná chyba nodu (konfigurace, disk, vnitřní stav).
#[derive(Debug)]
pub struct NodeError(pub String);

impl fmt::Display for NodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for NodeError {}

impl From<std::io::Error> for NodeError {
    fn from(e: std::io::Error) -> Self {
        Self(e.to_string())
    }
}

impl From<WorkerError> for NodeError {
    fn from(e: WorkerError) -> Self {
        Self(e.to_string())
    }
}
