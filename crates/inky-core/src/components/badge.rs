use super::El;

/// `<badge color="#e74c3c">New</badge>`
///
/// Renders a small inline badge/pill label, useful for status indicators or tags.
pub fn make_badge(el: &El) -> String {
    let attrs = el.attrs();
    let color = el.attr("color").unwrap_or_else(|| "#333333".to_string());
    let text_color = el
        .attr("text-color")
        .unwrap_or_else(|| "#ffffff".to_string());
    let inner = el.inner();

    let mut classes = vec!["badge".to_string()];
    classes.extend(el.classes());
    let class_str = classes.join(" ");

    format!(
        r#"<span{} class="{}" style="display: inline-block; padding: 2px 8px; background-color: {}; color: {}; border-radius: 12px; font-size: 12px; font-weight: bold; line-height: 1.4;">{}</span>"#,
        attrs, class_str, color, text_color, inner
    )
}
