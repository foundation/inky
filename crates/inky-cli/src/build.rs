use inky_core::pipeline::PipelineOptions;

/// Common build parameters shared across build, watch, and serve commands.
#[derive(Clone)]
pub struct BuildContext {
    pub inline_css: bool,
    pub framework_css: bool,
    pub components_dir: Option<String>,
    pub output_mode: inky_core::OutputMode,
    pub columns: u32,
    pub bulletproof_buttons: bool,
    pub plain_text: bool,
    pub json: bool,
}

impl BuildContext {
    pub fn pipeline_options(&self) -> PipelineOptions {
        PipelineOptions {
            inline_css: self.inline_css,
            framework_css: self.framework_css,
            components_dir: self
                .components_dir
                .clone()
                .unwrap_or_else(|| "components".to_string()),
        }
    }
}
