use super::El;

pub fn make_callout(el: &El) -> String {
    let attrs = el.attrs();
    let inner = el.inner();

    // Build inner classes: base "callout-inner" + element classes + v2 color attribute
    let mut classes = vec!["callout-inner".to_string()];
    classes.extend(el.classes());
    if let Some(color) = el.attr("color") {
        classes.push(color);
    }
    let class_str = classes.join(" ");

    format!(
        r#"<table role="presentation"{} class="callout"><tbody><tr><th class="{}">{}</th><th class="expander" aria-hidden="true"></th></tr></tbody></table>"#,
        attrs, class_str, inner
    )
}
