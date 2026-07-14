/// Errors produced by inky-core's fallible APIs.
///
/// Display output is stable and identical to the pre-2.0 string errors —
/// the variant structure adds matchability without changing messages.
///
/// Variant payloads are pre-formatted display messages, stable across
/// releases only in the sense that they are human-readable — match on
/// variants to branch on error kind; never parse the message strings.
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
    #[error("{0}")]
    Scss(String),
    /// CSS inlining failed.
    #[cfg(feature = "css-inlining")]
    #[error("{0}")]
    CssInline(String),
}

/// A pipeline failure plus the non-fatal warnings collected before it.
///
/// Variant payloads are pre-formatted display messages, stable across
/// releases only in the sense that they are human-readable — match on
/// variants to branch on error kind; never parse the message strings.
#[cfg(feature = "pipeline")]
#[derive(Debug)]
#[non_exhaustive]
pub struct PipelineError {
    pub error: InkyError,
    /// Warnings gathered before the failure (previously lost on the Err path).
    pub warnings: Vec<String>,
}

#[cfg(feature = "pipeline")]
impl PipelineError {
    /// Create a pipeline error carrying the warnings collected before the failure.
    pub fn new(error: InkyError, warnings: Vec<String>) -> Self {
        Self { error, warnings }
    }
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

/// `source()` returns `None`: per the `Error::source` convention against
/// duplication, a source should only be returned when it adds information
/// not already in `Display`. This `Display` impl already carries the full
/// message, including the underlying error's text, so re-exposing it as a
/// `source()` causes double-printing in `anyhow`-style `{:#}` chains. The
/// underlying `InkyError` remains available via the public `error` field.
#[cfg(feature = "pipeline")]
impl std::error::Error for PipelineError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
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

    #[cfg(feature = "pipeline")]
    #[test]
    fn pipeline_error_constructor_and_fields() {
        let e = crate::error::PipelineError::new(InkyError::Include("x".into()), vec!["w".into()]);
        assert_eq!(e.warnings, vec!["w".to_string()]);
        assert!(matches!(e.error, InkyError::Include(_)));
    }

    #[cfg(feature = "pipeline")]
    #[test]
    fn pipeline_error_source_is_none() {
        use std::error::Error;
        let e = crate::error::PipelineError::new(InkyError::Include("boom".into()), vec![]);
        assert!(
            e.source().is_none(),
            "source() must be None: Display already carries the inner message"
        );
        assert_eq!(e.to_string(), "boom");
    }
}
