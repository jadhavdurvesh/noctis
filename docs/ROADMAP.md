# NOCTIS Roadmap

## Phase 0 — Foundation
- [x] Project identity and no-existing-engine rule.
- [x] Rust workspace and subsystem boundaries.
- [x] Project-specific restricted source license.
- [x] Initial tokenizer/tree-builder slice and CLI harness.
- [x] Cross-platform CI for Linux and Windows.
- [x] Zero-external-dependency foundation and Cargo lockfile.

## Phase 1 — HTML and DOM
- [x] Tokenize basic start/end tags, text, comments, and doctypes.
- [x] Parse quoted, unquoted, and boolean attributes.
- [x] Preserve first duplicate attribute.
- [x] Decode a documented subset of named and numeric character references.
- [x] Handle raw-text elements and RCDATA elements.
- [x] Handle void elements and selected common implied end tags.
- [x] DOM queries by ID and tag name; text-content extraction.
- [x] DOM attribute mutation and child append/removal.
- [x] Basic HTML serialization and diagnostic tree output.
- [ ] Complete named character-reference table and exact HTML tokenizer states.
- [ ] Full HTML tree-construction algorithm, insertion modes, foster parenting,
      adoption agency algorithm, template contents, and document modes.
- [ ] DOM node identity, parent/sibling APIs, document fragments, events,
      namespaces, and standards-compatible mutation behavior.
- [ ] Differential testing against HTML parser conformance suites.

## Phase 2 — CSS
- [ ] CSS tokenizer and parser.
- [ ] Selectors and cascade.
- [ ] Inheritance and computed values.
- [ ] Box model and formatting contexts.

## Phase 3 — Rendering
- [ ] Layout tree and geometry.
- [ ] Text shaping and font integration.
- [ ] Display list, painting, compositing.
- [ ] Images, scrolling, and hit testing.

## Phase 4 — Runtime and network
- [ ] Independent JavaScript virtual machine.
- [ ] Host bindings and event loop.
- [ ] URL, HTTP(S), TLS, cookies, and cache.
- [ ] Process isolation, permissions, and sandboxing.

## Phase 5 — Browser product
- [ ] Desktop shell, tabs, profiles.
- [ ] Downloads, history, bookmarks, developer tools.
- [ ] Windows and Linux builds.
- [ ] SAND OS backend.
- [ ] Web-platform conformance and security testing.

A checked item means implemented in this repository, not full standards
conformance. The checked HTML features are an experimental subset; NOCTIS is
not yet suitable for browsing untrusted internet content.
