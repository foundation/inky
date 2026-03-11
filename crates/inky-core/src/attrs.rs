use scraper::ElementRef;

/// Attributes that should not be passed through to the output HTML.
const IGNORED_ATTRIBUTES: &[&str] = &[
    "class", "id", "href", "size", "size-sm", "size-lg", "large", "no-expander", "small", "target",
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
        Some(class_str) => class_str.split_whitespace().map(|s| s.to_string()).collect(),
        None => vec![],
    }
}

/// Check if an element has a specific class.
pub fn has_class(element: &ElementRef, class: &str) -> bool {
    get_classes(element).iter().any(|c| c == class)
}
