//! Specite: spec-driven iterative development companion CLI library surface.
//!
//! The CLI binary in `src/main.rs` re-exports modules here. Tests in `tests/`
//! exercise the public surface.

pub mod assets;
pub mod error;
pub mod init;
pub mod iterations;
pub mod project;
pub mod prompts;
