use scraper::ElementRef;

use crate::attrs::get_attr;

/// `<video src="movie.mp4" poster="poster.jpg" href="https://example.com/watch" width="600">`
///
/// Generates HTML5 `<video>` with poster image and `<img>` fallback.
/// Apple Mail/iOS play the video; all others show the poster linked to `href`.
pub fn make_video(element: &ElementRef) -> String {
    let src = get_attr(element, "src").unwrap_or_default();
    let poster = get_attr(element, "poster").unwrap_or_default();
    let href = get_attr(element, "href").unwrap_or_else(|| src.clone());
    let width = get_attr(element, "width").unwrap_or_else(|| "600".to_string());
    let alt = get_attr(element, "alt").unwrap_or_else(|| "Video".to_string());

    let mut html = String::new();

    // Outer link wraps everything — universal fallback
    html.push_str(&format!(
        r#"<a href="{}" target="_blank" style="text-decoration: none;">"#,
        href
    ));

    // HTML5 video tag (Apple Mail / iOS only)
    // data-parsed prevents the transform loop from re-matching this output <video> tag
    html.push_str(&format!(
        r#"<video data-parsed width="{}" autoplay muted loop playsinline poster="{}" style="max-width: 100%; display: block;">"#,
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
