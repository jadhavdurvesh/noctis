# NOCTIS CSS Implementation Scope

## Implemented subset

The `noctis-css` crate is a first-principles Rust implementation. It is
connected to NOCTIS DOM nodes through the internal `noctis-html` crate; it does
not delegate CSS parsing or matching to another browser engine.

- Tokenizer for whitespace, identifiers, at-keywords, hashes, strings, numbers,
  percentages, dimensions, functions, punctuation and delimiters.
- Comment removal and a subset of CSS escapes.
- Declaration-list parsing with top-level semicolon handling.
- Case-normalized ordinary property names and preserved custom-property names.
- `!important` recognition.
- Compound selectors: universal/type, ID, class and attribute selectors.
- Attribute operators: `=`, `~=`, `|=`, `^=`, `$=`, and `*=`.
- Selector specificity for the implemented compound selectors.
- Basic style-rule parsing and nested collection inside `@media`,
  `@supports`, and `@layer` blocks. These group conditions are collected;
  their conditions are **not evaluated**.
- A limited per-element cascade using importance, inline declarations,
  specificity and source order.

## Not implemented

This is not a complete CSS Syntax Level 3, Selectors, or Cascading and
Inheritance implementation. It does not yet provide:

- Selector combinators (descendant, child, adjacent/general sibling).
- Pseudo-classes or pseudo-elements.
- Full escape/tokenization/error-recovery behavior or URL token behavior.
- Full qualified-rule and at-rule grammar, layer ordering, scope, container
  queries, media evaluation, or origin handling.
- Inheritance, initial/inherit/unset/revert handling, computed values,
  shorthand expansion, logical properties, or custom property substitution.
- Integration with layout, computed geometry, or paint.

## Tests

The CSS crate includes tests for tokenization, dimensions/percentages, comments
and escapes, strings/functions, declaration parsing, nested semicolons,
compound/attribute selectors, stylesheet parsing and scoped cascade behavior.

Run:

```sh
cargo test -p noctis-css
```

Passing these tests indicates support for the tested subset, not full CSS
standards conformance.
