use super::helpers::build_classes;
use super::El;
use crate::config::{Config, OutputMode};

pub fn make_wrapper(el: &El, config: &Config) -> String {
    let attrs = el.attrs();
    let classes = build_classes("wrapper", el);
    let inner = el.inner();

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
