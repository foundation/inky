use scraper::ElementRef;

use super::helpers::build_classes;

/// <h-line> (v1 compat)
pub fn make_h_line(element: &ElementRef) -> String {
    let classes = build_classes("h-line", element);
    format!(
        r#"<table role="presentation" class="{}"><tbody><tr><th>&nbsp;</th></tr></tbody></table>"#,
        classes
    )
}

/// <divider> (v2)
pub fn make_divider(element: &ElementRef) -> String {
    let classes = build_classes("divider", element);
    format!(
        r#"<table role="presentation" class="{}"><tbody><tr><th>&nbsp;</th></tr></tbody></table>"#,
        classes
    )
}
