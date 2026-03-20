use inky_core::{Config, Inky, OutputMode};

// --- Bulletproof button tests ---

fn bulletproof_inky() -> Inky {
    Inky::with_config(Config {
        bulletproof_buttons: true,
        ..Config::default()
    })
}

#[test]
fn test_bulletproof_button_per_attribute() {
    let input = r#"<button href="https://example.com" bulletproof>Click</button>"#;
    let result = Inky::new().transform(input);
    assert!(result.contains("<!--[if mso]>"));
    assert!(result.contains("<v:roundrect"));
    assert!(result.contains("https://example.com"));
    assert!(result.contains("<!--[if !mso]><!-->"));
    assert!(result.contains("class=\"button\""));
}

#[test]
fn test_bulletproof_button_global_config() {
    let input = r#"<button href="https://example.com">Click</button>"#;
    let result = bulletproof_inky().transform(input);
    assert!(result.contains("<v:roundrect"));
    assert!(result.contains("<!--[if mso]>"));
}

#[test]
fn test_bulletproof_button_not_triggered_by_default() {
    let input = r#"<button href="https://example.com">Click</button>"#;
    let result = Inky::new().transform(input);
    assert!(!result.contains("<v:roundrect"));
    assert!(!result.contains("<!--[if mso]>"));
}

#[test]
fn test_bulletproof_button_custom_colors() {
    let input = r##"<button href="https://example.com" bulletproof bg-color="#ff0000" text-color="#000000">Go</button>"##;
    let result = Inky::new().transform(input);
    assert!(result.contains(r##"fillcolor="#ff0000""##));
    assert!(result.contains(r##"strokecolor="#ff0000""##));
    assert!(result.contains("color:#000000"));
}

#[test]
fn test_bulletproof_button_custom_dimensions() {
    let input =
        r#"<button href="https://example.com" bulletproof width="300" height="60">Big</button>"#;
    let result = Inky::new().transform(input);
    assert!(result.contains("width:300px"));
    assert!(result.contains("height:60px"));
}

#[test]
fn test_bulletproof_button_custom_radius() {
    let input =
        r#"<button href="https://example.com" bulletproof radius="10" height="40">Round</button>"#;
    let result = Inky::new().transform(input);
    // arcsize = 10 / (40/2) * 100 = 50%
    assert!(result.contains("arcsize=\"50%\""));
}

#[test]
fn test_bulletproof_button_no_href_skips_vml() {
    // No href means no bulletproof VML, just normal output
    let input = r#"<button bulletproof>No link</button>"#;
    let result = Inky::new().transform(input);
    assert!(!result.contains("<v:roundrect"));
}

#[test]
fn test_bulletproof_button_no_leaked_attributes() {
    let input = r##"<button href="https://example.com" bulletproof bg-color="#ff0000" radius="5">Go</button>"##;
    let result = Inky::new().transform(input);
    // These attributes should not appear in the non-MSO <a> tag
    assert!(!result.contains(r#"bulletproof="""#));
    assert!(!result.contains("bg-color="));
    assert!(!result.contains(r#"radius="5""#));
}

#[test]
fn test_bulletproof_button_works_in_hybrid_mode() {
    let input = r#"<button href="https://example.com" bulletproof>Click</button>"#;
    let result = hybrid_inky().transform(input);
    assert!(result.contains("<v:roundrect"));
    assert!(result.contains("class=\"button\""));
}

fn hybrid_inky() -> Inky {
    Inky::with_config(Config {
        output_mode: OutputMode::Hybrid,
        ..Config::default()
    })
}

#[test]
fn test_hybrid_container() {
    let input = "<container>content</container>";
    let result = hybrid_inky().transform(input);
    assert!(result.contains("<!--[if mso]>"));
    assert!(result.contains("<![endif]-->"));
    assert!(result.contains("<div"));
    assert!(result.contains("class=\"container\""));
    assert!(result.contains("max-width:580px"));
    assert!(result.contains("margin:0 auto"));
    // Should NOT contain the table-based output
    assert!(!result.contains("<tbody>"));
}

#[test]
fn test_hybrid_row() {
    let input = "<row>content</row>";
    let result = hybrid_inky().transform(input);
    assert!(result.contains("<!--[if mso]>"));
    assert!(result.contains("<![endif]-->"));
    assert!(result.contains("<div"));
    assert!(result.contains("class=\"row\""));
    assert!(result.contains("font-size:0"));
}

#[test]
fn test_hybrid_column_widths() {
    let input = r#"<row><column lg="6">Left</column><column lg="6">Right</column></row>"#;
    let result = hybrid_inky().transform(input);
    // Each column should be 50%
    assert!(result.contains("max-width:50%"));
    assert!(result.contains("display:inline-block"));
    assert!(result.contains("Left"));
    assert!(result.contains("Right"));
}

#[test]
fn test_hybrid_column_unequal() {
    let input = r#"<row><column lg="4">Sidebar</column><column lg="8">Main</column></row>"#;
    let result = hybrid_inky().transform(input);
    // 4/12 = 33.3333%, 8/12 = 66.6667%
    assert!(result.contains("33.33"));
    assert!(result.contains("66.66"));
}

#[test]
fn test_hybrid_full_width_column() {
    let input = "<row><column>Full width</column></row>";
    let result = hybrid_inky().transform(input);
    assert!(result.contains("max-width:100%"));
    assert!(result.contains("Full width"));
}

#[test]
fn test_hybrid_wrapper() {
    let input = "<wrapper>content</wrapper>";
    let result = hybrid_inky().transform(input);
    assert!(result.contains("<!--[if mso]>"));
    assert!(result.contains("<![endif]-->"));
    assert!(result.contains("<div"));
    assert!(result.contains("class=\"wrapper\""));
}

#[test]
fn test_hybrid_block_grid() {
    let input = r#"<block-grid up="3"><td>A</td><td>B</td><td>C</td></block-grid>"#;
    let result = hybrid_inky().transform(input);
    assert!(result.contains("<!--[if mso]>"));
    assert!(result.contains("<![endif]-->"));
    assert!(result.contains("<div"));
    assert!(result.contains("class=\"block-grid up-3\""));
}

#[test]
fn test_hybrid_mso_comments_balanced() {
    let input = "<container><row><column>Hello</column></row></container>";
    let result = hybrid_inky().transform(input);
    let opens = result.matches("<!--[if mso]>").count();
    let closes = result.matches("<![endif]-->").count();
    assert_eq!(opens, closes, "MSO conditional comments must be balanced");
}

#[test]
fn test_hybrid_no_expander() {
    // In hybrid mode, expander <th> should not appear
    let input = "<row><column>Full</column></row>";
    let result = hybrid_inky().transform(input);
    assert!(!result.contains("expander"));
}

#[test]
fn test_table_mode_unchanged() {
    // Verify default table mode still works as before
    let input = "<container>content</container>";
    let result = Inky::new().transform(input);
    assert!(result.contains("<table"));
    assert!(result.contains("<tbody>"));
    assert!(result.contains("class=\"container\""));
    assert!(!result.contains("<!--[if mso]>"));
}

#[test]
fn test_hybrid_button_unchanged() {
    // Button should still be table-based even in hybrid mode (for now)
    let input = r#"<button href="https://example.com">Click</button>"#;
    let result = hybrid_inky().transform(input);
    assert!(result.contains("https://example.com"));
    assert!(result.contains("Click"));
    assert!(result.contains("class=\"button\""));
}

#[test]
fn test_hybrid_nested_layout() {
    let input =
        r#"<container><row><column lg="6">A</column><column lg="6">B</column></row></container>"#;
    let result = hybrid_inky().transform(input);
    // Should have container div, row div, and column divs
    assert!(result.contains("class=\"container\""));
    assert!(result.contains("class=\"row\""));
    assert!(result.contains("max-width:50%"));
    assert!(result.contains("A"));
    assert!(result.contains("B"));
}
