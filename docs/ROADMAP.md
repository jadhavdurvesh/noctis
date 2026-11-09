# NOCTIS Roadmap

## Phase 0 — Foundation
- [x] Project identity and no-existing-engine rule.
- [x] Rust workspace and subsystem boundaries.
- [x] Project-specific restricted source license.
- [x] Initial tokenizer/tree-builder slice and CLI harness.
- [ ] Continuous integration and dependency audit.

## Phase 1 — HTML and DOM
- [x] Minimal tokenizer for text, tags, comments, and doctypes.
- [x] Basic nested-element tree construction.
- [ ] Attribute parsing and void-element handling.
- [ ] Character references.
- [ ] HTML tree-construction algorithm and error recovery.
- [ ] DOM traversal, mutation, and serialization.

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
conformance. Features will be expanded only with tests and documentation.
