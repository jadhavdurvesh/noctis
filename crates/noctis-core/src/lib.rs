//! Shared, platform-neutral types used by NOCTIS subsystems.

use thiserror::Error;

/// Stable identifier scoped to a single document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub u64);

/// Errors shared across engine boundaries.
#[derive(Debug, Error)]
pub enum EngineError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("operation is not implemented: {0}")]
    NotImplemented(&'static str),
}
