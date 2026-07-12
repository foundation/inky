use super::El;
use crate::config::{Config, OutputMode};

pub fn make_block_grid(el: &El, config: &Config) -> String {
    let up = el.attr("up").unwrap_or_default();
    let mut classes = vec!["block-grid".to_string(), format!("up-{}", up)];
    classes.extend(el.classes());
    let class_str = classes.join(" ");
    let inner = el.inner();

    match config.output_mode {
        OutputMode::Table => {
            format!(
                r#"<table role="presentation" class="{}"><tbody><tr>{}</tr></tbody></table>"#,
                class_str, inner
            )
        }
        OutputMode::Hybrid => {
            format!(
                r#"<!--[if mso]><table role="presentation" class="{}"><tr><![endif]--><div class="{}" style="font-size:0;">{}</div><!--[if mso]></tr></table><![endif]-->"#,
                class_str, class_str, inner
            )
        }
    }
}
