use scraper::ElementRef;

use super::helpers::{build_classes, inner_html};
use crate::attrs::get_attrs;

pub fn make_container(element: &ElementRef) -> String {
    let attrs = get_attrs(element);
    let classes = build_classes("container", element);
    let inner = inner_html(element);
    let align = if attrs.contains("align=") {
        ""
    } else {
        " align=\"center\""
    };
    format!(
        r#"<table role="presentation"{}{} class="{}"><tbody><tr><td>{}</td></tr></tbody></table>"#,
        attrs, align, classes, inner
    )
}
