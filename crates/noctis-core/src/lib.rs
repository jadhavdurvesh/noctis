//! Shared, platform-neutral types used by NOCTIS subsystems.
use std::fmt;

/// Stable identifier scoped to a single document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub u64);

/// Errors shared across engine boundaries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineError {
    InvalidInput(String),
    NotImplemented(&'static str),
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(message) => write!(f, "invalid input: {message}"),
            Self::NotImplemented(feature) => write!(f, "not implemented: {feature}"),
        }
    }
}

impl std::error::Error for EngineError {}
