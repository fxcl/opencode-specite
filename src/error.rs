//! Domain error types for the specite CLI.

use std::fmt;

/// Raised when no Specite project can be located from the current directory.
#[derive(Debug)]
pub struct ProjectNotInitializedError(pub String);

impl fmt::Display for ProjectNotInitializedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ProjectNotInitializedError {}

impl ProjectNotInitializedError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

/// Raised when prompt generation cannot proceed.
#[derive(Debug)]
pub struct PromptError(pub String);

impl fmt::Display for PromptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for PromptError {}

impl PromptError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

/// Raised when user-supplied arguments are invalid (parity with Python's ValueError).
#[derive(Debug)]
pub struct ValidationError(pub String);

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ValidationError {}

impl ValidationError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}
