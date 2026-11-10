use noctis_html::{parse_document, parse_fragment};
use std::fs;
use std::path::Path;

const INPUT_SUFFIX: &str = ".input.html";
const EXPECTED_SUFFIX: &str = ".expected.html";

#[test]
fn html_parser_matches_checked_in_serialization_fixtures() {
    let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures");
    let mut inputs = fs::read_dir(&fixture_root)
        .expect("fixture directory should exist")
        .map(|entry| entry.expect("fixture directory entry").path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(INPUT_SUFFIX))
        })
        .collect::<Vec<_>>();
    inputs.sort();

    assert!(
        !inputs.is_empty(),
        "at least one HTML fixture should be present"
    );

    for input_path in inputs {
        let file_name = input_path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("fixture names must be UTF-8");
        let expected_name = file_name.replace(INPUT_SUFFIX, EXPECTED_SUFFIX);
        let expected_path = input_path.with_file_name(expected_name);
        let input = fs::read_to_string(&input_path)
            .unwrap_or_else(|error| panic!("could not read {}: {error}", input_path.display()));
        let expected = fs::read_to_string(&expected_path).unwrap_or_else(|error| {
            panic!(
                "could not read expected output {}: {error}",
                expected_path.display()
            )
        });

        // Document fixtures exercise the document parser; all other files test
        // fragment parsing. Both compare the public serializer's observable output.
        let actual = if file_name.starts_with("document-") {
            parse_document(&input).to_html()
        } else {
            parse_fragment(&input).to_html()
        };
        let expected = expected.trim_end_matches(['\n', '\r']);

        assert_eq!(
            actual,
            expected,
            "HTML conformance fixture failed: {}",
            input_path.display()
        );
    }
}
