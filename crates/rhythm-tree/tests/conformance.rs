//! The Rust core against the frozen JavaScript reference, case by case.

use rhythm_tree::{Level, decode, encode};
use serde_json::Value;

/// The fixture cases whose every feature the core has.
const CODEC_CASES: &[usize] = &[0, 1, 2, 3, 10, 11, 12, 13, 14, 15, 16];

#[test]
fn codec_matches_the_reference() {
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/conformance.json")).expect("fixture is JSON");
    for &i in CODEC_CASES {
        let case = &fixture["cases"][i];
        let decoded = decode(case["input"].as_str().expect("input is text"));
        assert_eq!(
            encode(&decoded.score),
            case["encoded"].as_str().expect("encoded is text"),
            "case {i}"
        );
        let issues: Vec<(&str, String)> = decoded
            .issues
            .iter()
            .map(|issue| {
                let level = match issue.level() {
                    Level::Error => "error",
                    Level::Warn => "warn",
                };
                (level, issue.to_string())
            })
            .collect();
        let expected: Vec<(&str, String)> = case["issues"]
            .as_array()
            .expect("issues is a list")
            .iter()
            .map(|pair| {
                (
                    pair[0].as_str().expect("level is text"),
                    pair[1].as_str().expect("msg is text").to_string(),
                )
            })
            .collect();
        assert_eq!(issues, expected, "case {i}");
    }
}
