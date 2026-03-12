use scraper::ElementRef;

use crate::attrs::{get_attr, get_classes};
use super::helpers::inner_html;

pub fn make_block_grid(element: &ElementRef) -> String {
    let up = get_attr(element, "up").unwrap_or_default();
    let mut classes = vec!["block-grid".to_string(), format!("up-{}", up)];
    classes.extend(get_classes(element));
    let inner = inner_html(element);
    format!(
        r#"<table role="presentation" class="{}"><tbody><tr>{}</tr></tbody></table>"#,
        classes.join(" "),
        inner
    )
}
