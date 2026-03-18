use regex::Regex;

/// An RGBA color with components stored as f64.
#[derive(Debug, Clone, PartialEq)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Color {
    /// Create a new Color from RGBA values (r/g/b: 0-255, a: 0-1).
    pub fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self { r, g, b, a }
    }

    /// Parse a CSS color string. Supports hex, rgb(), rgba(), and common named colors.
    pub fn parse(s: &str) -> Option<Color> {
        let s = s.trim().to_lowercase();

        // Named colors
        if let Some(c) = parse_named_color(&s) {
            return Some(c);
        }

        // Hex
        if s.starts_with('#') {
            return parse_hex(&s);
        }

        // rgb() / rgba()
        if s.starts_with("rgb") {
            return parse_rgb_func(&s);
        }

        None
    }

    /// Compute the relative luminance per WCAG 2.1.
    /// See <https://www.w3.org/TR/WCAG21/#dfn-relative-luminance>
    pub fn relative_luminance(&self) -> f64 {
        let r = linearize(self.r / 255.0);
        let g = linearize(self.g / 255.0);
        let b = linearize(self.b / 255.0);
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }
}

/// Linearize an sRGB channel value (0-1) for luminance calculation.
fn linearize(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Compute the WCAG contrast ratio between two colors.
/// Returns a value >= 1.0, with the lighter color always in the numerator.
pub fn contrast_ratio(c1: &Color, c2: &Color) -> f64 {
    let l1 = c1.relative_luminance();
    let l2 = c2.relative_luminance();
    let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
    (lighter + 0.05) / (darker + 0.05)
}

/// Extract a CSS property value from an inline style string.
/// e.g. `extract_css_property("color: red; font-size: 14px", "color")` => `Some("red")`
pub fn extract_css_property(style: &str, property: &str) -> Option<String> {
    for decl in style.split(';') {
        let decl = decl.trim();
        if let Some(colon_pos) = decl.find(':') {
            let prop = decl[..colon_pos].trim();
            if prop.eq_ignore_ascii_case(property) {
                let value = decl[colon_pos + 1..].trim();
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

fn parse_hex(s: &str) -> Option<Color> {
    let hex = &s[1..]; // strip '#'
    match hex.len() {
        3 => {
            // #RGB
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            Some(Color::new(r as f64, g as f64, b as f64, 1.0))
        }
        6 => {
            // #RRGGBB
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color::new(r as f64, g as f64, b as f64, 1.0))
        }
        8 => {
            // #RRGGBBAA
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some(Color::new(r as f64, g as f64, b as f64, a as f64 / 255.0))
        }
        _ => None,
    }
}

fn parse_rgb_func(s: &str) -> Option<Color> {
    // Match rgb(...) or rgba(...)
    let re = Regex::new(r"rgba?\(\s*(.+?)\s*\)").ok()?;
    let caps = re.captures(s)?;
    let inner = caps.get(1)?.as_str();

    // Modern syntax: rgb(255 255 255 / 0.5)
    if inner.contains('/') {
        let parts: Vec<&str> = inner.split('/').collect();
        if parts.len() != 2 {
            return None;
        }
        let rgb_part = parts[0].trim();
        let alpha_str = parts[1].trim();
        let nums: Vec<f64> = rgb_part
            .split_whitespace()
            .filter_map(|n| n.trim_end_matches(',').parse().ok())
            .collect();
        if nums.len() != 3 {
            return None;
        }
        let a: f64 = alpha_str.parse().ok()?;
        return Some(Color::new(nums[0], nums[1], nums[2], a));
    }

    // Traditional comma-separated: rgb(255, 255, 255) or rgba(255, 255, 255, 0.5)
    let nums: Vec<&str> = inner.split(',').map(|n| n.trim()).collect();
    match nums.len() {
        3 => {
            let r: f64 = nums[0].parse().ok()?;
            let g: f64 = nums[1].parse().ok()?;
            let b: f64 = nums[2].parse().ok()?;
            Some(Color::new(r, g, b, 1.0))
        }
        4 => {
            let r: f64 = nums[0].parse().ok()?;
            let g: f64 = nums[1].parse().ok()?;
            let b: f64 = nums[2].parse().ok()?;
            let a: f64 = nums[3].parse().ok()?;
            Some(Color::new(r, g, b, a))
        }
        _ => None,
    }
}

fn parse_named_color(name: &str) -> Option<Color> {
    let (r, g, b, a) = match name {
        "white" => (255, 255, 255, 255),
        "black" => (0, 0, 0, 255),
        "red" => (255, 0, 0, 255),
        "green" => (0, 128, 0, 255),
        "blue" => (0, 0, 255, 255),
        "yellow" => (255, 255, 0, 255),
        "orange" => (255, 165, 0, 255),
        "purple" => (128, 0, 128, 255),
        "gray" | "grey" => (128, 128, 128, 255),
        "navy" => (0, 0, 128, 255),
        "maroon" => (128, 0, 0, 255),
        "teal" => (0, 128, 128, 255),
        "silver" => (192, 192, 192, 255),
        "olive" => (128, 128, 0, 255),
        "lime" => (0, 255, 0, 255),
        "aqua" => (0, 255, 255, 255),
        "fuchsia" => (255, 0, 255, 255),
        "transparent" => (0, 0, 0, 0),
        _ => return None,
    };
    Some(Color::new(r as f64, g as f64, b as f64, a as f64 / 255.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Color parsing ---

    #[test]
    fn test_parse_hex_3() {
        let c = Color::parse("#fff").unwrap();
        assert_eq!(c.r, 255.0);
        assert_eq!(c.g, 255.0);
        assert_eq!(c.b, 255.0);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn test_parse_hex_3_color() {
        let c = Color::parse("#f00").unwrap();
        assert_eq!(c.r, 255.0);
        assert_eq!(c.g, 0.0);
        assert_eq!(c.b, 0.0);
    }

    #[test]
    fn test_parse_hex_6() {
        let c = Color::parse("#ff8800").unwrap();
        assert_eq!(c.r, 255.0);
        assert_eq!(c.g, 136.0);
        assert_eq!(c.b, 0.0);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn test_parse_hex_8() {
        let c = Color::parse("#ff000080").unwrap();
        assert_eq!(c.r, 255.0);
        assert_eq!(c.g, 0.0);
        assert_eq!(c.b, 0.0);
        assert!((c.a - 128.0 / 255.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_hex_uppercase() {
        let c = Color::parse("#FF0000").unwrap();
        assert_eq!(c.r, 255.0);
        assert_eq!(c.g, 0.0);
        assert_eq!(c.b, 0.0);
    }

    #[test]
    fn test_parse_rgb() {
        let c = Color::parse("rgb(100, 200, 50)").unwrap();
        assert_eq!(c.r, 100.0);
        assert_eq!(c.g, 200.0);
        assert_eq!(c.b, 50.0);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn test_parse_rgba() {
        let c = Color::parse("rgba(100, 200, 50, 0.5)").unwrap();
        assert_eq!(c.r, 100.0);
        assert_eq!(c.g, 200.0);
        assert_eq!(c.b, 50.0);
        assert_eq!(c.a, 0.5);
    }

    #[test]
    fn test_parse_rgb_modern_syntax() {
        let c = Color::parse("rgb(255 255 255 / 0.5)").unwrap();
        assert_eq!(c.r, 255.0);
        assert_eq!(c.g, 255.0);
        assert_eq!(c.b, 255.0);
        assert_eq!(c.a, 0.5);
    }

    #[test]
    fn test_parse_named_white() {
        let c = Color::parse("white").unwrap();
        assert_eq!(c.r, 255.0);
        assert_eq!(c.g, 255.0);
        assert_eq!(c.b, 255.0);
    }

    #[test]
    fn test_parse_named_black() {
        let c = Color::parse("black").unwrap();
        assert_eq!(c.r, 0.0);
        assert_eq!(c.g, 0.0);
        assert_eq!(c.b, 0.0);
    }

    #[test]
    fn test_parse_named_transparent() {
        let c = Color::parse("transparent").unwrap();
        assert_eq!(c.a, 0.0);
    }

    #[test]
    fn test_parse_named_grey_alias() {
        let c1 = Color::parse("gray").unwrap();
        let c2 = Color::parse("grey").unwrap();
        assert_eq!(c1.r, c2.r);
        assert_eq!(c1.g, c2.g);
        assert_eq!(c1.b, c2.b);
    }

    #[test]
    fn test_parse_all_named_colors() {
        let names = [
            "white",
            "black",
            "red",
            "blue",
            "green",
            "yellow",
            "orange",
            "purple",
            "gray",
            "grey",
            "navy",
            "maroon",
            "teal",
            "silver",
            "olive",
            "lime",
            "aqua",
            "fuchsia",
            "transparent",
        ];
        for name in names {
            assert!(Color::parse(name).is_some(), "Failed to parse: {}", name);
        }
    }

    #[test]
    fn test_parse_invalid() {
        assert!(Color::parse("not-a-color").is_none());
        assert!(Color::parse("#zzzzzz").is_none());
        assert!(Color::parse("").is_none());
        assert!(Color::parse("#12345").is_none());
    }

    #[test]
    fn test_parse_with_whitespace() {
        let c = Color::parse("  #ff0000  ").unwrap();
        assert_eq!(c.r, 255.0);
    }

    // --- Luminance ---

    #[test]
    fn test_luminance_white() {
        let c = Color::parse("white").unwrap();
        assert!((c.relative_luminance() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_luminance_black() {
        let c = Color::parse("black").unwrap();
        assert!((c.relative_luminance() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_luminance_red() {
        let c = Color::parse("red").unwrap();
        // Red relative luminance should be ~0.2126
        assert!((c.relative_luminance() - 0.2126).abs() < 0.001);
    }

    // --- Contrast ratio ---

    #[test]
    fn test_contrast_white_black() {
        let white = Color::parse("white").unwrap();
        let black = Color::parse("black").unwrap();
        let ratio = contrast_ratio(&white, &black);
        assert!((ratio - 21.0).abs() < 0.1);
    }

    #[test]
    fn test_contrast_same_color() {
        let c = Color::parse("#808080").unwrap();
        let ratio = contrast_ratio(&c, &c);
        assert!((ratio - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_contrast_order_independent() {
        let white = Color::parse("white").unwrap();
        let black = Color::parse("black").unwrap();
        assert_eq!(
            contrast_ratio(&white, &black),
            contrast_ratio(&black, &white)
        );
    }

    // --- extract_css_property ---

    #[test]
    fn test_extract_color() {
        let style = "color: red; font-size: 14px";
        assert_eq!(
            extract_css_property(style, "color"),
            Some("red".to_string())
        );
    }

    #[test]
    fn test_extract_background_color() {
        let style = "background-color: #fff; color: black";
        assert_eq!(
            extract_css_property(style, "background-color"),
            Some("#fff".to_string())
        );
    }

    #[test]
    fn test_extract_missing_property() {
        let style = "color: red";
        assert_eq!(extract_css_property(style, "background-color"), None);
    }

    #[test]
    fn test_extract_empty_style() {
        assert_eq!(extract_css_property("", "color"), None);
    }

    #[test]
    fn test_extract_with_extra_spaces() {
        let style = "  color :  blue  ; font-size: 16px ";
        assert_eq!(
            extract_css_property(style, "color"),
            Some("blue".to_string())
        );
    }

    #[test]
    fn test_extract_background_shorthand() {
        let style = "background: #ff0000";
        assert_eq!(
            extract_css_property(style, "background"),
            Some("#ff0000".to_string())
        );
    }
}
