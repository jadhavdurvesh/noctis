# HTML and DOM Implementation Scope

## Current implemented subset

NOCTIS implements its own Rust parser and DOM; it does not wrap an existing
browser engine or parser.

- Complete named character-reference mapping (2,231 entries), longest-match
  decoding, multi-codepoint values, legacy aliases, numeric references, and an
  attribute-context ambiguous-ampersand check.
- Basic start/end tag, comment, doctype, attribute, text, raw-text, and RCDATA
  tokenization, with CRLF/CR normalization and NUL replacement.
- Selected tokenizer recovery for bogus comments, abrupt empty comments, the
  `</br>` reprocessing rule, and ignored self-closing flags on non-void HTML
  elements.
- Fragment construction with selected implied end tags and malformed end-tag
  recovery.
- Document skeleton construction with scoped before-html, before-head, in-head,
  after-head, and in-body routing; implicit `html`, `head`, and `body` elements.
- Selected table repair: common implied `tbody`/`tr` creation and foster
  parenting for non-whitespace text and unsupported elements in table insertion
  contexts.
- A small inline-formatting repair for common misnested formatting tags.
- Attribute-context character-reference handling for quoted and unquoted values.
- Arena-backed DOM identity with parent, child, previous/next sibling traversal,
  reparenting, insert-before, remove, attribute updates, ID/tag search, and
  text-content extraction.

## Not implemented / not yet standards-complete

This implementation is not the complete HTML Living Standard algorithm. In
particular, it does not yet implement every tokenizer state or parse-error
recovery rule; every tree-construction insertion mode; the full adoption-agency
algorithm; all table foster-parenting cases; templates; SVG/MathML foreign
content; document quirks mode; full contextual fragment parsing; or the full
DOM standard's document constraints, fragments, namespaces, events, and
mutation-observer behavior.

The inline-formatting and table recovery routines are intentionally scoped
implementations, not claims of full standards conformance.

## Verification

The Rust CI matrix runs formatting, Clippy with warnings denied, and workspace
tests on Linux and Windows. The crate includes 30 unit tests and a fixture
harness with seven checked-in input/expected-serialization pairs. The fixtures
cover named/multi-codepoint entities, implied list items, raw script text, void
elements, table foster parenting, misnested formatting, and document parsing.
The harness does not yet import upstream test files. Differential testing
against upstream html5lib tokenizer fixtures and Web Platform Tests remains
future work; see [HTML_CONFORMANCE.md](HTML_CONFORMANCE.md).

NOCTIS is experimental. Do not use this parser to browse untrusted internet
content, because the engine has no production security sandbox or mature
browser security model yet.
