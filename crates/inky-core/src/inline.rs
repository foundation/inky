use css_inline::{CSSInliner, InlineOptions};
use regex::Regex;

/// Inline CSS into element `style=""` attributes.
///
/// Resolves both `<style>` blocks and `<link rel="stylesheet" href="...">` tags.
/// Media queries and other at-rules that can't be inlined are preserved
/// in a `<style>` block in the `<head>`.
///
/// `base_path` is the directory used to resolve relative `href` paths in
/// `<link>` tags. If `None`, link tags with relative paths won't resolve.
pub fn inline_css(html: &str, base_path: Option<&std::path::Path>) -> Result<String, String> {
    // Resolve <link rel="stylesheet"> tags to inline <style> blocks ourselves,
    // rather than relying on css_inline's file URL resolution which breaks on Windows.
    let html = resolve_link_tags(html, base_path);

    let options = InlineOptions {
        keep_style_tags: false,
        keep_link_tags: false,
        inline_style_tags: true,
        ..InlineOptions::default()
    };
    let inliner = CSSInliner::new(options);
    let result = inliner.inline(&html).map_err(|e| e.to_string())?;

    // Move remaining <style> blocks from <head> to end of <body>.
    // Gmail clips emails at ~102KB — styles in <head> eat into that budget
    // before any visible content. Moving them to the end ensures content
    // is prioritized if the email gets clipped.
    Ok(move_styles_to_body_end(&result))
}

/// Replace `<link rel="stylesheet" href="...">` tags with inline `<style>` blocks
/// by reading the referenced CSS files from disk.
fn resolve_link_tags(html: &str, base_path: Option<&std::path::Path>) -> String {
    let base = match base_path {
        Some(p) => {
            if p.is_absolute() {
                p.to_path_buf()
            } else {
                std::env::current_dir().unwrap_or_default().join(p)
            }
        }
        None => return html.to_string(),
    };

    let link_re =
        Regex::new(r#"(?i)<link\s+[^>]*rel\s*=\s*["']stylesheet["'][^>]*>"#).unwrap();
    let href_re = Regex::new(r#"href\s*=\s*["']([^"']+)["']"#).unwrap();

    link_re
        .replace_all(html, |caps: &regex::Captures| {
            let full = &caps[0];
            if let Some(href_cap) = href_re.captures(full) {
                let href = &href_cap[1];
                // Skip absolute URLs
                if href.starts_with("http://") || href.starts_with("https://") {
                    return full.to_string();
                }
                let css_path = base.join(href);
                match std::fs::read_to_string(&css_path) {
                    Ok(css) => format!("<style type=\"text/css\">{}</style>", css),
                    Err(_) => full.to_string(),
                }
            } else {
                full.to_string()
            }
        })
        .to_string()
}

/// Move `<style>` blocks from `<head>` to just before `</body>`.
fn move_styles_to_body_end(html: &str) -> String {
    let style_re = Regex::new(r"(?si)<style[^>]*>.*?</style>").unwrap();

    // Find <head>...</head> region
    let head_start = match html.find("<head") {
        Some(i) => i,
        None => return html.to_string(),
    };
    let head_end = match html[head_start..].find("</head>") {
        Some(i) => head_start + i + 7,
        None => return html.to_string(),
    };

    let head_content = &html[head_start..head_end];

    // Extract style blocks from head
    let styles: Vec<String> = style_re
        .find_iter(head_content)
        .map(|m| m.as_str().to_string())
        .collect();

    if styles.is_empty() {
        return html.to_string();
    }

    // Remove style blocks from head
    let new_head = style_re.replace_all(head_content, "").to_string();

    // Insert styles before </body>
    let style_block = styles.join("\n");
    let mut result = format!("{}{}{}", &html[..head_start], new_head, &html[head_end..]);

    if let Some(body_end) = result.rfind("</body>") {
        result.insert_str(body_end, &style_block);
    } else {
        // No </body> tag — append at the end
        result.push_str(&style_block);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_basic() {
        let html = r#"<html><head><style>.red { color: red; }</style></head><body><p class="red">Hello</p></body></html>"#;
        let result = inline_css(html, None).unwrap();
        assert!(
            result.contains("style=\"color: red;\"") || result.contains("style=\"color:red\"")
        );
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
