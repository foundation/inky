use scraper::ElementRef;

use crate::attrs::{get_attr, get_attrs, get_classes};

pub fn make_spacer(element: &ElementRef) -> String {
    let attrs = get_attrs(element);
    let element_classes = get_classes(element);
    let mut classes = vec!["spacer".to_string()];
    classes.extend(element_classes);
    let class_str = classes.join(" ");

    // Accept both v2 (height, sm, lg) and v1 (size, size-sm, size-lg) attribute names
    let size_sm = get_attr(element, "sm").or_else(|| get_attr(element, "size-sm"));
    let size_lg = get_attr(element, "lg").or_else(|| get_attr(element, "size-lg"));

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
        let size = get_attr(element, "height")
            .or_else(|| get_attr(element, "size"))
            .unwrap_or_else(|| "16".to_string());
        format!(
            r#"<table role="presentation"{} class="{}" aria-hidden="true"><tbody><tr><td height="{}" style="font-size:{}px;line-height:{}px;">&nbsp;</td></tr></tbody></table>"#,
            attrs, class_str, size, size, size
        )
    }
}
