pub(crate) mod scanner;

use scanner::{scan, Attr, Doc, Quote, Token};

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

    // 9. <center><menu ...> → <menu align="center" ...>
    migrate_centered_menu(&mut doc, &mut changes);

    MigrateResult {
        html: doc.emit(),
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
        (
            "color",
            &["primary", "secondary", "success", "alert", "warning"],
        ),
    ],
    boolean: &["expand", "expanded", "radius", "rounded", "hollow"],
};

const CALLOUT_RULE: ClassRule = ClassRule {
    valued: &[(
        "color",
        &["primary", "secondary", "success", "alert", "warning"],
    )],
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
        let original_quote = tag.attrs[class_idx].quote;

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
            let remaining_value = remaining.join(" ");
            // Never emit a double-quoted attribute whose value contains a
            // literal `"` — that would corrupt the markup. Prefer single
            // quotes when the value needs them, otherwise keep the
            // original quote style if it was single, else double.
            let quote = if remaining_value.contains('"') || original_quote == Quote::Single {
                Quote::Single
            } else {
                Quote::Double
            };
            replacement.push(Attr {
                name: "class".to_string(),
                name_out: "class".to_string(),
                value: Some(remaining_value),
                quote,
            });
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

/// Migrate `<center><menu ...>...</menu></center>` to `<menu align="center" ...>`.
/// Matches only an attribute-less `<center>` directly wrapping a single menu
/// (the same shape the old regex matched).
fn migrate_centered_menu(doc: &mut Doc, changes: &mut Vec<MigrateChange>) {
    let mut changed = false;
    let mut i = 0;
    while i < doc.tokens.len() {
        // <center> with no attributes
        let Token::Open(center) = &doc.tokens[i] else {
            i += 1;
            continue;
        };
        if center.name != "center" || !center.attrs.is_empty() || center.self_closing {
            i += 1;
            continue;
        }

        // optional whitespace-only text, then <menu ...>
        let mut j = i + 1;
        let ws_before = matches!(&doc.tokens.get(j), Some(Token::Text(r)) if doc.src[r.clone()].trim().is_empty());
        if ws_before {
            j += 1;
        }
        let Some(Token::Open(menu)) = doc.tokens.get(j) else {
            i += 1;
            continue;
        };
        if menu.name != "menu" || menu.self_closing {
            i += 1;
            continue;
        }

        // first </menu> after j (menus don't nest — matches the old non-greedy regex)
        let Some(close_menu_rel) = doc.tokens[j + 1..]
            .iter()
            .position(|t| matches!(t, Token::Close(c) if c.name == "menu"))
        else {
            i += 1;
            continue;
        };
        let close_menu_idx = j + 1 + close_menu_rel;

        // optional whitespace-only text, then </center>
        let mut k = close_menu_idx + 1;
        let ws_after = matches!(&doc.tokens.get(k), Some(Token::Text(r)) if doc.src[r.clone()].trim().is_empty());
        if ws_after {
            k += 1;
        }
        let Some(Token::Close(close_center)) = doc.tokens.get(k) else {
            i += 1;
            continue;
        };
        if close_center.name != "center" {
            i += 1;
            continue;
        }

        // Apply: drop <center>/</center> (and the whitespace runs the old
        // regex consumed), add align="center" to the menu tag.
        if let Token::Open(center) = &mut doc.tokens[i] {
            center.deleted = true;
        }
        if ws_before {
            if let Token::Text(range) = &mut doc.tokens[i + 1] {
                *range = 0..0;
            }
        }
        if let Token::Open(menu) = &mut doc.tokens[j] {
            menu.attrs.push(Attr::new_double("align", "center"));
            menu.dirty = true;
        }
        if ws_after {
            if let Token::Text(range) = &mut doc.tokens[close_menu_idx + 1] {
                *range = 0..0;
            }
        }
        if let Token::Close(close_center) = &mut doc.tokens[k] {
            close_center.deleted = true;
        }
        changed = true;
        i = k + 1;
    }
    if changed {
        changes.push(MigrateChange {
            description: r#"<center><menu> → <menu align="center">"#.to_string(),
        });
    }
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
        assert!(
            result.html.contains(r#"id="promo""#),
            "attribute dropped: {}",
            result.html
        );
    }

    #[test]
    fn menu_attr_value_with_slash_not_corrupted() {
        let input = r#"<menu class="vertical" data-url="a/b">x</menu>"#;
        let result = migrate(input);
        assert!(result.html.contains(r#"direction="vertical""#));
        assert!(
            result.html.contains(r#"data-url="a/b""#),
            "value corrupted: {}",
            result.html
        );
    }

    #[test]
    fn data_large_attribute_untouched() {
        let input = r#"<column data-large="4" large="6">x</column>"#;
        let result = migrate(input);
        assert!(
            result.html.contains(r#"data-large="4""#),
            "data-* corrupted: {}",
            result.html
        );
        assert!(
            result.html.contains(r#"lg="6""#),
            "real attr unmigrated: {}",
            result.html
        );
        assert!(!result.html.contains("data-lg"));
    }

    #[test]
    fn attr_value_containing_attr_syntax_untouched() {
        let input = r#"<column title="large=6" large="4">x</column>"#;
        let result = migrate(input);
        assert!(
            result.html.contains(r#"title="large=6""#),
            "value rewritten: {}",
            result.html
        );
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
        assert!(
            result.html.contains("lg='6'"),
            "quote style changed: {}",
            result.html
        );
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
        assert!(
            result.html.contains("<column"),
            "uppercase tag skipped: {}",
            result.html
        );
        assert!(result.html.contains(r#"lg="6""#));
        assert!(result.html.contains("</column>"));
    }

    #[test]
    fn bytes_outside_migrated_tags_preserved() {
        let input =
            "prefix &amp; entities <b>bold</b>\n\t <spacer size=\"4\"></spacer> suffix &lt;";
        let result = migrate(input);
        assert!(result
            .html
            .starts_with("prefix &amp; entities <b>bold</b>\n\t "));
        assert!(result.html.ends_with(" suffix &lt;"));
        assert!(result.html.contains(r#"<spacer height="4"></spacer>"#));
    }

    // --- Task 3: token-based centered-menu rule ---

    #[test]
    fn centered_menu_attrs_preserved_and_center_removed() {
        let input = r##"<center><menu class="vertical" data-x="1"><item href="#">L</item></menu></center>"##;
        let result = migrate(input);
        assert!(result.html.contains(r#"direction="vertical""#));
        assert!(result.html.contains(r#"data-x="1""#));
        assert!(result.html.contains(r#"align="center""#));
        assert!(!result.html.contains("<center>"));
        assert!(!result.html.contains("</center>"));
    }

    #[test]
    fn center_with_attributes_not_unwrapped() {
        // The old regex only matched a bare <center>; keep that behavior.
        let input = r##"<center class="x"><menu><item href="#">L</item></menu></center>"##;
        let result = migrate(input);
        assert!(result.html.contains(r#"<center class="x">"#));
        assert!(!result.html.contains("align=\"center\""));
    }

    #[test]
    fn center_without_menu_untouched() {
        let input = "<center><p>hi</p></center>";
        let result = migrate(input);
        assert_eq!(result.html, input);
    }

    #[test]
    fn centered_menu_with_whitespace_between() {
        let input = "<center>\n  <menu><item href=\"#\">L</item></menu>\n</center>";
        let result = migrate(input);
        assert!(result.html.contains(r#"align="center""#));
        assert!(!result.html.contains("<center>"));
    }

    #[test]
    fn multiple_centered_menus_all_convert() {
        let input = r##"<center><menu><item href="#">a</item></menu></center><center><menu><item href="#">b</item></menu></center>"##;
        let result = migrate(input);
        assert_eq!(result.html.matches(r#"align="center""#).count(), 2);
        assert!(!result.html.contains("<center>"));
        // one MigrateChange per rule invocation, not per match (matches old behavior)
        assert_eq!(
            result
                .changes
                .iter()
                .filter(|c| c.description.contains("align"))
                .count(),
            1
        );
    }

    #[test]
    fn nested_center_only_inner_unwrapped() {
        let input = "<center><center><menu><item href=\"#\">a</item></menu></center></center>";
        let result = migrate(input);
        assert!(result.html.contains(r#"align="center""#));
        // The outer <center> stays: its immediate child is another <center>, not a <menu>.
        assert!(result.html.contains("<center>"));
    }

    // --- opacity regressions (the scanner guarantees these end-to-end) ---

    #[test]
    fn commented_out_v1_untouched() {
        let input = r#"<!-- <columns large="6">old</columns> --><p>x</p>"#;
        let result = migrate(input);
        assert_eq!(result.html, input);
        assert!(result.changes.is_empty());
    }

    #[test]
    fn raw_block_untouched() {
        let input = r#"<raw><columns large="6">keep v1</columns></raw>"#;
        let result = migrate(input);
        assert_eq!(result.html, input);
    }

    #[test]
    fn script_and_style_content_untouched() {
        let input = r#"<script>var t = '<spacer size="4">';</script><style>/* <h-line> */</style>"#;
        let result = migrate(input);
        assert_eq!(result.html, input);
    }

    #[test]
    fn erb_content_untouched_but_surroundings_migrate() {
        let input = r#"<%= tag("<columns>") %><spacer size="8"></spacer>"#;
        let result = migrate(input);
        assert!(result.html.starts_with(r#"<%= tag("<columns>") %>"#));
        assert!(result.html.contains(r#"height="8""#));
    }

    #[test]
    fn merge_tag_in_attr_value_preserved() {
        let input = r#"<spacer size="<%= n %>"></spacer>"#;
        let result = migrate(input);
        assert_eq!(result.html, r#"<spacer height="<%= n %>"></spacer>"#);
    }

    // --- Final code review regression tests ---

    #[test]
    fn unterminated_close_tag_never_swallows_bytes() {
        let input = "before\n</columns\n<spacer size=\"4\"></spacer>\nafter";
        let result = migrate(input);
        assert!(
            result.html.contains("</columns\n"),
            "malformed close mangled: {}",
            result.html
        );
        assert!(result.html.contains(r#"height="4""#));
        assert!(result.html.contains("after"));
    }

    #[test]
    fn close_tag_with_junk_is_literal_text() {
        let input = r#"</columns junk="a>b">text"#;
        let result = migrate(input);
        assert_eq!(result.html, input);
        assert!(result.changes.is_empty());
    }

    #[test]
    fn close_tag_with_whitespace_still_renames() {
        let input = "<columns>x</columns  >";
        let result = migrate(input);
        assert_eq!(result.html, "<column>x</column>");
    }

    #[test]
    fn class_with_embedded_double_quote_not_corrupted() {
        let input = r##"<button class='small has"x' href="#">Go</button>"##;
        let result = migrate(input);
        assert!(result.html.contains(r#"size="small""#));
        assert!(
            result.html.contains(r#"class='has"x'"#),
            "class re-quoting corrupted: {}",
            result.html
        );
    }

    #[test]
    fn single_quoted_class_attr_keeps_quote_style() {
        let input = "<callout class='primary custom'>M</callout>";
        let result = migrate(input);
        assert!(
            result.html.contains("class='custom'"),
            "quote style changed: {}",
            result.html
        );
        assert!(result.html.contains(r#"color="primary""#));
    }

    #[test]
    fn self_closing_raw_does_not_swallow_document() {
        let input = r#"<raw/><spacer size="8"></spacer>"#;
        let result = migrate(input);
        assert!(
            result.html.contains(r#"height="8""#),
            "raw/ swallowed the document: {}",
            result.html
        );
        assert!(result.html.starts_with("<raw/>"));
    }
}
