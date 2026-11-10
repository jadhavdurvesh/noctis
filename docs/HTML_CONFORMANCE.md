# HTML Conformance Testing

## Local regression corpus

Run the checked-in fragment and document fixtures with:

```sh
cargo test -p noctis-html
```

Each `*.input.html` file has a matching `*.expected.html` file. The fixture
harness compares the serialized output of NOCTIS's fragment parser. Files
prefixed with `document-` are parsed through `parse_document`.

The repository's local fixtures are a curated regression set authored for
NOCTIS. They currently cover named and multi-codepoint character references,
attribute decoding, implied list-item closures, raw script text, void
elements, table foster parenting, misnested inline formatting, and document
skeleton construction.

## External reference suites

These upstream projects are compatibility references, not engine
dependencies:

- [html5lib-tests](https://github.com/html5lib/html5lib-tests) provides
  tokenizer fixtures and documents their JSON test format in
  [tokenizer/README.md](https://github.com/html5lib/html5lib-tests/blob/master/tokenizer/README.md).
- [Web Platform Tests: HTML parsing](https://github.com/web-platform-tests/wpt/tree/master/html/syntax/parsing)
  is the current home for HTML parser tree-construction tests.

The html5lib-tests repository states that its tree-construction tests are now
maintained in Web Platform Tests. Its repository includes an MIT license.
NOCTIS does not copy those test files into this curated corpus at this stage;
the links identify the upstream compatibility references.

## Compatibility policy

A local fixture passing means only that NOCTIS matches that fixture's expected
serialization. It is not a claim of full HTML Living Standard conformance.
The next expansion is a carefully selected, licensed upstream test-data
adapter for tokenizer tokens and tree-construction outputs, with unsupported
features reported as skipped rather than counted as passed.
