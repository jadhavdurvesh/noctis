//! Narrow operating-system boundary for the NOCTIS browser.
//! Engine crates must not call native OS APIs directly.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformKind {
    Windows,
    Linux,
    SandOs,
    Unsupported,
}

/// Platform capabilities will be implemented by OS-specific backends.
/// This trait intentionally exposes no browser-engine implementation.
pub trait HostPlatform: Send + Sync {
    fn kind(&self) -> PlatformKind;
}
