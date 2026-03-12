use inky_core::{ComponentNames, Config, Inky};
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct FixtureFile {
    tests: Vec<TestCase>,
}

#[derive(Deserialize)]
struct TestCase {
    name: String,
    input: String,
    expected: String,
}

/// Normalize HTML for comparison: collapse whitespace, strip v2 additions, and trim.
fn normalize_html(html: &str) -> String {
    // Strip role="presentation" (v2 addition not in v1 fixture expected output)
    let html = html.replace(r#" role="presentation""#, "");
    // Remove newlines, collapse multiple spaces to one, trim
    let mut result = String::new();
    let mut last_was_space = false;
    for ch in html.chars() {
        if ch.is_whitespace() {
            if !last_was_space {
                result.push(' ');
                last_was_space = true;
            }
        } else {
            result.push(ch);
            last_was_space = false;
        }
    }
    result.trim().to_string()
}

fn run_fixtures(path: &str) {
    let content = fs::read_to_string(path).expect(&format!("Failed to read {}", path));
    let fixtures: FixtureFile = serde_json::from_str(&content).expect("Failed to parse JSON");

    let mut failures = Vec::new();

    // Use v1 config since fixtures use v1 syntax
    let config = Config {
        components: ComponentNames::v1(),
        ..Config::default()
    };
    let inky = Inky::with_config(config);

    for test in &fixtures.tests {
        let result = inky.transform(&test.input);
        let normalized_result = normalize_html(&result);
        let normalized_expected = normalize_html(&test.expected);

        if normalized_result != normalized_expected {
            failures.push(format!(
                "\n--- FAILED: {} ---\n  input:    {}\n  expected: {}\n  got:      {}",
                test.name, test.input, normalized_expected, normalized_result
            ));
        }
    }

    if !failures.is_empty() {
        panic!(
            "{} fixture(s) failed:{}\n",
            failures.len(),
            failures.join("")
        );
    }
}

#[test]
fn test_component_fixtures() {
    run_fixtures("../../tests/fixtures/components.json");
}

#[test]
fn test_grid_fixtures() {
    run_fixtures("../../tests/fixtures/grid.json");
}
