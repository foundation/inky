use scraper::ElementRef;

use crate::attrs::{get_attr, get_attrs, get_classes};
use super::helpers::inner_html;

/// `<card image="photo.jpg" title="Card Title" href="...">Card body text</card>`
///
/// Renders a card component with optional image, title, and body content.
pub fn make_card(element: &ElementRef) -> String {
    let attrs = get_attrs(element);
    let image = get_attr(element, "image");
    let title = get_attr(element, "title");
    let href = get_attr(element, "href");
    let inner = inner_html(element);

    let mut classes = vec!["card".to_string()];
    classes.extend(get_classes(element));
    let class_str = classes.join(" ");

    let mut content = String::new();

    // Optional image row
    if let Some(img_url) = image {
        let img_html = format!(
            r#"<img src="{}" alt="" width="100%" style="display: block; border: 0;">"#,
            img_url
        );
        let wrapped = if let Some(ref url) = href {
            format!(r#"<a href="{}" target="_blank">{}</a>"#, url, img_html)
        } else {
            img_html
        };
        content.push_str(&format!(
            r#"<tr><td class="card-image">{}</td></tr>"#,
            wrapped
        ));
    }

    // Optional title row
    if let Some(title_text) = title {
        content.push_str(&format!(
            r#"<tr><td class="card-title" style="padding: 16px 16px 0 16px; font-weight: bold; font-size: 18px;">{}</td></tr>"#,
            title_text
        ));
    }

    // Body content row
    if !inner.trim().is_empty() {
        content.push_str(&format!(
            r#"<tr><td class="card-body" style="padding: 16px;">{}</td></tr>"#,
            inner
        ));
    }

    format!(
        r#"<table role="presentation"{} class="{}" width="100%" cellpadding="0" cellspacing="0" style="border: 1px solid #e0e0e0; border-radius: 4px; overflow: hidden;"><tbody>{}</tbody></table>"#,
        attrs, class_str, content
    )
}
