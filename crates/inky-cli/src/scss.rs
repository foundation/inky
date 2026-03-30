use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::io;
use std::path::{Path, PathBuf};

// Embed all SCSS files into the binary
const SCSS_FILES: &[(&str, &str)] = &[
    ("inky.scss", include_str!("../scss/inky.scss")),
    (
        "settings/_index.scss",
        include_str!("../scss/settings/_index.scss"),
    ),
    (
        "settings/_settings.scss",
        include_str!("../scss/settings/_settings.scss"),
    ),
    ("util/_index.scss", include_str!("../scss/util/_index.scss")),
    ("util/_util.scss", include_str!("../scss/util/_util.scss")),
    ("_global.scss", include_str!("../scss/_global.scss")),
    (
        "components/_normalize.scss",
        include_str!("../scss/components/_normalize.scss"),
    ),
    (
        "components/_alignment.scss",
        include_str!("../scss/components/_alignment.scss"),
    ),
    (
        "components/_visibility.scss",
        include_str!("../scss/components/_visibility.scss"),
    ),
    (
        "components/_typography.scss",
        include_str!("../scss/components/_typography.scss"),
    ),
    (
        "components/_button.scss",
        include_str!("../scss/components/_button.scss"),
    ),
    (
        "components/_callout.scss",
        include_str!("../scss/components/_callout.scss"),
    ),
    (
        "components/_thumbnail.scss",
        include_str!("../scss/components/_thumbnail.scss"),
    ),
    (
        "components/_menu.scss",
        include_str!("../scss/components/_menu.scss"),
    ),
    (
        "components/_outlook-first.scss",
        include_str!("../scss/components/_outlook-first.scss"),
    ),
    (
        "components/_media-query.scss",
        include_str!("../scss/components/_media-query.scss"),
    ),
    (
        "components/_divider.scss",
        include_str!("../scss/components/_divider.scss"),
    ),
    (
        "components/_dark-mode.scss",
        include_str!("../scss/components/_dark-mode.scss"),
    ),
    ("grid/_grid.scss", include_str!("../scss/grid/_grid.scss")),
    (
        "grid/_block-grid.scss",
        include_str!("../scss/grid/_block-grid.scss"),
    ),
];

const EMBEDDED_ROOT: &str = "/embedded/scss";

/// Virtual filesystem serving embedded SCSS files to grass.
#[derive(Debug)]
struct EmbeddedFs {
    files: HashMap<PathBuf, Vec<u8>>,
    dirs: HashSet<PathBuf>,
}

impl EmbeddedFs {
    fn new(overrides: &[(String, String)]) -> Self {
        let mut files = HashMap::new();
        let mut dirs = HashSet::new();

        let root = PathBuf::from(EMBEDDED_ROOT);
        dirs.insert(root.clone());

        for (rel_path, content) in SCSS_FILES {
            let full_path = root.join(rel_path);

            // Add all parent directories
            let mut parent = full_path.parent();
            while let Some(p) = parent {
                if !dirs.insert(p.to_path_buf()) {
                    break;
                }
                parent = p.parent();
            }

            files.insert(full_path, content.as_bytes().to_vec());
        }

        // If there are variable overrides, inject a settings override wrapper
        if !overrides.is_empty() {
            let with_args: Vec<String> = overrides
                .iter()
                .map(|(name, value)| format!("  {}: {}", name, value))
                .collect();

            let override_scss = format!(
                "@forward 'settings' with (\n{}\n);\n",
                with_args.join(",\n")
            );

            // Replace settings/_index.scss with the override wrapper
            let index_path = root.join("settings/_index.scss");
            files.insert(index_path, override_scss.as_bytes().to_vec());
        }

        EmbeddedFs { files, dirs }
    }
}

/// Normalize a path by resolving `..` and `.` components.
fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::CurDir => {}
            other => components.push(other),
        }
    }
    components.iter().collect()
}

impl grass::Fs for EmbeddedFs {
    fn is_dir(&self, path: &Path) -> bool {
        self.dirs.contains(&normalize_path(path))
    }

    fn is_file(&self, path: &Path) -> bool {
        self.files.contains_key(&normalize_path(path))
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        let normalized = normalize_path(path);
        self.files.get(&normalized).cloned().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, format!("{}", normalized.display()))
        })
    }

    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        Ok(normalize_path(path))
    }
}

/// Extract SCSS variable overrides from `<style type="text/scss">` blocks and
/// `<link rel="stylesheet" href="*.scss">` tags.
/// Returns (html_with_scss_elements_removed, vec_of_variable_overrides).
pub fn extract_scss_overrides(
    html: &str,
    base_path: Option<&Path>,
) -> (String, Vec<(String, String)>) {
    let html_comment_re = Regex::new(r"(?s)<!--.*?-->").unwrap();
    let style_re =
        Regex::new(r#"(?si)<style\s+type\s*=\s*["']text/scss["']\s*>(.*?)</style>"#).unwrap();
    let link_re =
        Regex::new(r#"<link\s+[^>]*href\s*=\s*["']([^"']+\.scss)["'][^>]*/?\s*>"#).unwrap();
    let block_comment_re = Regex::new(r"(?s)/\*.*?\*/").unwrap();
    let line_comment_re = Regex::new(r"//[^\n]*").unwrap();
    let var_re = Regex::new(r#"(\$[\w-]+)\s*:\s*([^;]+)\s*;"#).unwrap();
    let mut overrides = Vec::new();

    // Strip HTML comments so we don't extract from commented-out examples
    let html_no_comments = html_comment_re.replace_all(html, "");

    // Extract from inline <style type="text/scss"> blocks
    for cap in style_re.captures_iter(&html_no_comments) {
        let block = block_comment_re.replace_all(&cap[1], "");
        let block = line_comment_re.replace_all(&block, "");
        for var_cap in var_re.captures_iter(&block) {
            overrides.push((var_cap[1].to_string(), var_cap[2].trim().to_string()));
        }
    }

    // Extract from linked .scss files
    if let Some(base) = base_path {
        for cap in link_re.captures_iter(&html_no_comments) {
            let href = &cap[1];
            let scss_path = base.join(href);
            match std::fs::read_to_string(&scss_path) {
                Ok(content) => {
                    let content = block_comment_re.replace_all(&content, "");
                    let content = line_comment_re.replace_all(&content, "");
                    for var_cap in var_re.captures_iter(&content) {
                        overrides.push((var_cap[1].to_string(), var_cap[2].trim().to_string()));
                    }
                }
                Err(e) => {
                    eprintln!(
                        "  warning: Failed to read SCSS file '{}' (resolved to '{}'): {}",
                        href,
                        scss_path.display(),
                        e
                    );
                }
            }
        }
    }

    let cleaned = style_re.replace_all(html, "").to_string();
    let cleaned = link_re.replace_all(&cleaned, "").to_string();
    (cleaned, overrides)
}

/// Compile the embedded Inky framework SCSS, optionally with variable overrides.
pub fn compile_framework_scss(overrides: &[(String, String)]) -> Result<String, Box<grass::Error>> {
    let embedded_fs = EmbeddedFs::new(overrides);
    let entry_path = format!("{}/inky.scss", EMBEDDED_ROOT);
    let options = grass::Options::default()
        .style(grass::OutputStyle::Compressed)
        .fs(&embedded_fs);

    let css = grass::from_path(&entry_path, &options)?;
    Ok(css.replace(" !important", "!important"))
}

/// Inject compiled CSS into HTML as a `<style>` block.
/// Places it in `<head>` if present, otherwise before the first tag.
pub fn inject_css_into_html(html: &str, css: &str) -> String {
    let style_block = format!("<style type=\"text/css\">\n{}</style>", css);

    // Try to insert before </head>
    if let Some(pos) = html.to_lowercase().find("</head>") {
        let mut result = String::with_capacity(html.len() + style_block.len());
        result.push_str(&html[..pos]);
        result.push_str(&style_block);
        result.push('\n');
        result.push_str(&html[pos..]);
        return result;
    }

    // Try to insert after <body> or <body ...>
    let body_re = Regex::new(r"(?i)<body[^>]*>").unwrap();
    if let Some(m) = body_re.find(html) {
        let mut result = String::with_capacity(html.len() + style_block.len());
        result.push_str(&html[..m.end()]);
        result.push('\n');
        result.push_str(&style_block);
        result.push_str(&html[m.end()..]);
        return result;
    }

    // Fallback: prepend
    format!("{}\n{}", style_block, html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_framework_no_overrides() {
        let css = compile_framework_scss(&[]).unwrap();
        assert!(
            css.len() > 1000,
            "CSS output too small: {} bytes",
            css.len()
        );
        assert!(css.contains("table.button"), "Should contain button styles");
        assert!(
            css.contains(".block-grid"),
            "Should contain block-grid styles"
        );
    }

    #[test]
    fn test_compile_framework_with_overrides() {
        let overrides = vec![("$primary-color".to_string(), "#ff0000".to_string())];
        let css = compile_framework_scss(&overrides).unwrap();
        assert!(
            css.contains("#ff0000") || css.contains("red"),
            "Should use overridden primary color"
        );
        // Default primary is #2199e8 — make sure it's NOT in the output
        assert!(
            !css.contains("#2199e8"),
            "Should not contain default primary color"
        );
    }

    #[test]
    fn test_extract_scss_overrides() {
        let html = r#"<html>
<head>
<style type="text/scss">
$primary-color: #ff0000;
$global-width: 640px;
</style>
</head>
<body><p>Hello</p></body>
</html>"#;

        let (cleaned, overrides) = extract_scss_overrides(html, None);
        assert_eq!(overrides.len(), 2);
        assert_eq!(overrides[0].0, "$primary-color");
        assert_eq!(overrides[0].1, "#ff0000");
        assert_eq!(overrides[1].0, "$global-width");
        assert_eq!(overrides[1].1, "640px");
        assert!(!cleaned.contains("text/scss"));
        assert!(cleaned.contains("<p>Hello</p>"));
    }

    #[test]
    fn test_extract_no_scss_blocks() {
        let html = "<html><body><p>No scss here</p></body></html>";
        let (cleaned, overrides) = extract_scss_overrides(html, None);
        assert!(overrides.is_empty());
        assert_eq!(cleaned, html);
    }

    #[test]
    fn test_inject_css_into_head() {
        let html = "<html><head><title>Test</title></head><body></body></html>";
        let result = inject_css_into_html(html, "body { color: red; }");
        assert!(result.contains("<style type=\"text/css\">\nbody { color: red; }</style>\n</head>"));
    }

    #[test]
    fn test_inject_css_no_head() {
        let html = "<html><body><p>Hello</p></body></html>";
        let result = inject_css_into_html(html, "p { color: blue; }");
        assert!(result.contains("<style type=\"text/css\">"));
        assert!(result.contains("p { color: blue; }"));
    }

    #[test]
    fn test_extract_scss_from_linked_file() {
        // Create a temp SCSS file
        let dir = std::env::temp_dir().join("inky-test-scss");
        std::fs::create_dir_all(&dir).unwrap();
        let scss_file = dir.join("theme.scss");
        std::fs::write(
            &scss_file,
            "$primary-color: #cc0000;\n$global-width: 700px;\n",
        )
        .unwrap();

        let html =
            r#"<html><head><link rel="stylesheet" href="theme.scss"></head><body></body></html>"#;
        let (cleaned, overrides) = extract_scss_overrides(html, Some(&dir));

        assert_eq!(overrides.len(), 2);
        assert_eq!(overrides[0].0, "$primary-color");
        assert_eq!(overrides[0].1, "#cc0000");
        assert_eq!(overrides[1].0, "$global-width");
        assert_eq!(overrides[1].1, "700px");
        assert!(!cleaned.contains("theme.scss"));

        // Cleanup
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_extract_scss_comments_ignored() {
        let html = r#"<style type="text/scss">
/* $primary-color: #ff0000; */
$global-width: 640px;
</style>"#;
        let (_, overrides) = extract_scss_overrides(html, None);
        assert_eq!(overrides.len(), 1);
        assert_eq!(overrides[0].0, "$global-width");
    }
}
