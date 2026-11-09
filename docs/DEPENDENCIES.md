# Dependency Register

NOCTIS aims to own its browser-engine implementations and keep its initial
foundation free of external package dependencies.

## Current external dependencies
None. The current workspace uses only the Rust standard library.

## Internal crate relationships
- `noctis-cli` uses `noctis-html` to exercise the parser.
- Other crates are currently subsystem boundaries and should acquire internal
  dependencies only when a real interface requires them.

## Prohibited engine substitutions
Do not add Chromium/Blink, Gecko, WebKit, CEF, Electron, or an embedded webview
as the implementation of NOCTIS.

## Required review for every proposed dependency
Record package/crate, exact version, purpose, license and notices, whether it
processes untrusted input, and whether it introduces platform coupling.
Prefer narrow, auditable libraries for well-defined primitives over black-box
browser or runtime engines.