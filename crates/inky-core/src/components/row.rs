use scraper::ElementRef;

use super::helpers::{build_classes, inner_html};
use crate::attrs::get_attrs;

pub fn make_row(element: &ElementRef) -> String {
    let attrs = get_attrs(element);
    let classes = build_classes("row", element);
    let inner = inner_html(element);
    format!(
        r#"<table role="presentation"{} class="{}"><tbody><tr>{}</tr></tbody></table>"#,
        attrs, classes, inner
    )
}
