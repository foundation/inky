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
