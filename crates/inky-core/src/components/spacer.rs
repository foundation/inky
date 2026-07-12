use super::El;

pub fn make_spacer(el: &El) -> String {
    let attrs = el.attrs();
    let element_classes = el.classes();
    let mut classes = vec!["spacer".to_string()];
    classes.extend(element_classes);
    let class_str = classes.join(" ");

    // Accept both v2 (height, sm, lg) and v1 (size, size-sm, size-lg) attribute names
    let size_sm = el.attr("sm").or_else(|| el.attr("size-sm"));
    let size_lg = el.attr("lg").or_else(|| el.attr("size-lg"));

    if size_sm.is_some() || size_lg.is_some() {
        let mut html = String::new();
        if let Some(size) = &size_sm {
            html.push_str(&format!(
                r#"<table role="presentation"{} class="{} hide-for-large" aria-hidden="true"><tbody><tr><td height="{}" style="font-size:{}px;line-height:{}px;">&nbsp;</td></tr></tbody></table>"#,
                attrs, class_str, size, size, size
            ));
        }
        if let Some(size) = &size_lg {
            html.push_str(&format!(
                r#"<table role="presentation"{} class="{} show-for-large" aria-hidden="true"><tbody><tr><td height="{}" style="font-size:{}px;line-height:{}px;">&nbsp;</td></tr></tbody></table>"#,
                attrs, class_str, size, size, size
            ));
        }
        html
    } else {
        // Accept both v2 (height) and v1 (size) attribute names
        let size = el
            .attr("height")
            .or_else(|| el.attr("size"))
            .unwrap_or_else(|| "16".to_string());
        format!(
            r#"<table role="presentation"{} class="{}" aria-hidden="true"><tbody><tr><td height="{}" style="font-size:{}px;line-height:{}px;">&nbsp;</td></tr></tbody></table>"#,
            attrs, class_str, size, size, size
        )
    }
}
