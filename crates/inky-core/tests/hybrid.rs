use inky_core::{Config, Inky, OutputMode};

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
