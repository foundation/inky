/// Errors produced by inky-core's fallible APIs.
///
/// Display output is stable and identical to the pre-2.0 string errors —
/// the variant structure adds matchability without changing messages.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum InkyError {
    /// Layout, include, or custom-component resolution failed.
    #[error("{0}")]
    Include(String),
    /// A file could not be read (used by callers wrapping I/O around the pipeline).
    #[error("{0}")]
    Io(String),
    /// MiniJinja template merge failed.
    #[cfg(feature = "templating")]
    #[error("{0}")]
    Template(String),
    /// Framework SCSS compilation failed.
    #[cfg(feature = "pipeline")]
    #[error(transparent)]
    Scss(Box<grass::Error>),
    /// CSS inlining failed.
    #[cfg(feature = "css-inlining")]
    #[error("{0}")]
    CssInline(String),
}

/// A pipeline failure plus the non-fatal warnings collected before it.
#[cfg(feature = "pipeline")]
#[derive(Debug)]
pub struct PipelineError {
    pub error: InkyError,
    /// Warnings gathered before the failure (previously lost on the Err path).
    pub warnings: Vec<String>,
}

#[cfg(feature = "pipeline")]
impl std::fmt::Display for PipelineError {
    /// Reproduces the pipeline's historical context prefixes exactly.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.error {
            #[cfg(feature = "templating")]
            InkyError::Template(m) => write!(f, "Template merge failed: {}", m),
            InkyError::Scss(e) => write!(f, "SCSS compilation failed: {}", e),
            #[cfg(feature = "css-inlining")]
            InkyError::CssInline(m) => write!(f, "CSS inlining failed: {}", m),
            other => write!(f, "{}", other),
        }
    }
}

#[cfg(feature = "pipeline")]
impl std::error::Error for PipelineError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

#[cfg(test)]
mod tests {
    use crate::InkyError;

    #[test]
    fn include_display_is_bare_message() {
        let e = InkyError::Include("Layout tag is missing src attribute".into());
        assert_eq!(e.to_string(), "Layout tag is missing src attribute");
    }

    #[cfg(feature = "templating")]
    #[test]
    fn template_display_is_bare_message() {
        let e = InkyError::Template("Template parse error: boom".into());
        assert_eq!(e.to_string(), "Template parse error: boom");
    }
}
