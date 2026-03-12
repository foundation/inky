use scraper::ElementRef;

use super::helpers::inner_html;

pub fn make_outlook(element: &ElementRef) -> String {
    let inner = inner_html(element);
    format!("<!--[if mso]>\n{}\n<![endif]-->", inner)
}

pub fn make_not_outlook(element: &ElementRef) -> String {
    let inner = inner_html(element);
    format!("<!--[if !mso]><!-->\n{}\n<!--<![endif]-->", inner)
}
