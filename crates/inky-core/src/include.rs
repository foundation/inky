use regex::Regex;
use std::path::{Path, PathBuf};

/// Trait for resolving include paths to their content.
pub trait IncludeResolver {
    fn resolve(&self, path: &str) -> Result<String, String>;
}

/// Resolves includes from the filesystem, relative to a base path.
pub struct FileIncludeResolver {
    base_path: PathBuf,
}

impl FileIncludeResolver {
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }
}

impl IncludeResolver for FileIncludeResolver {
    fn resolve(&self, path: &str) -> Result<String, String> {
        let full_path = self.base_path.join(path);
        std::fs::read_to_string(&full_path).map_err(|e| {
            format!(
                "Failed to include '{}' (resolved to '{}'): {}",
                path,
                full_path.display(),
                e
            )
        })
    }
}

const MAX_INCLUDE_DEPTH: usize = 10;

/// Parse attributes from a tag string like `src="foo" title="bar"`.
/// Returns a Vec of (name, value) pairs.
fn parse_attributes(attrs_str: &str) -> Vec<(String, String)> {
    let attr_re = Regex::new(r#"(\w[\w-]*)\s*=\s*"([^"]*)""#).unwrap();
    attr_re
        .captures_iter(attrs_str)
        .map(|cap| (cap[1].to_string(), cap[2].to_string()))
        .collect()
}

/// Replace `$name$` and `$name|default$` variable placeholders in content with provided values.
/// If a variable is not provided and has a default (e.g. `$title|Untitled$`), the default is used.
/// If a variable is not provided and has no default (e.g. `$title$`), it is left as-is.
fn replace_variables(content: &str, vars: &[(String, String)]) -> String {
    let mut result = content.to_string();

    // First, replace $name|default$ patterns where a value was provided
    // Then replace simple $name$ patterns where a value was provided
    for (name, value) in vars {
        let default_re = Regex::new(&format!(r"\${}(?:\|[^$]*)?\$", regex::escape(name))).unwrap();
        result = default_re.replace_all(&result, value.as_str()).to_string();
    }

    // Replace any remaining $name|default$ with their default values
    let default_re = Regex::new(r"\$(\w[\w-]*)\|([^$]*)\$").unwrap();
    result = default_re.replace_all(&result, "$2").to_string();

    result
}

/// Process a layout declaration and inject content into the layout.
///
/// If the template starts with `<layout src="...">`, the layout file is loaded and
/// the template content replaces the `<yield>` tag in the layout. The `<layout>` tag
/// is stripped from the content before injection.
///
/// Additional attributes on the `<layout>` tag (besides `src`) are passed as variables
/// and replace `$name$` placeholders in the layout file.
///
/// If no `<layout>` tag is found, the content is returned as-is.
pub fn process_layout(html: &str, base_path: &Path) -> Result<String, String> {
    let layout_re = Regex::new(r#"(?s)<layout\s+((?:[^>]*?))\s*>(.*)"#).unwrap();

    if let Some(caps) = layout_re.captures(html) {
        let attrs_str = &caps[1];
        let content = caps[2].trim();

        let attrs = parse_attributes(attrs_str);
        let layout_src = attrs
            .iter()
            .find(|(name, _)| name == "src")
            .map(|(_, v)| v.clone())
            .ok_or_else(|| "Layout tag is missing src attribute".to_string())?;

        // Collect variables (all attributes except src)
        let vars: Vec<(String, String)> = attrs
            .into_iter()
            .filter(|(name, _)| name != "src")
            .collect();

        let layout_path = base_path.join(&layout_src);
        let layout_html = std::fs::read_to_string(&layout_path).map_err(|e| {
            format!(
                "Failed to load layout '{}' (resolved to '{}'): {}",
                layout_src,
                layout_path.display(),
                e
            )
        })?;

        // Replace $name$ variables in the layout
        let layout_html = replace_variables(&layout_html, &vars);

        // Replace <yield>, <yield/>, or <yield /> in the layout with the content
        let yield_re = Regex::new(r"<yield\s*/?\s*>").unwrap();
        if !yield_re.is_match(&layout_html) {
            return Err(format!(
                "Layout '{}' does not contain a <yield> tag",
                layout_src
            ));
        }

        Ok(yield_re.replace(&layout_html, content).to_string())
    } else {
        Ok(html.to_string())
    }
}

/// Process all `<include src="...">` and `<include src="..." />` tags in the HTML,
/// replacing them with the content of the referenced files.
///
/// Includes are resolved recursively (included files may themselves contain includes),
/// up to a maximum depth to prevent infinite loops.
pub fn process_includes(html: &str, base_path: &Path) -> Result<String, String> {
    let resolver = FileIncludeResolver::new(base_path);
    process_includes_recursive(html, &resolver, 0)
}

/// Process includes with a custom resolver (useful for testing).
pub fn process_includes_with_resolver(
    html: &str,
    resolver: &dyn IncludeResolver,
) -> Result<String, String> {
    process_includes_recursive(html, resolver, 0)
}

fn process_includes_recursive(
    html: &str,
    resolver: &dyn IncludeResolver,
    depth: usize,
) -> Result<String, String> {
    if depth >= MAX_INCLUDE_DEPTH {
        return Err(format!(
            "Maximum include depth ({}) exceeded — check for circular includes",
            MAX_INCLUDE_DEPTH
        ));
    }

    // Match <include ...> and <include ... /> with any attributes
    let re = Regex::new(r#"<include\s+((?:[^>]*?))\s*/?\s*>"#).unwrap();

    if !re.is_match(html) {
        return Ok(html.to_string());
    }

    let mut result = html.to_string();

    // Process one include at a time to handle nested includes correctly
    while let Some(caps) = re.captures(&result) {
        let full_match = caps.get(0).unwrap();
        let attrs_str = &caps[1];

        let attrs = parse_attributes(attrs_str);
        let src = attrs
            .iter()
            .find(|(name, _)| name == "src")
            .map(|(_, v)| v.clone())
            .ok_or_else(|| "Include tag is missing src attribute".to_string())?;

        let vars: Vec<(String, String)> = attrs
            .into_iter()
            .filter(|(name, _)| name != "src")
            .collect();

        let content = resolver.resolve(&src)?;
        let content = replace_variables(&content, &vars);

        // For nested includes, resolve relative to the included file's directory
        let included_path = Path::new(&src);
        let nested_content = if let Some(parent) = included_path.parent() {
            if parent.as_os_str().is_empty() {
                process_includes_recursive(&content, resolver, depth + 1)?
            } else {
                // Create a resolver for the included file's directory
                // We need to figure out the full base path
                // Since the resolver resolves relative to its base, we adjust
                let nested_resolver = NestedResolver {
                    parent: resolver,
                    prefix: parent.to_string_lossy().to_string(),
                };
                process_includes_recursive(&content, &nested_resolver, depth + 1)?
            }
        } else {
            process_includes_recursive(&content, resolver, depth + 1)?
        };

        let start = full_match.start();
        let end = full_match.end();
        result = format!("{}{}{}", &result[..start], nested_content, &result[end..]);
    }

    Ok(result)
}

/// A resolver that prepends a prefix path for resolving nested includes.
struct NestedResolver<'a> {
    parent: &'a dyn IncludeResolver,
    prefix: String,
}

impl<'a> IncludeResolver for NestedResolver<'a> {
    fn resolve(&self, path: &str) -> Result<String, String> {
        let prefixed = format!("{}/{}", self.prefix, path);
        self.parent.resolve(&prefixed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Test resolver that uses an in-memory map.
    struct MapResolver {
        files: HashMap<String, String>,
    }

    impl IncludeResolver for MapResolver {
        fn resolve(&self, path: &str) -> Result<String, String> {
            self.files
                .get(path)
                .cloned()
                .ok_or_else(|| format!("File not found: {}", path))
        }
    }

    #[test]
    fn test_no_includes() {
        let resolver = MapResolver {
            files: HashMap::new(),
        };
        let html = "<p>Hello world</p>";
        let result = process_includes_with_resolver(html, &resolver).unwrap();
        assert_eq!(result, html);
    }

    #[test]
    fn test_simple_include() {
        let mut files = HashMap::new();
        files.insert("header.inky".to_string(), "<h1>Header</h1>".to_string());
        let resolver = MapResolver { files };

        let html = r#"<include src="header.inky">
<p>Body</p>"#;
        let result = process_includes_with_resolver(html, &resolver).unwrap();
        assert_eq!(result, "<h1>Header</h1>\n<p>Body</p>");
    }

    #[test]
    fn test_self_closing_include() {
        let mut files = HashMap::new();
        files.insert("footer.inky".to_string(), "<p>Footer</p>".to_string());
        let resolver = MapResolver { files };

        let html = r#"<p>Body</p>
<include src="footer.inky" />"#;
        let result = process_includes_with_resolver(html, &resolver).unwrap();
        assert_eq!(result, "<p>Body</p>\n<p>Footer</p>");
    }

    #[test]
    fn test_multiple_includes() {
        let mut files = HashMap::new();
        files.insert("header.inky".to_string(), "<h1>Header</h1>".to_string());
        files.insert("footer.inky".to_string(), "<p>Footer</p>".to_string());
        let resolver = MapResolver { files };

        let html = r#"<include src="header.inky">
<p>Body</p>
<include src="footer.inky">"#;
        let result = process_includes_with_resolver(html, &resolver).unwrap();
        assert_eq!(result, "<h1>Header</h1>\n<p>Body</p>\n<p>Footer</p>");
    }

    #[test]
    fn test_nested_includes() {
        let mut files = HashMap::new();
        files.insert(
            "wrapper.inky".to_string(),
            r#"<div><include src="inner.inky"></div>"#.to_string(),
        );
        files.insert("inner.inky".to_string(), "<span>nested</span>".to_string());
        let resolver = MapResolver { files };

        let html = r#"<include src="wrapper.inky">"#;
        let result = process_includes_with_resolver(html, &resolver).unwrap();
        assert_eq!(result, "<div><span>nested</span></div>");
    }

    #[test]
    fn test_missing_include_errors() {
        let resolver = MapResolver {
            files: HashMap::new(),
        };
        let html = r#"<include src="nonexistent.inky">"#;
        let result = process_includes_with_resolver(html, &resolver);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("nonexistent.inky"));
    }

    #[test]
    fn test_layout_no_declaration() {
        let html = "<p>No layout here</p>";
        let result = process_layout(html, Path::new(".")).unwrap();
        assert_eq!(result, html);
    }

    #[test]
    fn test_max_depth_protection() {
        let mut files = HashMap::new();
        // Create a circular include
        files.insert(
            "a.inky".to_string(),
            r#"<include src="b.inky">"#.to_string(),
        );
        files.insert(
            "b.inky".to_string(),
            r#"<include src="a.inky">"#.to_string(),
        );
        let resolver = MapResolver { files };

        let html = r#"<include src="a.inky">"#;
        let result = process_includes_with_resolver(html, &resolver);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Maximum include depth"));
    }

    #[test]
    fn test_include_with_variables() {
        let mut files = HashMap::new();
        files.insert(
            "greeting.inky".to_string(),
            "<h1>Hello, $name$!</h1><p>Your role: $role$</p>".to_string(),
        );
        let resolver = MapResolver { files };

        let html = r#"<include src="greeting.inky" name="Alice" role="admin">"#;
        let result = process_includes_with_resolver(html, &resolver).unwrap();
        assert_eq!(result, "<h1>Hello, Alice!</h1><p>Your role: admin</p>");
    }

    #[test]
    fn test_include_variables_unreplaced_stay() {
        let mut files = HashMap::new();
        files.insert(
            "greeting.inky".to_string(),
            "<h1>$name$</h1><p>$missing$</p>".to_string(),
        );
        let resolver = MapResolver { files };

        let html = r#"<include src="greeting.inky" name="Bob">"#;
        let result = process_includes_with_resolver(html, &resolver).unwrap();
        assert_eq!(result, "<h1>Bob</h1><p>$missing$</p>");
    }

    #[test]
    fn test_include_no_variables() {
        let mut files = HashMap::new();
        files.insert(
            "plain.inky".to_string(),
            "<p>No variables here</p>".to_string(),
        );
        let resolver = MapResolver { files };

        let html = r#"<include src="plain.inky">"#;
        let result = process_includes_with_resolver(html, &resolver).unwrap();
        assert_eq!(result, "<p>No variables here</p>");
    }

    #[test]
    fn test_variable_default_used_when_not_provided() {
        let mut files = HashMap::new();
        files.insert(
            "page.inky".to_string(),
            "<title>$title|Untitled$</title>".to_string(),
        );
        let resolver = MapResolver { files };

        let html = r#"<include src="page.inky">"#;
        let result = process_includes_with_resolver(html, &resolver).unwrap();
        assert_eq!(result, "<title>Untitled</title>");
    }

    #[test]
    fn test_variable_default_overridden_when_provided() {
        let mut files = HashMap::new();
        files.insert(
            "page.inky".to_string(),
            "<title>$title|Untitled$</title>".to_string(),
        );
        let resolver = MapResolver { files };

        let html = r#"<include src="page.inky" title="Welcome!">"#;
        let result = process_includes_with_resolver(html, &resolver).unwrap();
        assert_eq!(result, "<title>Welcome!</title>");
    }

    #[test]
    fn test_variable_empty_default() {
        let mut files = HashMap::new();
        files.insert(
            "page.inky".to_string(),
            "<span>$preheader|$</span>".to_string(),
        );
        let resolver = MapResolver { files };

        let html = r#"<include src="page.inky">"#;
        let result = process_includes_with_resolver(html, &resolver).unwrap();
        assert_eq!(result, "<span></span>");
    }

    #[test]
    fn test_variable_empty_default_overridden() {
        let mut files = HashMap::new();
        files.insert(
            "page.inky".to_string(),
            "<span>$preheader|$</span>".to_string(),
        );
        let resolver = MapResolver { files };

        let html = r#"<include src="page.inky" preheader="Preview text">"#;
        let result = process_includes_with_resolver(html, &resolver).unwrap();
        assert_eq!(result, "<span>Preview text</span>");
    }

    #[test]
    fn test_mixed_defaults_and_no_defaults() {
        let mut files = HashMap::new();
        files.insert(
            "page.inky".to_string(),
            "<title>$title|Default Title$</title><h1>$heading$</h1><p>$footer|© 2026$</p>"
                .to_string(),
        );
        let resolver = MapResolver { files };

        let html = r#"<include src="page.inky" heading="Hello">"#;
        let result = process_includes_with_resolver(html, &resolver).unwrap();
        assert_eq!(
            result,
            "<title>Default Title</title><h1>Hello</h1><p>© 2026</p>"
        );
    }
}
