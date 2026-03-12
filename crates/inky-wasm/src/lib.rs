use wasm_bindgen::prelude::*;
use inky_core::{Config, Inky};
use inky_core::validate::{self, Severity};

/// Transform Inky HTML into email-safe table markup.
#[wasm_bindgen]
pub fn transform(html: &str) -> String {
    Inky::new().transform(html)
}

/// Transform with a custom column count.
#[wasm_bindgen]
pub fn transform_with_config(html: &str, column_count: u32) -> String {
    let config = Config {
        column_count,
        ..Default::default()
    };
    Inky::with_config(config).transform(html)
}

/// Transform Inky HTML and inline CSS from `<style>` blocks.
/// Falls back to plain transform if inlining fails.
#[wasm_bindgen]
pub fn transform_inline(html: &str) -> String {
    match Inky::new().transform_and_inline(html, None) {
        Ok(r) => r,
        Err(_) => Inky::new().transform(html),
    }
}

/// Migrate v1 Inky syntax to v2.
/// Returns the migrated HTML string.
#[wasm_bindgen]
pub fn migrate(html: &str) -> String {
    inky_core::migrate::migrate(html).html
}

/// Migrate v1 syntax and return a JSON object with `html` and `changes` fields.
#[wasm_bindgen]
pub fn migrate_with_details(html: &str) -> String {
    let result = inky_core::migrate::migrate(html);
    let changes: Vec<String> = result.changes.iter().map(|c| c.description.clone()).collect();
    // Return as JSON manually to avoid serde dependency
    let changes_json: Vec<String> = changes.iter().map(|c| format!("\"{}\"", escape_json(c))).collect();
    format!(
        r#"{{"html":"{}","changes":[{}]}}"#,
        escape_json(&result.html),
        changes_json.join(",")
    )
}

/// Validate an Inky template and return diagnostics as JSON.
///
/// Returns a JSON array of objects with `severity`, `rule`, and `message` fields.
#[wasm_bindgen]
pub fn validate(html: &str) -> String {
    let config = Config::default();
    diagnostics_to_json(&validate::validate(html, &config))
}

/// Validate with a custom column count.
#[wasm_bindgen]
pub fn validate_with_config(html: &str, column_count: u32) -> String {
    let config = Config {
        column_count,
        ..Default::default()
    };
    diagnostics_to_json(&validate::validate(html, &config))
}

fn diagnostics_to_json(diagnostics: &[validate::Diagnostic]) -> String {
    let items: Vec<String> = diagnostics
        .iter()
        .map(|d| {
            let severity = match d.severity {
                Severity::Warning => "warning",
                Severity::Error => "error",
            };
            format!(
                r#"{{"severity":"{}","rule":"{}","message":"{}"}}"#,
                severity,
                escape_json(d.rule),
                escape_json(&d.message)
            )
        })
        .collect();
    format!("[{}]", items.join(","))
}

/// Get the Inky version.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Escape a string for JSON output.
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
