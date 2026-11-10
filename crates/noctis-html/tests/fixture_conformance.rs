use std::fs;
use std::path::{Path, PathBuf};

use noctis_html::{parse_document, parse_fragment};

fn fixture_paths() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures");
    let mut fixtures = fs::read_dir(&root)
        .unwrap_or_else(|error| {
            panic!(
                "cannot read HTML fixture directory {}: {error}",
                root.display()
            )
        })
        .map(|entry| entry.expect("fixture directory entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "html")
        })
        .filter(|path| path.with_extension("tree").is_file())
        .collect::<Vec<_>>();
    fixtures.sort();
    fixtures
}

#[test]
fn html_tree_construction_matches_checked_in_fixtures() {
    let fixtures = fixture_paths();
    assert!(
        !fixtures.is_empty(),
        "no paired .html/.tree fixtures were found"
    );

    for input_path in fixtures {
        let expected_path = input_path.with_extension("tree");
        let source = fs::read_to_string(&input_path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", input_path.display()));
        let expected = fs::read_to_string(&expected_path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", expected_path.display()));

        let tree = if input_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .is_some_and(|stem| stem.starts_with("document_"))
        {
            parse_document(&source)
        } else {
            parse_fragment(&source)
        };

        assert_eq!(
            tree.dump().trim_end(),
            expected.trim_end(),
            "HTML fixture mismatch: {}",
            input_path.display()
        );
    }
}
