use scraper::ElementRef;

use super::helpers::inner_html;
use crate::attrs::{get_attr, get_classes};
use crate::config::{Config, OutputMode};

pub fn make_block_grid(element: &ElementRef, config: &Config) -> String {
    let up = get_attr(element, "up").unwrap_or_default();
    let mut classes = vec!["block-grid".to_string(), format!("up-{}", up)];
    classes.extend(get_classes(element));
    let class_str = classes.join(" ");
    let inner = inner_html(element);

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
