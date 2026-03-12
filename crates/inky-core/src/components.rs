use scraper::ElementRef;

use crate::attrs::{get_attr, get_attrs, get_classes, has_class};
use crate::config::Config;

/// Transform a single component element into email-safe HTML.
pub fn transform_component(element: &ElementRef, config: &Config) -> Option<String> {
    let tag = element.value().name();
    let comps = &config.components;

    if tag == comps.h_line {
        Some(make_h_line(element))
    } else if tag == comps.columns {
        Some(make_column(element, config))
    } else if tag == comps.row {
        Some(make_row(element))
    } else if tag == comps.button {
        Some(make_button(element))
    } else if tag == comps.container {
        Some(make_container(element))
    } else if tag == comps.inky {
        Some(make_inky())
    } else if tag == comps.block_grid {
        Some(make_block_grid(element))
    } else if tag == comps.menu {
        Some(make_menu(element))
    } else if tag == comps.menu_item {
        Some(make_menu_item(element))
    } else if tag == comps.center {
        Some(make_center(element))
    } else if tag == comps.callout {
        Some(make_callout(element))
    } else if tag == comps.spacer {
        Some(make_spacer(element))
    } else if tag == comps.wrapper {
        Some(make_wrapper(element))
    } else {
        None
    }
}

/// Get inner HTML of an element.
fn inner_html(element: &ElementRef) -> String {
    element.inner_html()
}

/// Build a class string from a base class and element classes.
fn build_classes(base: &str, element: &ElementRef) -> String {
    let mut classes = vec![base.to_string()];
    classes.extend(get_classes(element));
    classes.join(" ")
}

// <h-line>
fn make_h_line(element: &ElementRef) -> String {
    let classes = build_classes("h-line", element);
    format!(
        r#"<table role="presentation" class="{}"><tbody><tr><th>&nbsp;</th></tr></tbody></table>"#,
        classes
    )
}

// <row>
fn make_row(element: &ElementRef) -> String {
    let attrs = get_attrs(element);
    let classes = build_classes("row", element);
    let inner = inner_html(element);
    format!(
        r#"<table role="presentation"{} class="{}"><tbody><tr>{}</tr></tbody></table>"#,
        attrs, classes, inner
    )
}

// <button>
fn make_button(element: &ElementRef) -> String {
    let attrs = get_attrs(element);
    let mut inner = inner_html(element);

    // Prepare optional target attribute
    let target = match get_attr(element, "target") {
        Some(t) => format!(" target={}", t),
        None => String::new(),
    };

    // Wrap content in <a> if href is present and non-empty
    if let Some(href) = get_attr(element, "href") {
        if !href.is_empty() {
            inner = format!(
                r#"<a{} href="{}"{}>{}</a>"#,
                attrs, href, target, inner
            );
        }
    }

    // Expanded buttons get a <center> tag and expander
    let expander;
    if has_class(element, "expand") || has_class(element, "expanded") {
        inner = format!("<center>{}</center>", inner);
        expander = "\n<td class=\"expander\"></td>"; // Matches JS output format
    } else {
        expander = "";
    }

    let classes = build_classes("button", element);
    format!(
        r#"<table role="presentation" class="{}"><tbody><tr><td><table role="presentation"><tbody><tr><td>{}</td></tr></tbody></table></td>{}</tr></tbody></table>"#,
        classes, inner, expander
    )
}

// <container>
fn make_container(element: &ElementRef) -> String {
    let attrs = get_attrs(element);
    let classes = build_classes("container", element);
    let inner = inner_html(element);
    // Only add align="center" if not already present from passthrough attrs
    let align = if attrs.contains("align=") { "" } else { " align=\"center\"" };
    format!(
        r#"<table role="presentation"{}{} class="{}"><tbody><tr><td>{}</td></tr></tbody></table>"#,
        attrs, align, classes, inner
    )
}

// <inky> easter egg
fn make_inky() -> String {
    r#"<tr><td><img src="https://raw.githubusercontent.com/arvida/emoji-cheat-sheet.com/master/public/graphics/emojis/octopus.png" /></tr></td>"#.to_string()
}

// <block-grid>
fn make_block_grid(element: &ElementRef) -> String {
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

// <menu>
fn make_menu(element: &ElementRef) -> String {
    let attrs = get_attrs(element);
    let classes = build_classes("menu", element);
    let inner = inner_html(element);
    format!(
        r#"<table role="presentation"{} class="{}"><tbody><tr><td><table role="presentation"><tbody><tr>{}</tr></tbody></table></td></tr></tbody></table>"#,
        attrs, classes, inner
    )
}

// <item>
fn make_menu_item(element: &ElementRef) -> String {
    let attrs = get_attrs(element);
    let href = get_attr(element, "href").unwrap_or_default();
    let target = match get_attr(element, "target") {
        Some(t) => format!(" target={}", t),
        None => String::new(),
    };
    let classes = build_classes("menu-item", element);
    let inner = inner_html(element);
    format!(
        r#"<th{} class="{}"><a href="{}"{}>{}</a></th>"#,
        attrs, classes, href, target, inner
    )
}

// <center>
fn make_center(element: &ElementRef) -> String {
    // We need to modify children in-place: add align="center" and class="float-center"
    // Since scraper doesn't support mutation, we rebuild the HTML with modifications
    let mut html = String::new();
    html.push_str("<center data-parsed=\"\">");

    for child in element.children() {
        if let Some(child_el) = ElementRef::wrap(child) {
            let tag_name = child_el.value().name().to_string();
            let mut attrs: Vec<(String, String)> = child_el
                .value()
                .attrs()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();

            // Add align="center"
            let has_align = attrs.iter().any(|(k, _)| k == "align");
            if !has_align {
                attrs.push(("align".into(), "center".into()));
            }

            // Add float-center to class
            let class_idx = attrs.iter().position(|(k, _)| k == "class");
            match class_idx {
                Some(idx) => {
                    let existing = attrs[idx].1.clone();
                    if !existing.split_whitespace().any(|c| c == "float-center") {
                        attrs[idx].1 = format!("{} float-center", existing);
                    }
                }
                None => {
                    attrs.push(("class".into(), "float-center".into()));
                }
            }

            // Rebuild the element
            html.push('<');
            html.push_str(&tag_name);
            for (key, value) in &attrs {
                html.push(' ');
                html.push_str(key);
                html.push_str("=\"");
                html.push_str(value);
                html.push('"');
            }
            html.push('>');

            html.push_str(&child_el.inner_html());

            html.push_str(&format!("</{}>", tag_name));
        } else if let Some(text) = child.value().as_text() {
            html.push_str(&text.text);
        }
    }

    html.push_str("</center>");
    html
}


// <callout>
fn make_callout(element: &ElementRef) -> String {
    let attrs = get_attrs(element);
    let classes = build_classes("callout-inner", element);
    let inner = inner_html(element);
    format!(
        r#"<table role="presentation"{} class="callout"><tbody><tr><th class="{}">{}</th><th class="expander"></th></tr></tbody></table>"#,
        attrs, classes, inner
    )
}

// <spacer>
fn make_spacer(element: &ElementRef) -> String {
    let attrs = get_attrs(element);
    let element_classes = get_classes(element);
    let mut classes = vec!["spacer".to_string()];
    classes.extend(element_classes);
    let class_str = classes.join(" ");

    let size_sm = get_attr(element, "size-sm");
    let size_lg = get_attr(element, "size-lg");

    if size_sm.is_some() || size_lg.is_some() {
        let mut html = String::new();
        if let Some(size) = &size_sm {
            html.push_str(&format!(
                r#"<table role="presentation"{} class="{} hide-for-large"><tbody><tr><td height="{}" style="font-size:{}px;line-height:{}px;">&nbsp;</td></tr></tbody></table>"#,
                attrs, class_str, size, size, size
            ));
        }
        if let Some(size) = &size_lg {
            html.push_str(&format!(
                r#"<table role="presentation"{} class="{} show-for-large"><tbody><tr><td height="{}" style="font-size:{}px;line-height:{}px;">&nbsp;</td></tr></tbody></table>"#,
                attrs, class_str, size, size, size
            ));
        }
        html
    } else {
        let size = get_attr(element, "size").unwrap_or_else(|| "16".to_string());
        format!(
            r#"<table role="presentation"{} class="{}"><tbody><tr><td height="{}" style="font-size:{}px;line-height:{}px;">&nbsp;</td></tr></tbody></table>"#,
            attrs, class_str, size, size, size
        )
    }
}

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

    let small_size = get_attr(element, "small")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(config.column_count);

    let large_size = get_attr(element, "large")
        .and_then(|s| s.parse::<u32>().ok())
        .or_else(|| get_attr(element, "small").and_then(|s| s.parse::<u32>().ok()))
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

    let no_expander = get_attr(element, "no-expander");
    let has_nested_row = inner.contains("class=\"row") || inner.contains("<row");
    let needs_expander = large_size == config.column_count
        && !has_nested_row
        && (no_expander.is_none() || no_expander.as_deref() == Some("false"));

    let expander = if needs_expander {
        "\n<th class=\"expander\"></th>"
    } else {
        ""
    };

    let attrs_str = if attrs.is_empty() {
        String::new()
    } else {
        attrs
    };

    format!(
        r#"<th class="{}"{}><table role="presentation"><tbody><tr><th>{}</th>{}</tr></tbody></table></th>"#,
        classes.join(" "),
        attrs_str,
        inner,
        expander
    )
}

// <columns> (single column, used as fallback)
fn make_column(element: &ElementRef, config: &Config) -> String {
    let attrs = get_attrs(element);
    let inner = inner_html(element);
    let mut classes = get_classes(element);

    // Count sibling columns (add 1 for current)
    let col_count = count_sibling_columns(element, config) + 1;

    // Determine small and large sizes
    let small_size = get_attr(element, "small")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(config.column_count);

    let large_size = get_attr(element, "large")
        .and_then(|s| s.parse::<u32>().ok())
        .or_else(|| get_attr(element, "small").and_then(|s| s.parse::<u32>().ok()))
        .unwrap_or(config.column_count / col_count);

    classes.push(format!("small-{}", small_size));
    classes.push(format!("large-{}", large_size));
    classes.push("columns".to_string());

    // Determine first/last
    if !has_prev_sibling_column(element, config) {
        classes.push("first".to_string());
    }
    if !has_next_sibling_column(element, config) {
        classes.push("last".to_string());
    }

    // Determine if expander is needed
    let no_expander = get_attr(element, "no-expander");
    let has_nested_row = inner.contains("class=\"row") || inner.contains("<row");
    // no-expander can be: absent (None), bare attribute (""), "true", or "false"
    // Only add expander if no-expander is absent or explicitly "false"
    let needs_expander = large_size == config.column_count
        && !has_nested_row
        && (no_expander.is_none() || no_expander.as_deref() == Some("false"));

    let expander = if needs_expander {
        "\n<th class=\"expander\"></th>"
    } else {
        ""
    };

    // Only include attrs if non-empty
    let attrs_str = if attrs.is_empty() {
        String::new()
    } else {
        attrs
    };

    format!(
        r#"<th class="{}"{}><table role="presentation"><tbody><tr><th>{}</th>{}</tr></tbody></table></th>"#,
        classes.join(" "),
        attrs_str,
        inner,
        expander
    )
}

/// Check if an element is a column — either a <columns> tag or a transformed <th> with "columns" class.
fn is_column_element(el: &ElementRef, config: &Config) -> bool {
    el.value().name() == config.components.columns
        || (el.value().name() == "th" && has_class(el, "columns"))
}

/// Count sibling elements that are columns (including already-transformed ones).
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

/// Check if there's a previous sibling that is a column.
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

/// Check if there's a next sibling that is a column.
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

// <wrapper>
fn make_wrapper(element: &ElementRef) -> String {
    let attrs = get_attrs(element);
    let classes = build_classes("wrapper", element);
    let inner = inner_html(element);
    format!(
        r#"<table role="presentation"{} class="{}" align="center"><tbody><tr><td class="wrapper-inner">{}</td></tr></tbody></table>"#,
        attrs, classes, inner
    )
}
