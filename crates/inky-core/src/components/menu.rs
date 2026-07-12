use super::helpers::build_classes;
use super::{El, RenderCtx};

pub fn make_menu(el: &El) -> String {
    let attrs = el.attrs();
    let inner = el.inner();

    // Build classes: base "menu" + element classes + v2 direction attribute
    let mut classes = vec!["menu".to_string()];
    classes.extend(el.classes());
    if let Some(direction) = el.attr("direction") {
        classes.push(direction);
    }
    let class_str = classes.join(" ");

    format!(
        r#"<table role="presentation"{} class="{}"><tbody><tr><td><table role="presentation"><tbody><tr>{}</tr></tbody></table></td></tr></tbody></table>"#,
        attrs, class_str, inner
    )
}

pub fn make_menu_item(el: &El, ctx: &RenderCtx) -> String {
    let _ = ctx; // used in Task 4 (float-center inside <center>)
    let attrs = el.attrs();
    let href = el.attr("href").unwrap_or_default();
    let target = match el.attr("target") {
        Some(t) => format!(" target={}", t),
        None => String::new(),
    };
    let classes = build_classes("menu-item", el);
    let inner = el.inner();
    format!(
        r#"<th{} class="{}"><a href="{}"{}>{}</a></th>"#,
        attrs, classes, href, target, inner
    )
}
