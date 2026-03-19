use scraper::ElementRef;

use super::helpers::inner_html;
use crate::attrs::{get_attr, get_attrs, get_classes, has_class};
use crate::config::Config;

pub fn make_button(element: &ElementRef, config: &Config) -> String {
    let attrs = get_attrs(element);
    let inner_text = inner_html(element);
    let href = get_attr(element, "href").unwrap_or_default();

    let target = match get_attr(element, "target") {
        Some(t) => format!(" target={}", t),
        None => String::new(),
    };

    // Build classes: base "button" + element classes + v2 size/color attributes
    let mut classes = vec!["button".to_string()];
    classes.extend(get_classes(element));
    if let Some(size) = get_attr(element, "size") {
        classes.push(size);
    }
    if let Some(color) = get_attr(element, "color") {
        classes.push(color);
    }
    let class_str = classes.join(" ");

    // Check if this button should use bulletproof VML
    let bulletproof = has_attr(element, "bulletproof") || config.bulletproof_buttons;

    if bulletproof && !href.is_empty() {
        make_bulletproof_button(element, &href, &target, &inner_text, &attrs, &class_str)
    } else {
        make_table_button(&href, &target, &inner_text, &attrs, &class_str, element)
    }
}

/// Standard table-based button output.
fn make_table_button(
    href: &str,
    target: &str,
    inner_text: &str,
    attrs: &str,
    class_str: &str,
    element: &ElementRef,
) -> String {
    let mut inner = inner_text.to_string();

    if !href.is_empty() {
        inner = format!(r#"<a{} href="{}"{}>{}</a>"#, attrs, href, target, inner);
    }

    let expander;
    if has_class(element, "expand") || has_class(element, "expanded") {
        inner = format!("<center>{}</center>", inner);
        expander = "\n<td class=\"expander\" aria-hidden=\"true\"></td>";
    } else {
        expander = "";
    }

    format!(
        r#"<table role="presentation" class="{}"><tbody><tr><td><table role="presentation"><tbody><tr><td>{}</td></tr></tbody></table></td>{}</tr></tbody></table>"#,
        class_str, inner, expander
    )
}

/// Bulletproof VML button: MSO conditional with v:roundrect for Outlook,
/// standard table button for everything else.
fn make_bulletproof_button(
    element: &ElementRef,
    href: &str,
    target: &str,
    inner_text: &str,
    attrs: &str,
    class_str: &str,
) -> String {
    // Read optional VML-specific attributes with sensible defaults
    let width = get_attr(element, "width").unwrap_or_else(|| "200".to_string());
    let height = get_attr(element, "height").unwrap_or_else(|| "40".to_string());
    let radius = get_attr(element, "radius").unwrap_or_else(|| "3".to_string());
    let bg_color = get_attr(element, "bg-color").unwrap_or_else(|| "#1a73b5".to_string());
    let text_color = get_attr(element, "text-color").unwrap_or_else(|| "#ffffff".to_string());

    // Convert radius px to arcsize percentage (arcsize = radius / (height/2) * 100)
    let arcsize = radius
        .parse::<f64>()
        .ok()
        .and_then(|r| height.parse::<f64>().ok().map(|h| (r / (h / 2.0)) * 100.0))
        .map(|a| format!("{:.0}%", a))
        .unwrap_or_else(|| "10%".to_string());

    // Build the standard table button for non-MSO clients
    let table_button = make_table_button(href, target, inner_text, attrs, class_str, element);

    format!(
        r#"<!--[if mso]><v:roundrect xmlns:v="urn:schemas-microsoft-com:vml" xmlns:w="urn:schemas-microsoft-com:office:word" href="{href}" style="height:{height}px;v-text-anchor:middle;width:{width}px;" arcsize="{arcsize}" strokecolor="{bg_color}" fillcolor="{bg_color}"><w:anchorlock/><center style="color:{text_color};font-family:sans-serif;font-size:16px;font-weight:bold;">{text}</center></v:roundrect><![endif]--><!--[if !mso]><!-->{table_button}<!--<![endif]-->"#,
        href = href,
        height = height,
        width = width,
        arcsize = arcsize,
        bg_color = bg_color,
        text_color = text_color,
        text = inner_text,
        table_button = table_button,
    )
}

fn has_attr(element: &ElementRef, name: &str) -> bool {
    element.value().attr(name).is_some()
}
