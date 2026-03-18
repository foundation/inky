#[cfg(feature = "templating")]
mod tests {
    use inky_core::templating::render_template;
    use inky_core::Inky;
    use serde_json::json;

    // --- Basic merge then transform integration tests ---

    #[test]
    fn test_merge_then_transform_button() {
        let template = r#"<button href="{{ url }}">{{ text }}</button>"#;
        let data = json!({"url": "https://example.com", "text": "Click me"});
        let merged = render_template(template, &data, false).unwrap();
        let result = Inky::new().transform(&merged);
        assert!(result.contains("https://example.com"));
        assert!(result.contains("Click me"));
        assert!(result.contains(r#"class="button""#));
    }

    #[test]
    fn test_merge_conditional_component() {
        let template = r#"{% if show_hero %}<hero><h1>{{ title }}</h1></hero>{% endif %}"#;

        // With show_hero = true
        let data = json!({"show_hero": true, "title": "Welcome"});
        let merged = render_template(template, &data, false).unwrap();
        let result = Inky::new().transform(&merged);
        assert!(result.contains("Welcome"));
        assert!(result.contains(r#"class="hero""#));

        // With show_hero = false
        let data = json!({"show_hero": false});
        let merged = render_template(template, &data, false).unwrap();
        let result = Inky::new().transform(&merged);
        assert!(!result.contains("hero"));
    }

    #[test]
    fn test_merge_loop_with_rows() {
        let template =
            r#"{% for item in items %}<row><column>{{ item.name }}</column></row>{% endfor %}"#;
        let data = json!({
            "items": [
                {"name": "Alpha"},
                {"name": "Beta"},
                {"name": "Gamma"}
            ]
        });
        let merged = render_template(template, &data, false).unwrap();
        let result = Inky::new().transform(&merged);
        assert!(result.contains("Alpha"));
        assert!(result.contains("Beta"));
        assert!(result.contains("Gamma"));
        // Should have 3 row tables
        assert_eq!(result.matches(r#"class="row""#).count(), 3);
    }

    #[test]
    fn test_merge_nested_data_in_attributes() {
        let template = r#"<button href="{{ user.profile_url }}" class="{{ style }}">Hello {{ user.name }}</button>"#;
        let data = json!({
            "user": {"name": "Alice", "profile_url": "https://example.com/alice"},
            "style": "primary"
        });
        let merged = render_template(template, &data, false).unwrap();
        let result = Inky::new().transform(&merged);
        assert!(result.contains("https://example.com/alice"));
        assert!(result.contains("Hello Alice"));
    }

    #[test]
    fn test_merge_preserves_inky_structure() {
        let template = r#"<container><row><column lg="6">{{ left }}</column><column lg="6">{{ right }}</column></row></container>"#;
        let data = json!({"left": "Left content", "right": "Right content"});
        let merged = render_template(template, &data, false).unwrap();
        let result = Inky::new().transform(&merged);
        assert!(result.contains("Left content"));
        assert!(result.contains("Right content"));
        assert!(result.contains(r#"class="container""#));
        assert!(result.contains(r#"class="row""#));
    }

    #[test]
    fn test_merge_with_callout() {
        let template = r#"{% if coupon %}<callout>Use code <strong>{{ coupon }}</strong></callout>{% endif %}"#;
        let data = json!({"coupon": "SAVE20"});
        let merged = render_template(template, &data, false).unwrap();
        let result = Inky::new().transform(&merged);
        assert!(result.contains("SAVE20"));
        assert!(result.contains(r#"class="callout""#));
    }

    #[test]
    fn test_merge_with_spacer() {
        let template = r#"<spacer height="{{ height }}"></spacer>"#;
        let data = json!({"height": "20"});
        let merged = render_template(template, &data, false).unwrap();
        let result = Inky::new().transform(&merged);
        assert!(result.contains(r#"height="20""#));
    }

    // --- Edge cases ---

    #[test]
    fn test_no_merge_tags_passthrough() {
        let template = "<button href=\"https://example.com\">Click</button>";
        let data = json!({});
        let merged = render_template(template, &data, false).unwrap();
        let result = Inky::new().transform(&merged);
        assert!(result.contains("https://example.com"));
        assert!(result.contains("Click"));
    }

    #[test]
    fn test_merge_empty_object() {
        let template = "Hello {{ name }}!";
        let data = json!({});
        let merged = render_template(template, &data, false).unwrap();
        assert_eq!(merged, "Hello !");
    }

    #[test]
    fn test_merge_with_filter_and_transform() {
        let template = r#"<button href="{{ url }}">{{ label | upper }}</button>"#;
        let data = json!({"url": "https://example.com", "label": "click here"});
        let merged = render_template(template, &data, false).unwrap();
        let result = Inky::new().transform(&merged);
        assert!(result.contains("CLICK HERE"));
    }

    #[test]
    fn test_merge_with_default_filter() {
        let template = r#"<button href="{{ url | default('https://fallback.com') }}">Go</button>"#;
        let data = json!({});
        let merged = render_template(template, &data, false).unwrap();
        let result = Inky::new().transform(&merged);
        assert!(result.contains("https://fallback.com"));
    }

    #[test]
    fn test_merge_preserves_raw_blocks() {
        let template = "<raw><p>{{ not_merged }}</p></raw>";
        let data = json!({"not_merged": "should not appear"});
        // MiniJinja will still process {{ }} inside <raw> since it runs before Inky's raw extraction.
        // This is expected — MiniJinja doesn't know about <raw> tags.
        let merged = render_template(template, &data, false).unwrap();
        assert!(merged.contains("should not appear"));
    }

    #[test]
    fn test_merge_html_passthrough() {
        // Auto-escaping is off (no file extension on internal template name),
        // so HTML in data passes through. This is desirable for email templates
        // where users may want to inject HTML snippets.
        let template = "<p>{{ message }}</p>";
        let data = json!({"message": "Price: $5 & tax"});
        let merged = render_template(template, &data, false).unwrap();
        assert!(merged.contains("Price: $5 & tax"));
    }

    #[test]
    fn test_merge_html_escape_filter() {
        let template = "<p>{{ message | e }}</p>";
        let data = json!({"message": "<strong>Bold</strong>"});
        let merged = render_template(template, &data, false).unwrap();
        assert!(merged.contains("&lt;strong&gt;"));
    }

    #[test]
    fn test_merge_elif_chain() {
        let template = r#"{% if tier == "premium" %}Premium{% elif tier == "trial" %}Trial{% else %}Free{% endif %}"#;

        let data = json!({"tier": "premium"});
        assert_eq!(render_template(template, &data, false).unwrap(), "Premium");

        let data = json!({"tier": "trial"});
        assert_eq!(render_template(template, &data, false).unwrap(), "Trial");

        let data = json!({"tier": "free"});
        assert_eq!(render_template(template, &data, false).unwrap(), "Free");
    }

    #[test]
    fn test_merge_loop_index() {
        let template = "{% for item in items %}{{ loop.index }}:{{ item }} {% endfor %}";
        let data = json!({"items": ["a", "b", "c"]});
        let merged = render_template(template, &data, false).unwrap();
        assert_eq!(merged, "1:a 2:b 3:c ");
    }

    #[test]
    fn test_strict_mode_error_message() {
        let template = "Hello {{ name }}!";
        let data = json!({});
        let result = render_template(template, &data, true);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("undefined") || err.contains("unknown"),
            "Expected error about undefined variable, got: {}",
            err
        );
    }
}
