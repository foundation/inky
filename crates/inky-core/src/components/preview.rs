use super::El;

/// `<preview>Your preheader text here</preview>`
///
/// Generates hidden preheader text that appears in inbox preview but is
/// invisible in the email body. Pads with zero-width spaces to prevent
/// other content from leaking into the preview.
pub fn make_preview(el: &El) -> String {
    let text = el.inner();

    // Hidden preheader span
    let mut html = format!(
        r#"<div style="display: none; max-height: 0; overflow: hidden; mso-hide: all;">{}</div>"#,
        text
    );

    // Zero-width space padding to prevent body content from showing in preview
    html.push_str(
        r#"<div style="display: none; max-height: 0; overflow: hidden; mso-hide: all;">&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;</div>"#,
    );

    html
}
