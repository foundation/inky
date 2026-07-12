use std::path::Path;

use colored::Colorize;
use inky_core::pipeline::{Pipeline, PipelineOptions};
use inky_core::Config;

/// How to handle errors during the build pipeline.
#[derive(Clone, Copy)]
pub enum ErrorMode {
    /// Exit the process on error (for `inky build`)
    Exit,
    /// Log the error and continue with empty output (for `inky watch`)
    Continue,
}

/// Common build parameters shared across build, watch, and serve commands.
#[derive(Clone)]
pub struct BuildContext {
    pub inline_css: bool,
    pub framework_css: bool,
    pub components_dir: Option<String>,
    pub error_mode: ErrorMode,
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

/// Temporary compatibility wrapper over `inky_core::pipeline::Pipeline`.
/// Tasks 3–5 move callers onto `builder::Builder`; Task 6 deletes this.
pub fn process_template(
    config: &Config,
    html: &str,
    ctx: &BuildContext,
    base_path: Option<&Path>,
    merge_data: Option<&serde_json::Value>,
) -> String {
    let pipeline = Pipeline::new(config.clone(), ctx.pipeline_options());
    match pipeline.process(html, base_path, merge_data) {
        Ok(processed) => {
            for w in &processed.warnings {
                eprintln!("  {} {}", "warning:".yellow().bold(), w);
            }
            processed.html
        }
        Err(e) => {
            eprintln!("{} {}", "error:".red().bold(), e);
            match ctx.error_mode {
                ErrorMode::Exit => std::process::exit(1),
                ErrorMode::Continue => String::new(),
            }
        }
    }
}
