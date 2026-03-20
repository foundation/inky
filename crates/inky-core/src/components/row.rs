use scraper::ElementRef;

use super::helpers::{build_classes, inner_html};
use crate::attrs::get_attrs;
use crate::config::{Config, OutputMode};

pub fn make_row(element: &ElementRef, config: &Config) -> String {
    let attrs = get_attrs(element);
    let classes = build_classes("row", element);
    let inner = inner_html(element);

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
