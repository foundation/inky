//! Framework SCSS compilation: embeds the Inky SCSS tree, extracts user
//! SCSS from templates, compiles via grass, and injects the CSS.

use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::io;
use std::path::{Path, PathBuf};

use crate::InkyError;

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
const ENTRY_FILENAME: &str = "__entry.scss";

/// Virtual filesystem serving embedded SCSS files to grass.
#[derive(Debug)]
struct EmbeddedFs {
    files: HashMap<PathBuf, Vec<u8>>,
    dirs: HashSet<PathBuf>,
}

impl EmbeddedFs {
    /// Build the embedded FS with a synthetic entry file that prepends
    /// the user's SCSS and then imports the framework.
    fn with_entry(user_scss: &str) -> Self {
        let mut files = HashMap::new();
        let mut dirs = HashSet::new();

        let root = PathBuf::from(EMBEDDED_ROOT);
        dirs.insert(root.clone());

        for (rel_path, content) in SCSS_FILES {
            let full_path = root.join(rel_path);

            let mut parent = full_path.parent();
            while let Some(p) = parent {
                if !dirs.insert(p.to_path_buf()) {
                    break;
                }
                parent = p.parent();
            }

            files.insert(full_path, content.as_bytes().to_vec());
        }

        let entry_contents = format!("{}\n@import 'inky';\n", user_scss);
        files.insert(root.join(ENTRY_FILENAME), entry_contents.into_bytes());

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

/// Extract raw SCSS text from `<style type="text/scss">` blocks and
/// `<link rel="stylesheet" href="*.scss">` tags, and strip those nodes from the HTML.
///
/// Linked files are concatenated first, then inline `<style>` blocks — so inline
/// SCSS can override linked theme values.
///
/// Returns (html_with_scss_elements_removed, concatenated_user_scss, warnings).
pub fn extract_scss_sources(html: &str, base_path: Option<&Path>) -> (String, String, Vec<String>) {
    let mut warnings: Vec<String> = Vec::new();

    let html_comment_re = Regex::new(r"(?s)<!--.*?-->").unwrap();
    let style_re =
        Regex::new(r#"(?si)<style\s+type\s*=\s*["']text/scss["']\s*>(.*?)</style>"#).unwrap();
    let link_re =
        Regex::new(r#"<link\s+[^>]*href\s*=\s*["']([^"']+\.scss)["'][^>]*/?\s*>"#).unwrap();

    // Strip HTML comments so we don't extract from commented-out examples.
    let html_no_comments = html_comment_re.replace_all(html, "");

    let mut linked = String::new();
    if let Some(base) = base_path {
        for cap in link_re.captures_iter(&html_no_comments) {
            let href = &cap[1];
            let scss_path = base.join(href);
            match std::fs::read_to_string(&scss_path) {
                Ok(content) => {
                    linked.push_str(&content);
                    if !content.ends_with('\n') {
                        linked.push('\n');
                    }
                }
                Err(e) => {
                    warnings.push(format!(
                        "Failed to read SCSS file '{}' (resolved to '{}'): {}",
                        href,
                        scss_path.display(),
                        e
                    ));
                }
            }
        }
    }

    let mut inline = String::new();
    for cap in style_re.captures_iter(&html_no_comments) {
        inline.push_str(&cap[1]);
        if !cap[1].ends_with('\n') {
            inline.push('\n');
        }
    }

    let mut combined = linked;
    combined.push_str(&inline);

    let cleaned = style_re.replace_all(html, "").to_string();
    let cleaned = link_re.replace_all(&cleaned, "").to_string();
    (cleaned, combined, warnings)
}

/// Compile the embedded Inky framework SCSS together with user SCSS.
///
/// The user SCSS is prepended before `@import 'inky';`, so user variable
/// definitions (without `!default`) override the framework's `!default` values.
/// Arbitrary SCSS — maps, `@each` loops, custom selectors — is supported.
pub fn compile_framework_scss(user_scss: &str) -> Result<String, InkyError> {
    let embedded_fs = EmbeddedFs::with_entry(user_scss);
    let entry_path = format!("{}/{}", EMBEDDED_ROOT, ENTRY_FILENAME);
    let options = grass::Options::default()
        .style(grass::OutputStyle::Compressed)
        .fs(&embedded_fs);

    let css = grass::from_path(&entry_path, &options).map_err(InkyError::Scss)?;
    Ok(css.replace(" !important", "!important"))
}

/// Inject compiled CSS into HTML as a `<style>` block.
/// Places it in `<head>` if present, otherwise before the first tag.
pub fn inject_css_into_html(html: &str, css: &str) -> String {
    let style_block = format!("<style type=\"text/css\">\n{}</style>", css);

    // Try to insert before </head>. Case-insensitive regex rather than
    // to_lowercase(): lowercasing can change byte lengths (e.g. 'İ'),
    // which would misplace the offset or split a char boundary.
    let head_re = Regex::new(r"(?i)</head>").unwrap();
    if let Some(pos) = head_re.find(html).map(|m| m.start()) {
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
        let css = compile_framework_scss("").unwrap();
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
        let css = compile_framework_scss("$primary-color: #ff0000;").unwrap();
        assert!(
            css.contains("#ff0000") || css.contains("red"),
            "Should use overridden primary color"
        );
        // Default primary is #1a73b5 — make sure it's NOT in the output
        assert!(
            !css.contains("#1a73b5"),
            "Should not contain default primary color"
        );
    }

    #[test]
    fn test_compile_user_map_and_each() {
        // Regression: user SCSS with a map + @each loop that references
        // other user variables must compile and emit the generated rules.
        let user = r#"
$white: #FFFFFF;
$magenta: #FF00FF;
$wow: (
    "white": $white,
    "magenta": $magenta,
);
@each $name, $c in $wow {
  .bg-#{$name} { background-color: $c; }
}
"#;
        let css = compile_framework_scss(user).unwrap();
        assert!(
            css.contains(".bg-white") && css.contains("#fff"),
            "Expected .bg-white with white background in CSS"
        );
        assert!(
            css.contains(".bg-magenta") && css.contains("#f0f"),
            "Expected .bg-magenta with magenta background in CSS"
        );
    }

    #[test]
    fn test_extract_scss_sources() {
        let html = r#"<html>
<head>
<style type="text/scss">
$primary-color: #ff0000;
$global-width: 640px;
</style>
</head>
<body><p>Hello</p></body>
</html>"#;

        let (cleaned, scss, _warnings) = extract_scss_sources(html, None);
        assert!(scss.contains("$primary-color: #ff0000;"));
        assert!(scss.contains("$global-width: 640px;"));
        assert!(!cleaned.contains("text/scss"));
        assert!(cleaned.contains("<p>Hello</p>"));
    }

    #[test]
    fn test_extract_no_scss_blocks() {
        let html = "<html><body><p>No scss here</p></body></html>";
        let (cleaned, scss, _warnings) = extract_scss_sources(html, None);
        assert!(scss.is_empty());
        assert_eq!(cleaned, html);
    }

    #[test]
    fn test_extract_scss_from_linked_file() {
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
        let (cleaned, scss, _warnings) = extract_scss_sources(html, Some(&dir));

        assert!(scss.contains("$primary-color: #cc0000;"));
        assert!(scss.contains("$global-width: 700px;"));
        assert!(!cleaned.contains("theme.scss"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_extract_linked_then_inline_order() {
        // Linked files should come before inline <style> blocks in the
        // concatenated source, so inline blocks can override linked values.
        let dir = std::env::temp_dir().join("inky-test-order");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("theme.scss"), "$primary-color: #aa0000;\n").unwrap();

        let html = r#"<html><head>
<link rel="stylesheet" href="theme.scss">
<style type="text/scss">$primary-color: #00aa00;</style>
</head><body></body></html>"#;
        let (_, scss, _warnings) = extract_scss_sources(html, Some(&dir));

        let linked_pos = scss.find("#aa0000").expect("linked value missing");
        let inline_pos = scss.find("#00aa00").expect("inline value missing");
        assert!(linked_pos < inline_pos, "linked should precede inline");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_extract_missing_linked_file_warns() {
        let dir = std::env::temp_dir().join("inky-test-scss-missing");
        std::fs::create_dir_all(&dir).unwrap();
        let html =
            r#"<html><head><link rel="stylesheet" href="nope.scss"></head><body></body></html>"#;
        let (_, scss, warnings) = extract_scss_sources(html, Some(&dir));
        assert!(scss.is_empty());
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("nope.scss"));
        std::fs::remove_dir_all(&dir).ok();
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
    fn inject_css_multibyte_head_offset() {
        // 'İ' lowercases to 2 chars / 3 bytes; byte offsets from a
        // lowercased copy would land mid-char in the original.
        let html = "<html><head><title>İstanbul İİİ</title></head><body></body></html>";
        let out = inject_css_into_html(html, "body{color:#000}\n");
        let style_pos = out.find("<style").unwrap();
        let head_close = out.find("</head>").unwrap();
        assert!(style_pos < head_close);
        assert!(out.contains("İstanbul"));
    }
}
