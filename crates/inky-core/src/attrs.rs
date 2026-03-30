use scraper::ElementRef;

/// Attributes that should not be passed through to the output HTML.
const IGNORED_ATTRIBUTES: &[&str] = &[
    "class",
    "id",
    "href",
    "size",
    "size-sm",
    "size-lg",
    "large",
    "no-expander",
    "small",
    "target",
    // v2 attribute names
    "sm",
    "lg",
    "height",
    "up",
    "retina",
    "color",
    "direction",
    // new component attributes
    "align",
    "background",
    "platform",
    "icon",
    "poster",
    "src",
    "alt",
    "width",
    "image",
    "title",
    "cite",
    "text-color",
    "type",
    // bulletproof button attributes
    "bulletproof",
    "bg-color",
    "radius",
];

/// Extract passthrough attributes from an element, excluding ignored ones.
/// Returns a string like ` foo="bar" baz="qux"`.
pub fn get_attrs(element: &ElementRef) -> String {
    let mut result = String::new();
    for (key, value) in element.value().attrs() {
        if !IGNORED_ATTRIBUTES.contains(&key) {
            result.push(' ');
            result.push_str(key);
            result.push_str("=\"");
            result.push_str(value);
            result.push('"');
        }
    }
    result
}

/// Get the value of a specific attribute from an element.
pub fn get_attr(element: &ElementRef, name: &str) -> Option<String> {
    element.value().attr(name).map(|s| s.to_string())
}

/// Get class list from an element.
pub fn get_classes(element: &ElementRef) -> Vec<String> {
    match element.value().attr("class") {
        Some(class_str) => class_str
            .split_whitespace()
            .map(|s| s.to_string())
            .collect(),
        None => vec![],
    }
}

/// Check if an element has a specific class.
pub fn has_class(element: &ElementRef, class: &str) -> bool {
    get_classes(element).iter().any(|c| c == class)
}

/// Extract a CSS property value from an inline style string.
/// e.g. `extract_css_property("color: red; font-size: 14px", "color")` => `Some("red")`
pub fn extract_css_property(style: &str, property: &str) -> Option<String> {
    for decl in style.split(';') {
        let decl = decl.trim();
        if let Some(colon_pos) = decl.find(':') {
            let prop = decl[..colon_pos].trim();
            if prop.eq_ignore_ascii_case(property) {
                let value = decl[colon_pos + 1..].trim();
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}
