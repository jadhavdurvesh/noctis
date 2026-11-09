# Dependency Register

NOCTIS aims to own its browser-engine implementations and keep its initial
foundation free of external package dependencies.

## Runtime dependencies
None. The Rust workspace currently uses only the Rust standard library.

## Development-time data generator
`tools/generate_html_entities.py` uses Python's standard-library
`html.entities.html5` mapping to regenerate
`crates/noctis-html/src/named_entities.tsv`. Python is a development/CI tool
only; the Rust browser runtime does not depend on Python. The generated table
is committed to the repository so normal Cargo builds need no network access
or external package.

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