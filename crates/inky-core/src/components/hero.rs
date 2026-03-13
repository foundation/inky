use scraper::ElementRef;

use super::helpers::inner_html;
use crate::attrs::{get_attr, get_classes};

/// `<hero background="hero.jpg" width="600" height="400">Content</hero>`
///
/// Full-width background image section with overlaid content.
/// Uses VML for Outlook support, CSS background-image for modern clients.
pub fn make_hero(element: &ElementRef) -> String {
    let background = get_attr(element, "background").unwrap_or_default();
    let width = get_attr(element, "width").unwrap_or_else(|| "600".to_string());
    let height = get_attr(element, "height").unwrap_or_else(|| "400".to_string());
    let inner = inner_html(element);

    let mut classes = vec!["hero".to_string()];
    classes.extend(get_classes(element));
    let class_str = classes.join(" ");

    let mut html = String::new();

    // Outlook VML background
    html.push_str(&format!(
        "<!--[if mso]>\n<v:rect xmlns:v=\"urn:schemas-microsoft-com:vml\" fill=\"true\" stroke=\"false\" style=\"width:{}px;height:{}px;\">\n<v:fill type=\"frame\" src=\"{}\" />\n<v:textbox style=\"mso-fit-shape-to-text:true\" inset=\"0,0,0,0\">\n<![endif]-->\n",
        width, height, background
    ));

    // Modern clients: CSS background-image
    html.push_str(&format!(
        r#"<table role="presentation" class="{}" width="{}" cellpadding="0" cellspacing="0" style="background-image: url('{}'); background-size: cover; background-position: center center; background-repeat: no-repeat; width: {}px;"><tbody><tr><td style="padding: 0;">{}</td></tr></tbody></table>"#,
        class_str, width, background, width, inner
    ));

    // Close Outlook VML
    html.push_str("\n<!--[if mso]>\n</v:textbox>\n</v:rect>\n<![endif]-->");

    html
}
