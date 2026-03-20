use scraper::ElementRef;

pub fn make_center(element: &ElementRef) -> String {
    let mut html = String::new();
    html.push_str("<center data-parsed=\"\">");

    for child in element.children() {
        if let Some(child_el) = ElementRef::wrap(child) {
            let tag_name = child_el.value().name().to_string();
            let mut attrs: Vec<(String, String)> = child_el
                .value()
                .attrs()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();

            // Add align="center"
            let has_align = attrs.iter().any(|(k, _)| k == "align");
            if !has_align {
                attrs.push(("align".into(), "center".into()));
            }

            // Add float-center to class
            let class_idx = attrs.iter().position(|(k, _)| k == "class");
            match class_idx {
                Some(idx) => {
                    let existing = attrs[idx].1.clone();
                    if !existing.split_whitespace().any(|c| c == "float-center") {
                        attrs[idx].1 = format!("{} float-center", existing);
                    }
                }
                None => {
                    attrs.push(("class".into(), "float-center".into()));
                }
            }

            html.push('<');
            html.push_str(&tag_name);
            for (key, value) in &attrs {
                html.push(' ');
                html.push_str(key);
                html.push_str("=\"");
                html.push_str(value);
                html.push('"');
            }
            html.push('>');

            html.push_str(&child_el.inner_html());
            html.push_str(&format!("</{}>", tag_name));
        } else if let Some(text) = child.value().as_text() {
            html.push_str(&text.text);
        }
    }

    html.push_str("</center>");
    html
}
