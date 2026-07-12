use super::helpers::build_classes;
use super::El;
use crate::config::{Config, OutputMode};

pub fn make_container(el: &El, config: &Config) -> String {
    let attrs = el.attrs();
    let classes = build_classes("container", el);
    let inner = el.inner();

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
