use super::El;

pub fn make_outlook(el: &El) -> String {
    let inner = el.inner();
    format!("<!--[if mso]>\n{}\n<![endif]-->", inner)
}

pub fn make_not_outlook(el: &El) -> String {
    let inner = el.inner();
    format!("<!--[if !mso]><!-->\n{}\n<!--<![endif]-->", inner)
}
