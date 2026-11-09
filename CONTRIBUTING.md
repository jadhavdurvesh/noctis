# Contributing to NOCTIS

Thank you for helping build an independent browser engine.

## Engineering rules
- Never add Chromium/Blink, Gecko, WebKit, CEF, Electron, or an embedded webview
  as the implementation of the engine.
- Keep parser and rendering crates platform-neutral.
- Record every new dependency, purpose, version, license, and input-security
  considerations in `docs/DEPENDENCIES.md`.
- Add regression tests for parser and runtime behavior.
- Avoid unsafe Rust unless necessary, justified, and reviewed.
- Do not claim standards conformance beyond tested behavior.

## Before submitting
Run:
```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Contributions
By intentionally submitting a contribution for inclusion, You agree that it
is licensed to the project under NRSL 1.0 unless a separate written agreement
says otherwise. You retain ownership of Your contribution.
