//! The single shared per-file build path used by build, watch, serve,
//! validate, and spam-check. Owns data resolution, the core pipeline call,
//! validation, and plain-text generation. Callers own printing, writing,
//! and exit policy.

use std::path::{Path, PathBuf};

use inky_core::pipeline::{Pipeline, PipelineOptions};
use inky_core::validate::{self, Diagnostic};
use inky_core::Config;

/// Resolved data source for template merging.
pub enum DataSource {
    /// No data — merge tags pass through untouched.
    None,
    /// Single JSON file applied to all templates.
    File(serde_json::Value),
    /// Directory of per-template JSON files (e.g., data/welcome.json for welcome.inky).
    Directory(PathBuf),
}

/// Resolve merge data for a specific template file based on the data source.
pub fn resolve_data_for_file(
    file: &Path,
    input_dir: &Path,
    source: &DataSource,
) -> Option<serde_json::Value> {
    match source {
        DataSource::None => None,
        DataSource::File(data) => Some(data.clone()),
        DataSource::Directory(dir) => {
            let relative = file.strip_prefix(input_dir).ok()?;
            let json_path = dir.join(relative).with_extension("json");
            if json_path.is_file() {
                let content = std::fs::read_to_string(&json_path).ok()?;
                serde_json::from_str(&content).ok()
            } else {
                None
            }
        }
    }
}

/// One built template plus everything the caller may want to report or write.
pub struct BuiltFile {
    pub html: String,
    pub plain_text: Option<String>,
    pub diagnostics: Vec<Diagnostic>,
    pub warnings: Vec<String>,
}

pub struct Builder {
    pipeline: Pipeline,
    plain_text: bool,
}

impl Builder {
    pub fn new(config: Config, options: PipelineOptions, plain_text: bool) -> Self {
        Self {
            pipeline: Pipeline::new(config, options),
            plain_text,
        }
    }

    pub fn config(&self) -> &Config {
        self.pipeline.config()
    }

    /// Build from in-memory source (stdin or an already-read file).
    pub fn build_source(
        &self,
        html: &str,
        base_path: Option<&Path>,
        data: Option<&serde_json::Value>,
    ) -> Result<BuiltFile, String> {
        let processed = self.pipeline.process(html, base_path, data)?;

        let mut diagnostics = validate::validate_source(html, self.config());
        diagnostics.extend(validate::validate_output(&processed.html));

        let plain_text = self
            .plain_text
            .then(|| inky_core::plaintext::html_to_plain_text(&processed.html));

        Ok(BuiltFile {
            html: processed.html,
            plain_text,
            diagnostics,
            warnings: processed.warnings,
        })
    }

    /// Read a template file, resolve its per-file merge data, and build it.
    pub fn build_file(
        &self,
        file: &Path,
        input_dir: &Path,
        data: &DataSource,
    ) -> Result<BuiltFile, String> {
        let html = std::fs::read_to_string(file)
            .map_err(|e| format!("Failed to read {}: {}", file.display(), e))?;
        let file_data = resolve_data_for_file(file, input_dir, data);
        self.build_source(&html, file.parent(), file_data.as_ref())
    }
}

/// Find template files under `input_dir`, excluding anything under
/// `exclude_dir` (the output directory). Prevents `"src": ".", "dist": "dist"`
/// from re-ingesting built output (`dist/dist/dist/…`).
pub fn find_template_files(input_dir: &Path, exclude_dir: Option<&Path>) -> Vec<PathBuf> {
    let exclude = exclude_dir.map(|d| std::fs::canonicalize(d).unwrap_or_else(|_| d.to_path_buf()));
    crate::util::find_files(input_dir, crate::util::TEMPLATE_EXTENSIONS)
        .into_iter()
        .filter(|f| {
            let Some(ref ex) = exclude else { return true };
            let canonical = std::fs::canonicalize(f).unwrap_or_else(|_| f.clone());
            !canonical.starts_with(ex)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use inky_core::pipeline::PipelineOptions;
    use inky_core::Config;

    fn write(dir: &std::path::Path, name: &str, content: &str) {
        let p = dir.join(name);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(p, content).unwrap();
    }

    fn no_css_builder(plain_text: bool) -> Builder {
        Builder::new(
            Config::default(),
            PipelineOptions {
                inline_css: false,
                framework_css: false,
                ..Default::default()
            },
            plain_text,
        )
    }

    #[test]
    fn build_source_returns_html_and_diagnostics() {
        let b = no_css_builder(false);
        // <button> with no href triggers a source validation warning
        let built = b.build_source("<button>Go</button>", None, None).unwrap();
        assert!(built.html.contains(r#"class="button""#));
        assert!(
            built
                .diagnostics
                .iter()
                .any(|d| d.rule.contains("href") || d.message.contains("href")),
            "expected button-no-href diagnostic, got {:?}",
            built.diagnostics.iter().map(|d| d.rule).collect::<Vec<_>>()
        );
        assert!(built.plain_text.is_none());
    }

    #[test]
    fn plain_text_generated_when_enabled() {
        let b = no_css_builder(true);
        let built = b.build_source("<p>Hello world</p>", None, None).unwrap();
        let txt = built.plain_text.expect("plain text missing");
        assert!(txt.contains("Hello world"));
    }

    #[test]
    fn build_file_resolves_directory_data_per_template() {
        let dir = std::env::temp_dir().join("inky-builder-dirdata");
        let _ = std::fs::remove_dir_all(&dir);
        write(&dir, "src/welcome.inky", "<p>Hi {{ name }}</p>");
        write(&dir, "data/welcome.json", r#"{"name": "Joe"}"#);
        let b = no_css_builder(false);
        let built = b
            .build_file(
                &dir.join("src/welcome.inky"),
                &dir.join("src"),
                &DataSource::Directory(dir.join("data")),
            )
            .unwrap();
        assert!(
            built.html.contains("Hi Joe"),
            "directory data not applied: {}",
            built.html
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn find_template_files_excludes_output_dir_inside_input() {
        let dir = std::env::temp_dir().join("inky-builder-recursion");
        let _ = std::fs::remove_dir_all(&dir);
        write(&dir, "a.html", "<p>a</p>");
        write(&dir, "dist/a.html", "<p>built</p>");
        let files = find_template_files(&dir, Some(&dir.join("dist")));
        assert_eq!(
            files.len(),
            1,
            "dist/ contents must be excluded: {:?}",
            files
        );
        assert!(files[0].ends_with("a.html") && !files[0].to_string_lossy().contains("dist"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn unreadable_file_is_err() {
        let b = no_css_builder(false);
        let res = b.build_file(
            std::path::Path::new("/nonexistent/inky/file.html"),
            std::path::Path::new("/nonexistent/inky"),
            &DataSource::None,
        );
        assert!(res.is_err());
    }
}
