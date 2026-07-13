pub(crate) mod scanner;

use regex::Regex;
use scanner::{scan, Attr, Doc, Token};

/// Migrate v1 Inky syntax to v2 syntax.
///
/// This is a text-level conversion — it does NOT produce table output.
/// It converts old tag names, attribute names, and class-based patterns
/// to the modern v2 equivalents.
pub fn migrate(html: &str) -> MigrateResult {
    let mut doc = scan(html);
    let mut changes = Vec::new();

    // 1. <columns> → <column> (plural → singular)
    rename_tag(&mut doc, "columns", "column", &mut changes);
    // 2. <h-line> → <divider>
    rename_tag(&mut doc, "h-line", "divider", &mut changes);
    // 3. Attribute renames on <column>: large→lg, small→sm
    rename_attr_on_tag(&mut doc, "column", "large", "lg", &mut changes);
    rename_attr_on_tag(&mut doc, "column", "small", "sm", &mut changes);
    // 4./5. <spacer> size→height, size-sm→sm, size-lg→lg
    rename_attr_on_tag(&mut doc, "spacer", "size", "height", &mut changes);
    rename_attr_on_tag(&mut doc, "spacer", "size-sm", "sm", &mut changes);
    rename_attr_on_tag(&mut doc, "spacer", "size-lg", "lg", &mut changes);
    // 6.–8. class → attribute migrations
    migrate_classes(
        &mut doc,
        "button",
        &BUTTON_RULE,
        "<button> class → attributes (size, color, expand, etc.)",
        &mut changes,
    );
    migrate_classes(
        &mut doc,
        "callout",
        &CALLOUT_RULE,
        r#"<callout class="..."> → <callout color="...">"#,
        &mut changes,
    );
    migrate_classes(
        &mut doc,
        "menu",
        &MENU_RULE,
        r#"<menu class="vertical"> → <menu direction="vertical">"#,
        &mut changes,
    );

    let html = doc.emit();

    // 9. <center><menu ...> → <menu align="center" ...> (token-based in the
    // next task; the old regex still runs on the emitted string until then)
    let html = migrate_centered_menu(&html, &mut changes);

    MigrateResult { html, changes }
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

/// Rename a tag (both opening and closing), case-insensitively.
fn rename_tag(doc: &mut Doc, from: &str, to: &str, changes: &mut Vec<MigrateChange>) {
    let mut changed = false;
    for token in &mut doc.tokens {
        match token {
            Token::Open(tag) if tag.name == from => {
                tag.name = to.to_string();
                tag.dirty = true;
                changed = true;
            }
            Token::Close(close) if close.name == from => {
                close.renamed = Some(to.to_string());
                changed = true;
            }
            _ => {}
        }
    }
    if changed {
        changes.push(MigrateChange {
            description: format!("<{}> → <{}>", from, to),
        });
    }
}

/// Rename an attribute on a specific tag. Exact name matching — `data-large`
/// can never match a rule for `large`.
fn rename_attr_on_tag(
    doc: &mut Doc,
    tag_name: &str,
    from_attr: &str,
    to_attr: &str,
    changes: &mut Vec<MigrateChange>,
) {
    let mut changed = false;
    for token in &mut doc.tokens {
        let Token::Open(tag) = token else { continue };
        if tag.name != tag_name {
            continue;
        }
        for attr in &mut tag.attrs {
            if attr.name == from_attr {
                attr.name = to_attr.to_string();
                attr.name_out = to_attr.to_string();
                tag.dirty = true;
                changed = true;
            }
        }
    }
    if changed {
        changes.push(MigrateChange {
            description: format!("<{}> attribute {} → {}", tag_name, from_attr, to_attr),
        });
    }
}

/// A class→attribute migration rule for one tag.
struct ClassRule {
    /// (target attribute, class values that map to it) — emitted in this order.
    valued: &'static [(&'static str, &'static [&'static str])],
    /// Class values that become bare boolean attributes.
    boolean: &'static [&'static str],
}

const BUTTON_RULE: ClassRule = ClassRule {
    valued: &[
        ("size", &["tiny", "small", "large"]),
        ("color", &["primary", "secondary", "success", "alert", "warning"]),
    ],
    boolean: &["expand", "expanded", "radius", "rounded", "hollow"],
};

const CALLOUT_RULE: ClassRule = ClassRule {
    valued: &[("color", &["primary", "secondary", "success", "alert", "warning"])],
    boolean: &[],
};

const MENU_RULE: ClassRule = ClassRule {
    valued: &[("direction", &["vertical"])],
    boolean: &[],
};

/// Convert recognized classes on `tag_name` into attributes, preserving all
/// other attributes and the position of the class attribute.
fn migrate_classes(
    doc: &mut Doc,
    tag_name: &str,
    rule: &ClassRule,
    description: &str,
    changes: &mut Vec<MigrateChange>,
) {
    let mut changed = false;
    for token in &mut doc.tokens {
        let Token::Open(tag) = token else { continue };
        if tag.name != tag_name {
            continue;
        }
        let Some(class_idx) = tag.attrs.iter().position(|a| a.name == "class") else {
            continue;
        };
        let Some(class_value) = tag.attrs[class_idx].value.clone() else {
            continue;
        };

        // One slot per valued rule entry (last matching class wins, as before).
        let mut valued: Vec<Option<String>> = vec![None; rule.valued.len()];
        let mut booleans: Vec<String> = Vec::new();
        let mut remaining: Vec<&str> = Vec::new();

        for class in class_value.split_whitespace() {
            let lower = class.to_lowercase();
            if let Some(idx) = rule
                .valued
                .iter()
                .position(|(_, values)| values.contains(&lower.as_str()))
            {
                valued[idx] = Some(lower);
            } else if rule.boolean.contains(&lower.as_str()) {
                booleans.push(lower);
            } else {
                remaining.push(class);
            }
        }

        if valued.iter().all(Option::is_none) && booleans.is_empty() {
            continue;
        }

        // Rebuild in place: [attrs before class] class(remaining)? valued... booleans [attrs after]
        let mut replacement: Vec<Attr> = Vec::new();
        if !remaining.is_empty() {
            replacement.push(Attr::new_double("class", &remaining.join(" ")));
        }
        for (slot, (attr_name, _)) in valued.iter().zip(rule.valued.iter()) {
            if let Some(value) = slot {
                replacement.push(Attr::new_double(attr_name, value));
            }
        }
        for boolean in &booleans {
            replacement.push(Attr::new_bare(boolean));
        }

        tag.attrs.splice(class_idx..class_idx + 1, replacement);
        tag.dirty = true;
        changed = true;
    }
    if changed {
        changes.push(MigrateChange {
            description: description.to_string(),
        });
    }
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

    // --- Phase 4 regression tests: bugs in the regex-based migrator ---

    #[test]
    fn callout_preserves_attributes_after_class() {
        let input = r#"<callout class="primary" id="promo">M</callout>"#;
        let result = migrate(input);
        assert!(result.html.contains(r#"color="primary""#));
        assert!(result.html.contains(r#"id="promo""#), "attribute dropped: {}", result.html);
    }

    #[test]
    fn menu_attr_value_with_slash_not_corrupted() {
        let input = r#"<menu class="vertical" data-url="a/b">x</menu>"#;
        let result = migrate(input);
        assert!(result.html.contains(r#"direction="vertical""#));
        assert!(result.html.contains(r#"data-url="a/b""#), "value corrupted: {}", result.html);
    }

    #[test]
    fn data_large_attribute_untouched() {
        let input = r#"<column data-large="4" large="6">x</column>"#;
        let result = migrate(input);
        assert!(result.html.contains(r#"data-large="4""#), "data-* corrupted: {}", result.html);
        assert!(result.html.contains(r#"lg="6""#), "real attr unmigrated: {}", result.html);
        assert!(!result.html.contains("data-lg"));
    }

    #[test]
    fn attr_value_containing_attr_syntax_untouched() {
        let input = r#"<column title="large=6" large="4">x</column>"#;
        let result = migrate(input);
        assert!(result.html.contains(r#"title="large=6""#), "value rewritten: {}", result.html);
        assert!(result.html.contains(r#"lg="4""#));
    }

    #[test]
    fn data_class_not_treated_as_class() {
        let input = r##"<button data-class="small" href="#">x</button>"##;
        let result = migrate(input);
        assert_eq!(result.html, input);
        assert!(result.changes.is_empty());
    }

    #[test]
    fn attr_value_with_gt_parses() {
        let input = r#"<column large="6" title="a > b">x</column>"#;
        let result = migrate(input);
        assert!(result.html.contains(r#"lg="6""#));
        assert!(result.html.contains(r#"title="a > b""#));
    }

    #[test]
    fn single_quoted_values_preserved() {
        let input = "<column large='6'>x</column>";
        let result = migrate(input);
        assert!(result.html.contains("lg='6'"), "quote style changed: {}", result.html);
    }

    #[test]
    fn self_closing_spacer_migrates() {
        let input = r#"<spacer size="16"/>"#;
        let result = migrate(input);
        assert!(result.html.contains(r#"height="16""#));
        assert!(result.html.trim_end().ends_with("/>"));
    }

    #[test]
    fn uppercase_v1_tag_migrated() {
        let input = r#"<COLUMNS LARGE="6">x</COLUMNS>"#;
        let result = migrate(input);
        assert!(result.html.contains("<column"), "uppercase tag skipped: {}", result.html);
        assert!(result.html.contains(r#"lg="6""#));
        assert!(result.html.contains("</column>"));
    }

    #[test]
    fn bytes_outside_migrated_tags_preserved() {
        let input = "prefix &amp; entities <b>bold</b>\n\t <spacer size=\"4\"></spacer> suffix &lt;";
        let result = migrate(input);
        assert!(result.html.starts_with("prefix &amp; entities <b>bold</b>\n\t "));
        assert!(result.html.ends_with(" suffix &lt;"));
        assert!(result.html.contains(r#"<spacer height="4"></spacer>"#));
    }
}
