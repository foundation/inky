use super::helpers::build_classes;
use super::El;

/// <h-line> (v1 compat)
pub fn make_h_line(el: &El) -> String {
    let classes = build_classes("h-line", el);
    format!(
        r#"<table role="presentation" class="{}" aria-hidden="true"><tbody><tr><th>&nbsp;</th></tr></tbody></table>"#,
        classes
    )
}

/// <divider> (v2)
pub fn make_divider(el: &El) -> String {
    let classes = build_classes("divider", el);
    format!(
        r#"<table role="presentation" class="{}" aria-hidden="true"><tbody><tr><th>&nbsp;</th></tr></tbody></table>"#,
        classes
    )
}
