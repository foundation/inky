use scraper::ElementRef;

use super::helpers::inner_html;
use crate::attrs::{get_attr, get_attrs, get_classes};

/// `<social align="center"><social-link platform="facebook" href="...">Facebook</social-link></social>`
///
/// Renders a horizontal row of social media icon links.
pub fn make_social(element: &ElementRef) -> String {
    let attrs = get_attrs(element);
    let inner = inner_html(element);

    let mut classes = vec!["social".to_string()];
    classes.extend(get_classes(element));
    let class_str = classes.join(" ");

    let align = get_attr(element, "align").unwrap_or_else(|| "center".to_string());

    format!(
        r#"<table role="presentation"{} class="{}" align="{}"><tbody><tr><td><table role="presentation" align="{}"><tbody><tr>{}</tr></tbody></table></td></tr></tbody></table>"#,
        attrs, class_str, align, align, inner
    )
}

/// Known social platforms and their default icon colors.
const SOCIAL_PLATFORMS: &[(&str, &str)] = &[
    ("facebook", "#3b5998"),
    ("twitter", "#1da1f2"),
    ("x", "#000000"),
    ("instagram", "#e1306c"),
    ("linkedin", "#0077b5"),
    ("youtube", "#ff0000"),
    ("github", "#333333"),
    ("tiktok", "#000000"),
    ("pinterest", "#bd081c"),
    ("snapchat", "#fffc00"),
    ("threads", "#000000"),
    ("mastodon", "#6364ff"),
    ("bluesky", "#0085ff"),
    ("discord", "#5865f2"),
    ("whatsapp", "#25d366"),
    ("telegram", "#0088cc"),
    ("reddit", "#ff4500"),
    ("dribbble", "#ea4c89"),
    ("behance", "#1769ff"),
];

/// `<social-link platform="facebook" href="..." icon="custom.png">Facebook</social-link>`
///
/// Renders a single social media icon link within a `<social>` row.
pub fn make_social_link(element: &ElementRef) -> String {
    let href = get_attr(element, "href").unwrap_or_else(|| "#".to_string());
    let platform = get_attr(element, "platform").unwrap_or_default();
    let icon = get_attr(element, "icon");
    let inner = inner_html(element);
    let color = get_attr(element, "color");

    let bg_color = color.unwrap_or_else(|| {
        SOCIAL_PLATFORMS
            .iter()
            .find(|(name, _)| *name == platform.as_str())
            .map(|(_, c)| c.to_string())
            .unwrap_or_else(|| "#333333".to_string())
    });

    let icon_html = if let Some(icon_url) = icon {
        format!(
            r#"<img src="{}" alt="{}" width="24" height="24" style="display: block; border: 0;">"#,
            icon_url, platform
        )
    } else {
        // Text-only fallback when no icon URL is provided
        String::new()
    };

    let label = if inner.trim().is_empty() {
        // Capitalize platform name as fallback
        let mut chars = platform.chars();
        match chars.next() {
            None => String::new(),
            Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        }
    } else {
        inner
    };

    format!(
        r#"<td class="social-link {}" style="padding: 0 4px;"><a href="{}" target="_blank" style="color: {}; text-decoration: none;">{}{}</a></td>"#,
        platform, href, bg_color, icon_html, label
    )
}
