# Dependency Register

NOCTIS owns its browser-engine implementations.

## Current direct dependencies
- `thiserror` — derives ergonomic Rust error implementations; it does not
  provide browser functionality.

## Prohibited engine substitutions
Do not add Chromium/Blink, Gecko, WebKit, CEF, Electron, or an embedded webview
as the implementation of NOCTIS.

## Required review for every new dependency
Record package/crate, exact version, purpose, license and notices, whether it
processes untrusted input, and whether it introduces platform coupling.
Prefer narrow libraries for well-defined primitives over black-box engines.
