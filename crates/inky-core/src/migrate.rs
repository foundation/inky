use regex::Regex;

/// Migrate v1 Inky syntax to v2 syntax.
///
/// This is a text-level conversion — it does NOT produce table output.
/// It converts old tag names, attribute names, and class-based patterns
/// to the modern v2 equivalents.
pub fn migrate(html: &str) -> MigrateResult {
    let mut result = html.to_string();
    let mut changes = Vec::new();

    // 1. <columns> → <column> (plural → singular)
    result = rename_tag(&result, "columns", "column", &mut changes);

    // 2. <h-line> → <divider>
    result = rename_tag(&result, "h-line", "divider", &mut changes);

    // 3. Attribute renames on <column>: large→lg, small→sm
    result = rename_attr_on_tag(&result, "column", "large", "lg", &mut changes);
    result = rename_attr_on_tag(&result, "column", "small", "sm", &mut changes);

    // 4. <spacer size="N"> → <spacer height="N">
    result = rename_attr_on_tag(&result, "spacer", "size", "height", &mut changes);

    // 5. <spacer size-sm="N" size-lg="N"> → <spacer sm="N" lg="N">
    result = rename_attr_on_tag(&result, "spacer", "size-sm", "sm", &mut changes);
    result = rename_attr_on_tag(&result, "spacer", "size-lg", "lg", &mut changes);

    // 6. <button class="small alert expand"> → <button size="small" color="alert" expand>
    result = migrate_button_classes(&result, &mut changes);

    // 7. <callout class="primary"> → <callout color="primary">
    result = migrate_callout_classes(&result, &mut changes);

    // 8. <menu class="vertical"> → <menu direction="vertical">
    result = migrate_menu_classes(&result, &mut changes);

    // 9. <center><menu ...> → <menu align="center" ...> and remove wrapping <center>
    result = migrate_centered_menu(&result, &mut changes);

    MigrateResult {
        html: result,
        changes,
    }
}

/// Result of a migration, including the transformed HTML and a list of changes made.
#[derive(Debug)]
pub struct MigrateResult {
    pub html: String,
    pub changes: Vec<MigrateChange>,
}

/// A single migration change for reporting.
#[derive(Debug, Clone)]
pub struct MigrateChange {
    pub description: String,
}

/// Rename a tag (both opening and closing).
fn rename_tag(html: &str, from: &str, to: &str, changes: &mut Vec<MigrateChange>) -> String {
    let from_escaped = regex::escape(from);
    // Opening tag: <from ...> or <from>
    let open_re = Regex::new(&format!(r"<{f}(\s|>|/>)", f = from_escaped)).unwrap();
    // Closing tag: </from>
    let close_re = Regex::new(&format!(r"</{f}\s*>", f = from_escaped)).unwrap();

    let mut result = html.to_string();
    if open_re.is_match(&result) {
        changes.push(MigrateChange {
            description: format!("<{}> → <{}>", from, to),
        });
        result = open_re
            .replace_all(&result, |caps: &regex::Captures| {
                format!("<{}{}", to, &caps[1])
            })
            .to_string();
        result = close_re
            .replace_all(&result, format!("</{}>", to))
            .to_string();
    }
    result
}

/// Rename an attribute on a specific tag.
fn rename_attr_on_tag(
    html: &str,
    tag: &str,
    from_attr: &str,
    to_attr: &str,
    changes: &mut Vec<MigrateChange>,
) -> String {
    let tag_escaped = regex::escape(tag);
    let from_escaped = regex::escape(from_attr);

    // Match the full opening tag for this element
    let tag_re = Regex::new(&format!(r"<{t}(\s[^>]*)?>", t = tag_escaped)).unwrap();

    // Match the attribute within the tag
    let attr_re = Regex::new(&format!(r#"\b{a}\s*="#, a = from_escaped)).unwrap();

    let mut made_change = false;

    let result = tag_re
        .replace_all(html, |caps: &regex::Captures| {
            let full = caps[0].to_string();
            if attr_re.is_match(&full) {
                made_change = true;
                attr_re.replace(&full, format!("{}=", to_attr)).to_string()
            } else {
                full
            }
        })
        .to_string();

    if made_change {
        changes.push(MigrateChange {
            description: format!("<{}> attribute {} → {}", tag, from_attr, to_attr),
        });
    }

    result
}

/// Button size class names that become size="..." attributes.
const BUTTON_SIZES: &[&str] = &["tiny", "small", "large"];
/// Button color class names that become color="..." attributes.
const BUTTON_COLORS: &[&str] = &["primary", "secondary", "success", "alert", "warning"];
/// Button boolean class names that become bare attributes.
const BUTTON_BOOLEANS: &[&str] = &["expand", "expanded", "radius", "rounded", "hollow"];

/// Migrate button classes to attributes.
fn migrate_button_classes(html: &str, changes: &mut Vec<MigrateChange>) -> String {
    let re = Regex::new(r#"(?i)(<button\s)([^>]*?)class\s*=\s*"([^"]*)"([^>]*>)"#).unwrap();

    if !re.is_match(html) {
        return html.to_string();
    }

    let mut made_changes = false;

    let result = re
        .replace_all(html, |caps: &regex::Captures| {
            let prefix = &caps[1]; // "<button "
            let before = &caps[2]; // attrs before class
            let class_val = &caps[3]; // class value
            let after = &caps[4]; // attrs after class + >

            let classes: Vec<&str> = class_val.split_whitespace().collect();

            let mut size_attr = String::new();
            let mut color_attr = String::new();
            let mut bool_attrs = Vec::new();
            let mut remaining_classes = Vec::new();

            for cls in &classes {
                let cls_lower = cls.to_lowercase();
                if BUTTON_SIZES.contains(&cls_lower.as_str()) {
                    size_attr = format!(r#"size="{}""#, cls_lower);
                    made_changes = true;
                } else if BUTTON_COLORS.contains(&cls_lower.as_str()) {
                    color_attr = format!(r#"color="{}""#, cls_lower);
                    made_changes = true;
                } else if BUTTON_BOOLEANS.contains(&cls_lower.as_str()) {
                    bool_attrs.push(cls_lower);
                    made_changes = true;
                } else {
                    remaining_classes.push(*cls);
                }
            }

            // Rebuild the tag
            let mut parts = Vec::new();
            parts.push(prefix.to_string());
            if !before.trim().is_empty() {
                parts.push(before.trim().to_string());
                parts.push(" ".to_string());
            }
            if !remaining_classes.is_empty() {
                parts.push(format!(r#"class="{}""#, remaining_classes.join(" ")));
                parts.push(" ".to_string());
            }
            if !size_attr.is_empty() {
                parts.push(size_attr);
                parts.push(" ".to_string());
            }
            if !color_attr.is_empty() {
                parts.push(color_attr);
                parts.push(" ".to_string());
            }
            for b in &bool_attrs {
                parts.push(b.clone());
                parts.push(" ".to_string());
            }

            // Append remaining attrs from after class (everything before the closing >)
            let after_trimmed = after.trim();
            if after_trimmed != ">" && after_trimmed != "/>" {
                // Strip the closing > or /> to get remaining attrs
                let remaining = after_trimmed
                    .trim_end_matches('>')
                    .trim_end_matches('/')
                    .trim();
                if !remaining.is_empty() {
                    parts.push(remaining.to_string());
                    parts.push(" ".to_string());
                }
            }

            // Build result, trimming trailing space before >
            let mut tag = parts.join("");
            tag = tag.trim_end().to_string();
            // Re-add the closing >
            let closing = if after.contains("/>") { "/>" } else { ">" };
            format!("{}{}", tag, closing)
        })
        .to_string();

    if made_changes {
        changes.push(MigrateChange {
            description: "<button> class → attributes (size, color, expand, etc.)".to_string(),
        });
    }

    result
}

/// Migrate callout classes to color attribute.
fn migrate_callout_classes(html: &str, changes: &mut Vec<MigrateChange>) -> String {
    let re = Regex::new(r#"(?i)(<callout\s)([^>]*?)class\s*=\s*"([^"]*)"([^>]*>)"#).unwrap();

    if !re.is_match(html) {
        return html.to_string();
    }

    let colors = ["primary", "secondary", "success", "alert", "warning"];
    let mut made_changes = false;

    let result = re
        .replace_all(html, |caps: &regex::Captures| {
            let prefix = &caps[1];
            let before = &caps[2];
            let class_val = &caps[3];
            let after = &caps[4];

            let classes: Vec<&str> = class_val.split_whitespace().collect();
            let mut color = String::new();
            let mut remaining = Vec::new();

            for cls in &classes {
                if colors.contains(&cls.to_lowercase().as_str()) {
                    color = cls.to_lowercase();
                    made_changes = true;
                } else {
                    remaining.push(*cls);
                }
            }

            let mut tag = prefix.to_string();
            if !before.trim().is_empty() {
                tag.push_str(before.trim());
                tag.push(' ');
            }
            if !remaining.is_empty() {
                tag.push_str(&format!(r#"class="{}""#, remaining.join(" ")));
                tag.push(' ');
            }
            if !color.is_empty() {
                tag.push_str(&format!(r#"color="{}""#, color));
            }
            tag = tag.trim_end().to_string();
            let closing = after.trim_start_matches(|c: char| c != '>' && c != '/');
            format!("{}{}", tag, closing)
        })
        .to_string();

    if made_changes {
        changes.push(MigrateChange {
            description: r#"<callout class="..."> → <callout color="...">"#.to_string(),
        });
    }

    result
}

/// Migrate menu class="vertical" to direction="vertical".
fn migrate_menu_classes(html: &str, changes: &mut Vec<MigrateChange>) -> String {
    let re = Regex::new(r#"(?i)(<menu\s)([^>]*?)class\s*=\s*"([^"]*)"([^>]*>)"#).unwrap();

    if !re.is_match(html) {
        return html.to_string();
    }

    let mut made_changes = false;

    let result = re
        .replace_all(html, |caps: &regex::Captures| {
            let prefix = &caps[1];
            let before = &caps[2];
            let class_val = &caps[3];
            let after = &caps[4];

            let classes: Vec<&str> = class_val.split_whitespace().collect();
            let mut direction = String::new();
            let mut remaining = Vec::new();

            for cls in &classes {
                if cls.eq_ignore_ascii_case("vertical") {
                    direction = "vertical".to_string();
                    made_changes = true;
                } else {
                    remaining.push(*cls);
                }
            }

            let mut tag = prefix.to_string();
            if !before.trim().is_empty() {
                tag.push_str(before.trim());
                tag.push(' ');
            }
            if !remaining.is_empty() {
                tag.push_str(&format!(r#"class="{}""#, remaining.join(" ")));
                tag.push(' ');
            }
            if !direction.is_empty() {
                tag.push_str(&format!(r#"direction="{}""#, direction));
            }
            tag = tag.trim_end().to_string();
            let closing = after.trim_start_matches(|c: char| c != '>' && c != '/');
            format!("{}{}", tag, closing)
        })
        .to_string();

    if made_changes {
        changes.push(MigrateChange {
            description: r#"<menu class="vertical"> → <menu direction="vertical">"#.to_string(),
        });
    }

    result
}

/// Migrate <center><menu ...></menu></center> to <menu align="center" ...>.
fn migrate_centered_menu(html: &str, changes: &mut Vec<MigrateChange>) -> String {
    let re = Regex::new(r#"(?si)<center>\s*<menu(\s[^>]*)?(>)(.*?)</menu>\s*</center>"#).unwrap();

    if !re.is_match(html) {
        return html.to_string();
    }

    changes.push(MigrateChange {
        description: r#"<center><menu> → <menu align="center">"#.to_string(),
    });

    re.replace_all(html, |caps: &regex::Captures| {
        let existing_attrs = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let inner = &caps[3];

        format!(r#"<menu{} align="center">{}</menu>"#, existing_attrs, inner)
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_columns_to_column() {
        let input = r#"<columns large="6" small="12">Content</columns>"#;
        let result = migrate(input);
        assert!(result.html.contains("<column"));
        assert!(!result.html.contains("<columns"));
        assert!(result.html.contains("</column>"));
    }

    #[test]
    fn test_large_small_to_lg_sm() {
        let input = r#"<column large="6" small="12">Content</column>"#;
        let result = migrate(input);
        assert!(result.html.contains(r#"lg="6""#));
        assert!(result.html.contains(r#"sm="12""#));
        assert!(!result.html.contains("large="));
        assert!(!result.html.contains("small="));
    }

    #[test]
    fn test_columns_combined() {
        // Test that columns→column AND large→lg happen together
        let input = r#"<columns large="6" small="12">Content</columns>"#;
        let result = migrate(input);
        assert!(result.html.contains("<column"));
        assert!(result.html.contains(r#"lg="6""#));
        assert!(result.html.contains(r#"sm="12""#));
    }

    #[test]
    fn test_h_line_to_divider() {
        let input = "<h-line></h-line>";
        let result = migrate(input);
        assert_eq!(result.html, "<divider></divider>");
    }

    #[test]
    fn test_spacer_size_to_height() {
        let input = r#"<spacer size="16"></spacer>"#;
        let result = migrate(input);
        assert!(result.html.contains(r#"height="16""#));
        assert!(!result.html.contains("size="));
    }

    #[test]
    fn test_spacer_responsive() {
        let input = r#"<spacer size-sm="10" size-lg="20"></spacer>"#;
        let result = migrate(input);
        assert!(result.html.contains(r#"sm="10""#));
        assert!(result.html.contains(r#"lg="20""#));
    }

    #[test]
    fn test_button_classes() {
        let input = r##"<button class="small alert expand" href="#">Click</button>"##;
        let result = migrate(input);
        assert!(result.html.contains(r#"size="small""#));
        assert!(result.html.contains(r#"color="alert""#));
        assert!(result.html.contains("expand"));
        // class attribute should be removed (no remaining classes)
        assert!(!result.html.contains("class="));
    }

    #[test]
    fn test_button_mixed_classes() {
        let input = r##"<button class="small alert custom-btn" href="#">Click</button>"##;
        let result = migrate(input);
        assert!(result.html.contains(r#"size="small""#));
        assert!(result.html.contains(r#"color="alert""#));
        assert!(result.html.contains(r#"class="custom-btn""#));
    }

    #[test]
    fn test_callout_class_to_color() {
        let input = r#"<callout class="primary">Message</callout>"#;
        let result = migrate(input);
        assert!(result.html.contains(r#"color="primary""#));
        assert!(!result.html.contains("class="));
    }

    #[test]
    fn test_menu_vertical() {
        let input = r##"<menu class="vertical"><item href="#">Link</item></menu>"##;
        let result = migrate(input);
        assert!(result.html.contains(r#"direction="vertical""#));
        assert!(!result.html.contains(r#"class="vertical""#));
    }

    #[test]
    fn test_centered_menu() {
        let input = r##"<center><menu><item href="#">Link</item></menu></center>"##;
        let result = migrate(input);
        assert!(result.html.contains(r#"align="center""#));
        assert!(!result.html.contains("<center>"));
        assert!(!result.html.contains("</center>"));
    }

    #[test]
    fn test_no_changes_needed() {
        let input = r#"<column lg="6" sm="12">Content</column>"#;
        let result = migrate(input);
        assert_eq!(result.html, input);
        assert!(result.changes.is_empty());
    }

    #[test]
    fn test_full_migration() {
        let input = r##"<container>
  <row>
    <columns large="6" small="12">
      <button class="small alert" href="#">Click</button>
      <spacer size="16"></spacer>
      <h-line></h-line>
      <callout class="primary">Important</callout>
    </columns>
    <columns large="6" small="12">
      <center><menu class="vertical"><item href="#">Link</item></menu></center>
    </columns>
  </row>
</container>"##;

        let result = migrate(input);

        // All v1 patterns should be gone
        assert!(!result.html.contains("<columns"));
        assert!(!result.html.contains("large="));
        assert!(!result.html.contains("small="));
        assert!(!result.html.contains("<h-line"));
        // spacer size= should be gone, but button size="small" is valid v2
        assert!(!result.html.contains(r#"size="16""#));

        // All v2 patterns should be present
        assert!(result.html.contains("<column"));
        assert!(result.html.contains("lg="));
        assert!(result.html.contains("sm="));
        assert!(result.html.contains("<divider"));
        assert!(result.html.contains(r#"height=""#));
        assert!(result.html.contains(r#"size="small""#));
        assert!(result.html.contains(r#"color="alert""#));
        assert!(result.html.contains(r#"color="primary""#));
        assert!(result.html.contains(r#"direction="vertical""#));
        assert!(result.html.contains(r#"align="center""#));

        // Should have multiple changes
        assert!(result.changes.len() >= 5);
    }
}
