use super::helpers::build_classes;
use super::El;
use crate::config::{Config, OutputMode};

pub fn make_row(el: &El, config: &Config) -> String {
    let attrs = el.attrs();
    let classes = build_classes("row", el);
    let inner = el.inner();

    match config.output_mode {
        OutputMode::Table => {
            format!(
                r#"<table role="presentation"{} class="{}"><tbody><tr>{}</tr></tbody></table>"#,
                attrs, classes, inner
            )
        }
        OutputMode::Hybrid => {
            format!(
                r#"<!--[if mso]><table role="presentation" width="100%"{} class="{}"><tr><![endif]--><div class="{}" style="font-size:0;">{}</div><!--[if mso]></tr></table><![endif]-->"#,
                attrs, classes, classes, inner
            )
        }
    }
}
