use scraper::ElementRef;

use super::helpers::inner_html;
use crate::attrs::{get_attr, get_attrs, get_classes, has_class};
use crate::config::{Config, OutputMode};

/// Transform a column with explicit position info (used for batch column processing).
pub fn transform_column_with_position(
    element: &ElementRef,
    config: &Config,
    col_count: u32,
    is_first: bool,
    is_last: bool,
) -> String {
    let attrs = get_attrs(element);
    let inner = inner_html(element);
    let mut classes = get_classes(element);

    let small_size = get_attr(element, "sm")
        .or_else(|| get_attr(element, "small"))
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(config.column_count);

    let large_size = get_attr(element, "lg")
        .or_else(|| get_attr(element, "large"))
        .and_then(|s| s.parse::<u32>().ok())
        .or_else(|| {
            get_attr(element, "sm")
                .or_else(|| get_attr(element, "small"))
                .and_then(|s| s.parse::<u32>().ok())
        })
        .unwrap_or(config.column_count / col_count);

    classes.push(format!("small-{}", small_size));
    classes.push(format!("large-{}", large_size));
    classes.push("columns".to_string());

    if is_first {
        classes.push("first".to_string());
    }
    if is_last {
        classes.push("last".to_string());
    }

    let attrs_str = if attrs.is_empty() {
        String::new()
    } else {
        attrs
    };

    match config.output_mode {
        OutputMode::Table => {
            let no_expander = get_attr(element, "no-expander");
            let has_nested_row = inner.contains("class=\"row") || inner.contains("<row");
            let needs_expander = large_size == config.column_count
                && !has_nested_row
                && (no_expander.is_none() || no_expander.as_deref() == Some("false"));

            let expander = if needs_expander {
                "\n<th class=\"expander\" aria-hidden=\"true\"></th>"
            } else {
                ""
            };

            format!(
                r#"<th class="{}"{}><table role="presentation"><tbody><tr><th>{}</th>{}</tr></tbody></table></th>"#,
                classes.join(" "),
                attrs_str,
                inner,
                expander
            )
        }
        OutputMode::Hybrid => {
            let width_pct = (large_size as f64 / config.column_count as f64) * 100.0;
            let width_pct_str = format!("{:.4}", width_pct)
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string();
            let mso_width_pct = width_pct_str.clone();

            format!(
                r#"<!--[if mso]><td width="{mso_width_pct}%" valign="top"><![endif]--><div class="{classes}"{attrs} style="display:inline-block;width:100%;max-width:{width_pct}%;vertical-align:top;">{inner}</div><!--[if mso]></td><![endif]-->"#,
                mso_width_pct = mso_width_pct,
                classes = classes.join(" "),
                attrs = attrs_str,
                width_pct = width_pct_str,
                inner = inner,
            )
        }
    }
}

/// Single column fallback (used when not batch-processed via row).
pub fn make_column(element: &ElementRef, config: &Config) -> String {
    let attrs = get_attrs(element);
    let inner = inner_html(element);
    let mut classes = get_classes(element);

    let col_count = count_sibling_columns(element, config) + 1;

    let small_size = get_attr(element, "sm")
        .or_else(|| get_attr(element, "small"))
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(config.column_count);

    let large_size = get_attr(element, "lg")
        .or_else(|| get_attr(element, "large"))
        .and_then(|s| s.parse::<u32>().ok())
        .or_else(|| {
            get_attr(element, "sm")
                .or_else(|| get_attr(element, "small"))
                .and_then(|s| s.parse::<u32>().ok())
        })
        .unwrap_or(config.column_count / col_count);

    classes.push(format!("small-{}", small_size));
    classes.push(format!("large-{}", large_size));
    classes.push("columns".to_string());

    if !has_prev_sibling_column(element, config) {
        classes.push("first".to_string());
    }
    if !has_next_sibling_column(element, config) {
        classes.push("last".to_string());
    }

    let attrs_str = if attrs.is_empty() {
        String::new()
    } else {
        attrs
    };

    match config.output_mode {
        OutputMode::Table => {
            let no_expander = get_attr(element, "no-expander");
            let has_nested_row = inner.contains("class=\"row") || inner.contains("<row");
            let needs_expander = large_size == config.column_count
                && !has_nested_row
                && (no_expander.is_none() || no_expander.as_deref() == Some("false"));

            let expander = if needs_expander {
                "\n<th class=\"expander\" aria-hidden=\"true\"></th>"
            } else {
                ""
            };

            format!(
                r#"<th class="{}"{}><table role="presentation"><tbody><tr><th>{}</th>{}</tr></tbody></table></th>"#,
                classes.join(" "),
                attrs_str,
                inner,
                expander
            )
        }
        OutputMode::Hybrid => {
            let width_pct = (large_size as f64 / config.column_count as f64) * 100.0;
            let width_pct_str = format!("{:.4}", width_pct)
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string();

            format!(
                r#"<!--[if mso]><td width="{}%" valign="top"><![endif]--><div class="{}"{} style="display:inline-block;width:100%;max-width:{}%;vertical-align:top;">{}</div><!--[if mso]></td><![endif]-->"#,
                width_pct_str,
                classes.join(" "),
                attrs_str,
                width_pct_str,
                inner,
            )
        }
    }
}

/// Check if an element is a column.
pub fn is_column_element(el: &ElementRef, config: &Config) -> bool {
    let name = el.value().name();
    name == config.components.columns
        || name == "columns"
        || (name == "th" && has_class(el, "columns"))
}

fn count_sibling_columns(element: &ElementRef, config: &Config) -> u32 {
    let mut count = 0;
    let mut node = element.prev_sibling();
    while let Some(sibling) = node {
        if let Some(el) = scraper::ElementRef::wrap(sibling) {
            if is_column_element(&el, config) {
                count += 1;
            }
        }
        node = sibling.prev_sibling();
    }
    let mut node = element.next_sibling();
    while let Some(sibling) = node {
        if let Some(el) = scraper::ElementRef::wrap(sibling) {
            if is_column_element(&el, config) {
                count += 1;
            }
        }
        node = sibling.next_sibling();
    }
    count
}

fn has_prev_sibling_column(element: &ElementRef, config: &Config) -> bool {
    let mut node = element.prev_sibling();
    while let Some(sibling) = node {
        if let Some(el) = scraper::ElementRef::wrap(sibling) {
            if is_column_element(&el, config) {
                return true;
            }
        }
        node = sibling.prev_sibling();
    }
    false
}

fn has_next_sibling_column(element: &ElementRef, config: &Config) -> bool {
    let mut node = element.next_sibling();
    while let Some(sibling) = node {
        if let Some(el) = scraper::ElementRef::wrap(sibling) {
            if is_column_element(&el, config) {
                return true;
            }
        }
        node = sibling.next_sibling();
    }
    false
}
