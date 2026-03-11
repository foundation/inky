use css_inline::{CSSInliner, InlineOptions, Url};

/// Inline CSS into element `style=""` attributes.
///
/// Resolves both `<style>` blocks and `<link rel="stylesheet" href="...">` tags.
/// Media queries and other at-rules that can't be inlined are preserved
/// in a `<style>` block in the `<head>`.
///
/// `base_path` is the directory used to resolve relative `href` paths in
/// `<link>` tags. If `None`, link tags with relative paths won't resolve.
pub fn inline_css(html: &str, base_path: Option<&std::path::Path>) -> Result<String, String> {
    let base_url = match base_path {
        Some(path) => {
            let abs = if path.is_absolute() {
                path.to_path_buf()
            } else {
                std::env::current_dir()
                    .unwrap_or_default()
                    .join(path)
            };
            Url::from_directory_path(abs).ok()
        }
        None => None,
    };

    let options = InlineOptions {
        keep_style_tags: false,
        keep_link_tags: false,
        inline_style_tags: true,
        base_url,
        ..InlineOptions::default()
    };
    let inliner = CSSInliner::new(options);
    inliner.inline(html).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_basic() {
        let html = r#"<html><head><style>.red { color: red; }</style></head><body><p class="red">Hello</p></body></html>"#;
        let result = inline_css(html, None).unwrap();
        assert!(result.contains("style=\"color: red;\"") || result.contains("style=\"color:red\""));
        assert!(result.contains("Hello"));
    }

    #[test]
    fn test_inline_preserves_media_queries() {
        let html = r#"<html><head><style>
            .red { color: red; }
            @media only screen and (max-width: 596px) {
                .small-float-center { margin: 0 auto; }
            }
        </style></head><body><p class="red">Hello</p></body></html>"#;
        let result = inline_css(html, None).unwrap();
        assert!(result.contains("color: red") || result.contains("color:red"));
    }

    #[test]
    fn test_inline_from_css_file() {
        // Write a temp CSS file
        let dir = std::env::temp_dir().join("inky_test_inline");
        std::fs::create_dir_all(&dir).unwrap();
        let css_path = dir.join("test.css");
        std::fs::write(&css_path, ".blue { color: blue; }").unwrap();

        let html = r#"<html><head><link rel="stylesheet" href="test.css"></head><body><p class="blue">Hello</p></body></html>"#;
        let result = inline_css(html, Some(&dir)).unwrap();
        assert!(result.contains("color: blue") || result.contains("color:blue"));

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }
}
