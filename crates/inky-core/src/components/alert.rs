use scraper::ElementRef;

use super::helpers::inner_html;
use crate::attrs::{get_attr, get_attrs, get_classes};

/// Known alert types and their default background colors.
const ALERT_TYPES: &[(&str, &str)] = &[
    ("success", "#dff0d8"),
    ("warning", "#fcf8e3"),
    ("error", "#f2dede"),
    ("info", "#d9edf7"),
];

/// `<alert type="success">Operation completed!</alert>`
///
/// Renders a styled alert/notification banner. Supports types:
/// success, warning, error, info. Custom colors via `color` attribute.
pub fn make_alert(element: &ElementRef) -> String {
    let attrs = get_attrs(element);
    let alert_type = get_attr(element, "type").unwrap_or_else(|| "info".to_string());
    let color = get_attr(element, "color");
    let inner = inner_html(element);

    let mut classes = vec!["alert".to_string(), format!("alert-{}", alert_type)];
    classes.extend(get_classes(element));
    let class_str = classes.join(" ");

    let bg_color = color.unwrap_or_else(|| {
        ALERT_TYPES
            .iter()
            .find(|(name, _)| *name == alert_type.as_str())
            .map(|(_, c)| c.to_string())
            .unwrap_or_else(|| "#d9edf7".to_string())
    });

    format!(
        r#"<table role="presentation"{} class="{}" width="100%" cellpadding="0" cellspacing="0"><tbody><tr><td style="padding: 12px 16px; background-color: {}; border-radius: 4px;">{}</td></tr></tbody></table>"#,
        attrs, class_str, bg_color, inner
    )
}
