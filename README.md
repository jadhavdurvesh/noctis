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
| `noctis-css` | CSS syntax and style system | Skeleton |
| `noctis-layout` | Layout tree and geometry | Skeleton |
| `noctis-paint` | Display list and painting | Skeleton |
| `noctis-net` | URL and networking | Skeleton |
| `noctis-js` | Independent JavaScript runtime | Skeleton |
| `noctis-platform` | Windows/Linux/SAND OS boundary | Skeleton |
| `noctis-cli` | Development harness | First implementation |

## Build and run

Requires stable Rust.

```sh
cargo test --workspace
cargo run -p noctis-cli -- examples/hello.html
```

The CLI parses a local HTML file and prints a debug DOM tree. The HTML subsystem
now includes the complete 2,231-entry named character-reference table,
numeric references, CRLF/NUL input preprocessing, raw-text/RCDATA handling,
bogus-comment handling, HTML void/self-closing rules, basic document insertion
modes, implied `html/head/body` construction, selected table foster parenting
and inline-formatting recovery, plus an arena-backed DOM with stable node IDs
and parent/child/sibling queries and mutation.

The HTML crate also has a fixture-based serialization test harness. Its
checked-in cases cover entities, implied list items, raw script text, void
elements, table foster parenting, misnested formatting, and document parsing.
Run them with `cargo test -p noctis-html`; see
[`docs/HTML_CONFORMANCE.md`](docs/HTML_CONFORMANCE.md).

This remains a deliberately scoped implementation, not full HTML Living
Standard conformance. Full tokenizer states, all tree-construction insertion
modes, the complete adoption-agency algorithm, template/foreign-content
handling, DOM events/namespaces, and broad differential conformance testing
remain future work. The CLI is not a graphical browser, does not fetch URLs,
and does not execute JavaScript.

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
