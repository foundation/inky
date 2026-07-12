use super::El;
use crate::attrs::has_class;
use crate::config::{Config, OutputMode};

/// Transform a column with explicit position info.
pub fn transform_column_with_position(
    el: &El,
    config: &Config,
    col_count: u32,
    is_first: bool,
    is_last: bool,
) -> String {
    let attrs = el.attrs();
    let inner = el.inner().to_string();
    let mut classes = el.classes();

    let small_size = el
        .attr("sm")
        .or_else(|| el.attr("small"))
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(config.column_count);

    let large_size = el
        .attr("lg")
        .or_else(|| el.attr("large"))
        .and_then(|s| s.parse::<u32>().ok())
        .or_else(|| {
            el.attr("sm")
                .or_else(|| el.attr("small"))
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

    let attrs_str = if attrs.is_empty() { String::new() } else { attrs };

    match config.output_mode {
        OutputMode::Table => {
            let no_expander = el.attr("no-expander");
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

/// Single column entry point: detects sibling columns from the DOM.
pub fn make_column(el: &El, config: &Config) -> String {
    let col_count = count_sibling_columns(el, config) + 1;
    let is_first = !has_prev_sibling_column(el, config);
    let is_last = !has_next_sibling_column(el, config);
    transform_column_with_position(el, config, col_count, is_first, is_last)
}

/// Check if an element is a column.
pub fn is_column_element(el: &scraper::ElementRef, config: &Config) -> bool {
    let name = el.value().name();
    name == config.components.columns
        || name == "columns"
        || (name == "th" && has_class(el, "columns"))
}

fn count_sibling_columns(el: &El, config: &Config) -> u32 {
    let mut count = 0;
    let mut node = el.element.prev_sibling();
    while let Some(sibling) = node {
        if let Some(sib_el) = scraper::ElementRef::wrap(sibling) {
            if is_column_element(&sib_el, config) {
                count += 1;
            }
        }
        node = sibling.prev_sibling();
    }
    let mut node = el.element.next_sibling();
    while let Some(sibling) = node {
        if let Some(sib_el) = scraper::ElementRef::wrap(sibling) {
            if is_column_element(&sib_el, config) {
                count += 1;
            }
        }
        node = sibling.next_sibling();
    }
    count
}

fn has_prev_sibling_column(el: &El, config: &Config) -> bool {
    let mut node = el.element.prev_sibling();
    while let Some(sibling) = node {
        if let Some(sib_el) = scraper::ElementRef::wrap(sibling) {
            if is_column_element(&sib_el, config) {
                return true;
            }
        }
        node = sibling.prev_sibling();
    }
    false
}

fn has_next_sibling_column(el: &El, config: &Config) -> bool {
    let mut node = el.element.next_sibling();
    while let Some(sibling) = node {
        if let Some(sib_el) = scraper::ElementRef::wrap(sibling) {
            if is_column_element(&sib_el, config) {
                return true;
            }
        }
        node = sibling.next_sibling();
    }
    false
}
