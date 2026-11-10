# NOCTIS

**An independent browser engine built from first principles.**

NOCTIS is a greenfield browser-engine project targeting **Windows, Linux, and
eventually SAND OS**. It is not a Chromium, Blink, Gecko, or WebKit fork and
does not embed those browser engines.

> **Status:** experimental foundation. It is not suitable for everyday browsing
> and does not yet implement the modern web platform.

## Non-negotiable engineering rules

- No Chromium/Blink, Gecko, or WebKit engine dependency.
- Our own HTML, DOM, CSS, layout, JavaScript runtime, and browser orchestration.
- Platform-neutral engine crates; OS-specific code behind a small platform API.
- Every third-party dependency must be justified and recorded.
- Treat all documents and network data as untrusted.

## Workspace

| Crate | Responsibility | Initial status |
|---|---|---|
| `noctis-core` | Shared engine primitives | Foundation |
| `noctis-html` | Tokenizer, parser, DOM | Experimental subset with tests |
| `noctis-css` | CSS tokenizer, declarations, selectors and scoped cascade | Experimental subset |
| `noctis-layout` | Layout tree and geometry | Skeleton |
| `noctis-paint` | Display list and painting | Skeleton |
| `noctis-net` | URL and networking | Skeleton |
| `noctis-js` | Independent JavaScript runtime | Skeleton |
| `noctis-platform` | Windows/Linux/SAND OS boundary | Skeleton |
| `noctis-android-bridge` | JNI bridge for Android Engine Lab | Preview implementation |
| `noctis-cli` | Developer CLI | First implementation |

## Build and run

Requires stable Rust.

```sh
cargo test --workspace
cargo run -p noctis-cli -- --tree examples/hello.html
cargo run -p noctis-cli -- --html examples/hello.html
cargo run -p noctis-cli -- --css path/to/styles.css
```

The CLI supports `--tree`, `--html`, `--css`, `--fragment`, `--help`,
and `--version`. It is a local developer tool: it does not open network
URLs, execute JavaScript, render pages, or provide a graphical browser window.

## Current engine implementation

### HTML and DOM

The HTML subsystem includes the complete 2,231-entry named character-reference
table, numeric references, CRLF/NUL input preprocessing, raw-text/RCDATA
handling, bogus-comment handling, HTML void/self-closing rules, basic document
insertion modes, implied `html/head/body` construction, selected table foster
parenting and inline-formatting recovery, plus an arena-backed DOM with stable
node IDs and parent/child/sibling queries and mutation.

The HTML crate has seven checked-in input/expected-serialization fixtures
covering entities, implied list items, raw script text, void elements, table
foster parenting, misnested formatting, and document parsing. Run them with
`cargo test -p noctis-html`. See
[HTML conformance testing](docs/HTML_CONFORMANCE.md) and
[implementation scope](docs/HTML_IMPLEMENTATION.md).

This is a deliberately scoped implementation, not full HTML Living Standard
conformance. Full tokenizer states, all tree-construction insertion modes,
the complete adoption-agency algorithm, templates/foreign-content handling,
DOM events/namespaces, and broad differential conformance testing remain future
work.

### CSS

The independent CSS crate currently includes CSS tokenization, declaration
parsing, custom-property name preservation, `!important` detection, a scoped
compound-selector subset (type, ID, class and attribute selectors), simple
stylesheet rule parsing, and a limited per-element cascade. It does not yet
implement selector combinators, pseudo-classes, inheritance, media-condition
evaluation, a complete cascade, or layout/painting. See
[CSS implementation scope](docs/CSS_IMPLEMENTATION.md).

## Build artifacts

GitHub Actions now contains:
- Windows x64 CLI executable package (`noctis-cli.exe` inside a ZIP).
- Linux and macOS CLI archives.
- Android Engine Lab debug APK for ARM64, ARMv7 and x86-64.
- SHA-256 checksums on tagged GitHub Releases.

The Android Engine Lab is a native Kotlin preview app that calls the NOCTIS
Rust parser through JNI. It does not use Android WebView or another browser
engine. It accepts HTML text and displays the parsed DOM tree; it is not a
complete mobile browser. The APK is debug-signed for testing, not Play Store
distribution.

See [Building and Packaging](docs/BUILDING.md) for local commands, artifact
locations, and release workflow instructions. The release workflow runs on
`v*` tags or manual dispatch; the Android preview workflow uploads a debug
APK on relevant changes.

## Platform strategy

The engine crates must not depend on window handles, native UI widgets, or OS
APIs. Windows and Linux are the first desktop targets. SAND OS support will be
added through a dedicated backend when its system interfaces are ready.

## Dependency policy

A library may be considered for a focused primitive such as cryptography,
font rasterization, or image decoding. It must not provide the browser engine,
DOM, CSS engine, JavaScript engine, or embedded webview. See
[`docs/DEPENDENCIES.md`](docs/DEPENDENCIES.md).

## License

NOCTIS is distributed under the **NOCTIS Restricted Source License (NRSL) 1.0**
in [`LICENSE`](LICENSE). It allows personal, educational, research, and
non-commercial evaluation use, subject to its terms. Public redistribution
and commercial use require prior written permission. This is a custom,
source-available license, not an OSI-approved open-source license.
