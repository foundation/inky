use super::El;

/// `<video src="movie.mp4" poster="poster.jpg" href="https://example.com/watch" width="600">`
///
/// Generates HTML5 `<video>` with poster image and `<img>` fallback.
/// Apple Mail/iOS play the video; all others show the poster linked to `href`.
pub fn make_video(el: &El) -> String {
    let src = el.attr("src").unwrap_or_default();
    let poster = el.attr("poster").unwrap_or_default();
    let href = el.attr("href").unwrap_or_else(|| src.clone());
    let width = el.attr("width").unwrap_or_else(|| "600".to_string());
    let alt = el.attr("alt").unwrap_or_else(|| "Video".to_string());

    let mut html = String::new();

    // Outer link wraps everything — universal fallback
    html.push_str(&format!(
        r#"<a href="{}" target="_blank" style="text-decoration: none;">"#,
        href
    ));

    // HTML5 video tag (Apple Mail / iOS only)
    html.push_str(&format!(
        r#"<video width="{}" autoplay muted loop playsinline poster="{}" style="max-width: 100%; display: block;">"#,
        width, poster
    ));
    html.push_str(&format!(r#"<source src="{}" type="video/mp4">"#, src));

    // Fallback image for clients that don't support <video>
    html.push_str(&format!(
        r#"<img src="{}" alt="{}" width="{}" style="width: {}px; max-width: 100%; display: block;">"#,
        poster, alt, width, width
    ));

    html.push_str("</video></a>");

    html
}
