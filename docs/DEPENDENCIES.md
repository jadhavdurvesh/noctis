# Dependency Register

NOCTIS owns its browser-engine implementations. Narrow third-party bindings
and build tools are permitted when documented; existing browser engines and
embedded webviews are not.

## Rust runtime dependencies

| Package | Version | Purpose | License | Scope |
|---|---:|---|---|---|
| `jni` | 0.21.1 | Call the NOCTIS Rust HTML parser from the Android Kotlin app through JNI | MIT OR Apache-2.0 | Android bridge crate only |

The JNI crate is a language/platform binding. It does not parse HTML, render
pages, provide a DOM, execute JavaScript, or embed a browser engine. The
HTML parser remains in `noctis-html`.

## Development and packaging tools

| Tool | Version / action | Purpose |
|---|---|---|
| Rust stable | `rust-toolchain.toml` | Build/test the workspace |
| Python standard library | host Python 3 | Generate the committed HTML entity table |
| Java | 17 | Android Gradle build |
| Android SDK | API 35 | Android app compile target |
| Android NDK | 27.2.12479018 | Cross-compile Rust JNI library |
| Gradle | 8.9 | Assemble Android APK |
| `cargo-ndk` | 4.1.2 | Place Rust Android ABI outputs in Android JNI library directories |
| Android Gradle Plugin | 8.7.3 | Android application build |
| Kotlin Gradle plugin | 2.0.21 | Android app source compilation |

GitHub Actions uses the following workflow actions for CI/release automation:
`actions/checkout@v4`, `dtolnay/rust-toolchain@stable`,
`actions/setup-java@v5`, `android-actions/setup-android@v4`,
`gradle/actions/setup-gradle@v6`, `actions/upload-artifact@v4`,
`actions/download-artifact@v4`, and `softprops/action-gh-release@v2`.

These are build/automation actions, not runtime dependencies shipped in
NOCTIS binaries.

## Development-time entity generator

`tools/generate_html_entities.py` uses Python's standard-library
`html.entities.html5` mapping to regenerate
`crates/noctis-html/src/named_entities.tsv`. Python is a development tool
only; Rust builds consume the committed TSV file and do not require Python.

## Internal crate relationships

- `noctis-cli` uses `noctis-html` for HTML/DOM parsing and `noctis-css` for
  stylesheet inspection.
- `noctis-css` uses `noctis-html` DOM types for selector matching.
- `noctis-android-bridge` uses `noctis-html` and the JNI binding crate.
- Other crates remain independent subsystem boundaries until their public
  interfaces are implemented.

## Prohibited engine substitutions

Do not add Chromium/Blink, Gecko, WebKit, CEF, Electron, or an embedded webview
as the implementation of NOCTIS.

## Required review for future dependencies

Record package/crate, exact version, purpose, license and notices, whether it
processes untrusted input, and whether it introduces platform coupling.
Prefer narrow, auditable libraries for well-defined primitives over
black-box browser or runtime engines.
