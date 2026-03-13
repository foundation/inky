use scraper::ElementRef;

use super::helpers::{build_classes, inner_html};
use crate::attrs::get_attrs;

pub fn make_wrapper(element: &ElementRef) -> String {
    let attrs = get_attrs(element);
    let classes = build_classes("wrapper", element);
    let inner = inner_html(element);
    format!(
        r#"<table role="presentation"{} class="{}" align="center"><tbody><tr><td class="wrapper-inner">{}</td></tr></tbody></table>"#,
        attrs, classes, inner
    )
}
