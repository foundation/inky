use minijinja::{Environment, UndefinedBehavior};
use serde_json::Value as JsonValue;

/// Render a MiniJinja template string with the provided JSON data.
///
/// When `strict` is false (the default), missing variables render as empty strings.
/// When `strict` is true, missing variables cause an error.
pub fn render_template(template: &str, data: &JsonValue, strict: bool) -> Result<String, String> {
    let mut env = Environment::new();
    if strict {
        env.set_undefined_behavior(UndefinedBehavior::Strict);
    } else {
        env.set_undefined_behavior(UndefinedBehavior::Lenient);
    }

    env.add_template("__inky__", template)
        .map_err(|e| format!("Template parse error: {}", e))?;

    let tmpl = env
        .get_template("__inky__")
        .map_err(|e| format!("Template error: {}", e))?;

    let ctx = minijinja::Value::from_serialize(data);
    tmpl.render(ctx)
        .map_err(|e| format!("Template render error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_variable_substitution() {
        let result = render_template("Hello {{ name }}!", &json!({"name": "World"}), false);
        assert_eq!(result.unwrap(), "Hello World!");
    }

    #[test]
    fn test_conditional_true() {
        let tmpl = "{% if show %}visible{% endif %}";
        let result = render_template(tmpl, &json!({"show": true}), false);
        assert_eq!(result.unwrap(), "visible");
    }

    #[test]
    fn test_conditional_false() {
        let tmpl = "{% if show %}visible{% endif %}";
        let result = render_template(tmpl, &json!({"show": false}), false);
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_loop() {
        let tmpl = "{% for item in items %}{{ item }} {% endfor %}";
        let result = render_template(tmpl, &json!({"items": ["a", "b", "c"]}), false);
        assert_eq!(result.unwrap(), "a b c ");
    }

    #[test]
    fn test_missing_key_lenient() {
        let result = render_template("Hello {{ name }}!", &json!({}), false);
        assert_eq!(result.unwrap(), "Hello !");
    }

    #[test]
    fn test_missing_key_strict() {
        let result = render_template("Hello {{ name }}!", &json!({}), true);
        assert!(result.is_err());
    }

    #[test]
    fn test_nested_data() {
        let tmpl = "{{ user.name }} ({{ user.email }})";
        let data = json!({"user": {"name": "Alice", "email": "alice@example.com"}});
        let result = render_template(tmpl, &data, false);
        assert_eq!(result.unwrap(), "Alice (alice@example.com)");
    }

    #[test]
    fn test_filter() {
        let result = render_template("{{ name | upper }}", &json!({"name": "hello"}), false);
        assert_eq!(result.unwrap(), "HELLO");
    }

    #[test]
    fn test_html_in_template() {
        let tmpl = r#"<button href="{{ url }}">{{ text }}</button>"#;
        let data = json!({"url": "https://example.com", "text": "Click me"});
        let result = render_template(tmpl, &data, false);
        assert_eq!(
            result.unwrap(),
            r#"<button href="https://example.com">Click me</button>"#
        );
    }
}
