use inky_core::{migrate, ComponentNames, Config, Inky};
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

/// Normalize HTML for comparison: collapse whitespace and trim.
fn normalize_html(html: &str) -> String {
    let mut result = String::new();
    let mut last_was_space = false;
    for ch in html.chars() {
        if ch == '\n' {
            // Preserve explicit newlines (used in expander output)
            result.push('\n');
            last_was_space = false;
        } else if ch.is_whitespace() {
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

/// Normalize v1 fixture HTML: strip role="presentation" since v1 expected output doesn't include it.
fn normalize_v1_html(html: &str) -> String {
    let html = html.replace(r#" role="presentation""#, "");
    normalize_html(&html)
}

fn run_v1_fixtures(path: &str) {
    let content = fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read {}", path));
    let fixtures: FixtureFile = serde_json::from_str(&content).expect("Failed to parse JSON");

    let config = Config {
        components: ComponentNames::v1(),
        ..Config::default()
    };
    let inky = Inky::with_config(config);

    let mut failures = Vec::new();

    for test in &fixtures.tests {
        let result = inky.transform(&test.input);
        let normalized_result = normalize_v1_html(&result);
        let normalized_expected = normalize_v1_html(&test.expected);

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

fn run_v2_fixtures(path: &str) {
    let content = fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read {}", path));
    let fixtures: FixtureFile = serde_json::from_str(&content).expect("Failed to parse JSON");

    // Default config uses v2 component names
    let inky = Inky::new();

    let mut failures = Vec::new();

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

fn run_migration_fixtures(path: &str) {
    let content = fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read {}", path));
    let fixtures: FixtureFile = serde_json::from_str(&content).expect("Failed to parse JSON");

    let mut failures = Vec::new();

    for test in &fixtures.tests {
        let result = migrate::migrate(&test.input).html;
        if result != test.expected {
            failures.push(format!(
                "\n--- FAILED: {} ---\n  input:    {}\n  expected: {}\n  got:      {}",
                test.name, test.input, test.expected, result
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

// --- v1 fixture tests (legacy syntax) ---

#[test]
fn test_v1_component_fixtures() {
    run_v1_fixtures("../../tests/fixtures/components.json");
}

#[test]
fn test_v1_grid_fixtures() {
    run_v1_fixtures("../../tests/fixtures/grid.json");
}

// --- v2 fixture tests (modern syntax) ---

#[test]
fn test_v2_component_fixtures() {
    run_v2_fixtures("../../tests/fixtures/v2-components.json");
}

#[test]
fn test_v2_grid_fixtures() {
    run_v2_fixtures("../../tests/fixtures/v2-grid.json");
}

// --- v2 nesting/integration tests ---

#[test]
fn test_v2_nesting_fixtures() {
    run_v2_fixtures("../../tests/fixtures/v2-nesting.json");
}

// --- Migration fixture tests (v1 → v2) ---

#[test]
fn test_migration_fixtures() {
    run_migration_fixtures("../../tests/fixtures/migration.json");
}
