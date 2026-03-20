use scraper::ElementRef;

use super::helpers::{build_classes, inner_html};
use crate::attrs::get_attrs;
use crate::config::{Config, OutputMode};

pub fn make_container(element: &ElementRef, config: &Config) -> String {
    let attrs = get_attrs(element);
    let classes = build_classes("container", element);
    let inner = inner_html(element);

    match config.output_mode {
        OutputMode::Table => {
            let align = if attrs.contains("align=") {
                ""
            } else {
                " align=\"center\""
            };
            format!(
                r#"<table role="presentation"{}{} class="{}"><tbody><tr><td class="container-inner">{}</td></tr></tbody></table>"#,
                attrs, align, classes, inner
            )
        }
        OutputMode::Hybrid => {
            format!(
                r#"<!--[if mso]><table role="presentation" width="580" align="center"><tr><td><![endif]--><div{} class="{}" style="max-width:580px;margin:0 auto;">{}</div><!--[if mso]></td></tr></table><![endif]-->"#,
                attrs, classes, inner
            )
        }
    }
}
