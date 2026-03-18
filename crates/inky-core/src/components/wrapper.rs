use scraper::ElementRef;

use super::helpers::{build_classes, inner_html};
use crate::attrs::get_attrs;
use crate::config::{Config, OutputMode};

pub fn make_wrapper(element: &ElementRef, config: &Config) -> String {
    let attrs = get_attrs(element);
    let classes = build_classes("wrapper", element);
    let inner = inner_html(element);

    match config.output_mode {
        OutputMode::Table => {
            format!(
                r#"<table role="presentation"{} class="{}" align="center"><tbody><tr><td class="wrapper-inner">{}</td></tr></tbody></table>"#,
                attrs, classes, inner
            )
        }
        OutputMode::Hybrid => {
            format!(
                r#"<!--[if mso]><table role="presentation" align="center"{} class="{}"><tr><td class="wrapper-inner"><![endif]--><div class="{}" style="width:100%;">{}</div><!--[if mso]></td></tr></table><![endif]-->"#,
                attrs, classes, classes, inner
            )
        }
    }
}
