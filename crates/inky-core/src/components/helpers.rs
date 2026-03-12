use scraper::ElementRef;

use crate::attrs::{get_classes};

/// Get inner HTML of an element.
pub fn inner_html(element: &ElementRef) -> String {
    element.inner_html()
}

/// Build a class string from a base class and element classes.
pub fn build_classes(base: &str, element: &ElementRef) -> String {
    let mut classes = vec![base.to_string()];
    classes.extend(get_classes(element));
    classes.join(" ")
}
