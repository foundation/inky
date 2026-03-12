use scraper::ElementRef;

use crate::attrs::get_attrs;
use super::helpers::{inner_html, build_classes};

pub fn make_callout(element: &ElementRef) -> String {
    let attrs = get_attrs(element);
    let classes = build_classes("callout-inner", element);
    let inner = inner_html(element);
    format!(
        r#"<table role="presentation"{} class="callout"><tbody><tr><th class="{}">{}</th><th class="expander"></th></tr></tbody></table>"#,
        attrs, classes, inner
    )
}
