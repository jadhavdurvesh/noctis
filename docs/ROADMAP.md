# NOCTIS Roadmap

## Phase 0 — Foundation
- [x] Project identity and no-existing-engine rule.
- [x] Rust workspace and subsystem boundaries.
- [x] Project-specific restricted source license.
- [x] Parser CLI harness and example document.
- [x] Cross-platform CI for Linux and Windows.
- [x] Zero-runtime-dependency Rust workspace and Cargo lockfile.

## Phase 1 — HTML and DOM
- [x] Tokenize basic start/end tags, text, comments, and doctypes.
- [x] Parse quoted, unquoted, and boolean attributes; preserve first duplicate.
- [x] Decode the full 2,231-entry HTML named-character-reference table.
- [x] Decode numeric references, including Windows-1252 remapping and invalid values.
- [x] Apply the attribute ambiguous-ampersand rule for legacy references.
- [x] Handle raw-text elements, RCDATA elements, and void elements.
- [x] Handle selected implied end tags and malformed closing tags.
- [x] Build an `html/head/body` document skeleton with scoped document insertion modes.
- [x] Implement selected table recovery: foster-parent stray table content and insert
      implicit `tbody` / `tr` in common cases.
- [x] Implement a basic inline-formatting reopen repair for common misnested tags.
- [x] Add arena-backed DOM IDs, parent/child/previous-sibling/next-sibling queries,
      reparenting, insert-before, removal, attributes, subtree search, and text content.
- [x] Add basic HTML serialization and diagnostic tree output.
- [x] Add targeted regression tests for entities, document structure, table recovery,
      formatting recovery, DOM identity, and mutation.
- [x] Preprocess CRLF/CR line endings and replace literal NUL input characters.
- [x] Add selected tokenizer recovery for bogus comments, abrupt empty comments,
      the `</br>` start-tag reprocessing rule, and HTML self-closing flags.
- [x] Apply attribute-context character-reference handling to quoted and unquoted
      attribute values.
- [x] Add a deterministic file-pair conformance harness and seven checked-in HTML
      serialization fixtures, including document-mode parsing.
- [ ] Add an adapter for upstream html5lib tokenizer fixtures and selected WPT
      tree-construction fixtures, with explicit pass/fail/skip accounting.
- [ ] Implement all HTML tokenizer states and exact error-recovery behavior.
- [ ] Implement every HTML tree-construction insertion mode, including the complete
      adoption-agency algorithm, foster-parenting rules, templates, and foreign content.
- [ ] Implement document-mode/quirks handling and standards-complete fragment contexts.
- [ ] Extend the DOM with document fragments, node types, namespaces, events, and
      fully standards-compatible hierarchy/mutation constraints.
- [ ] Differential-test against upstream HTML parser conformance suites (including
      broad Web Platform Tests coverage).

The checked HTML items describe the implementation that exists in this repository;
they do not claim full HTML Living Standard conformance.

## Phase 2 — CSS
- [x] CSS tokenizer subset: identifiers, strings, numbers, dimensions, percentages,
      functions, punctuation, comments and a subset of escapes.
- [x] Declaration-list parser with custom-property name preservation and
      `!important` recognition.
- [x] Compound type/ID/class/attribute selector parsing and matching.
- [x] Basic style-rule parsing and scoped specificity/source-order/importance cascade.
- [ ] Selector combinators, pseudo-classes and pseudo-elements.
- [ ] Complete CSS Syntax Level 3 tokenization, parser states and error recovery.
- [ ] Full cascade origins/layers/scope, media evaluation and inheritance.
- [ ] Shorthand expansion, computed values and custom-property substitution.
- [ ] Box model, formatting contexts and layout integration.
- [ ] CSS conformance tests against upstream CSS Syntax and Selectors fixtures.

The checked CSS items describe the implemented subset only; see
[CSS_IMPLEMENTATION.md](CSS_IMPLEMENTATION.md).

## Phase 3 — Rendering
- [ ] Layout tree and geometry.
- [ ] Text shaping and font integration.
- [ ] Display list, painting, compositing.
- [ ] Images, scrolling, and hit testing.

## Phase 4 — Runtime and network
- [ ] Independent JavaScript virtual machine.
- [ ] Host bindings and event loop.
- [x] Initial bounded HTTP/HTTPS fetch layer with URL validation, TLS transport,
      request timeout, redirect limit and response-size cap.
- [ ] Cookies, cache, content encodings and full charset support.
- [ ] Process isolation, permissions, and sandboxing.

## Phase 5 — Browser product and packaging
- [x] First terminal browser alpha: HTTP/HTTPS navigation, HTML parsing,
      readable document extraction, numbered links and history.
- [x] Desktop CLI package workflow for Windows, Linux and macOS.
- [x] Android Engine Lab APK workflow for arm64-v8a, armeabi-v7a and x86_64.
- [x] Tagged GitHub Release workflow with SHA-256 asset checksums.
- [ ] Graphical desktop browser shell and native window lifecycle.
- [ ] Graphical browser shell, tabs, profiles, downloads, history UI and bookmarks.
- [ ] MSI/DMG/AppImage/DEB installer formats.
- [ ] Production-signed Android APK/AAB and update/signing process.
- [ ] SAND OS backend.
- [ ] Web-platform conformance and browser security hardening.

NOCTIS is experimental and is not suitable for browsing untrusted internet content.
