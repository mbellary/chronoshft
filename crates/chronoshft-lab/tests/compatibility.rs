//! Traceability of the complete frozen Python regression inventory to Rust tests.
use chronoshft_lab::runtime::Git;
use regex::Regex;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
};
#[test]
fn every_frozen_python_regression_has_an_existing_native_test_counterpart() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let reference: Value = serde_json::from_slice(
        &fs::read(root.join("tests/compatibility/python-v1/manifest.json")).unwrap(),
    )
    .unwrap();
    let coverage: Value = serde_json::from_slice(
        &fs::read(root.join("tests/compatibility/rust-coverage.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(coverage["reference_commit"], reference["reference_commit"]);
    let git = Git::new(&root).unwrap();
    let original = Regex::new(r"(?m)^    def (test_[A-Za-z0-9_]+)\(").unwrap();
    let test = Regex::new(r"#\[test\]\s*fn ([A-Za-z0-9_]+)\(").unwrap();
    let mut expected = BTreeSet::new();
    for module in reference["python_test_modules"].as_array().unwrap() {
        let module = module.as_str().unwrap();
        let source = String::from_utf8(
            git.blob(reference["reference_commit"].as_str().unwrap(), module)
                .unwrap(),
        )
        .unwrap();
        for c in original.captures_iter(&source) {
            assert!(expected.insert((module.to_owned(), c[1].to_owned())));
        }
    }
    assert_eq!(expected.len(), 110);
    let mut native = BTreeMap::new();
    let mut actual = BTreeSet::new();
    for row in coverage["cases"].as_array().unwrap() {
        let module = row["python_module"].as_str().unwrap();
        let name = row["python_case"].as_str().unwrap();
        assert!(
            actual.insert((module.to_owned(), name.to_owned())),
            "duplicate {module}:{name}"
        );
        let cases = row["rust_cases"].as_array().unwrap();
        assert!(!cases.is_empty());
        for case in cases {
            let target = case["target"].as_str().unwrap();
            assert!(target
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_'));
            if !native.contains_key(target) {
                let source = fs::read_to_string(
                    root.join(format!("crates/chronoshft-lab/tests/{target}.rs")),
                )
                .unwrap();
                native.insert(
                    target,
                    test.captures_iter(&source)
                        .map(|c| c[1].to_owned())
                        .collect::<BTreeSet<_>>(),
                );
            }
            assert!(
                native[target].contains(case["case"].as_str().unwrap()),
                "missing counterpart for {module}:{name}: {case}"
            );
        }
    }
    assert_eq!(actual, expected);
}
