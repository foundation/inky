use super::El;

/// Build a class string from a base class and element classes.
pub fn build_classes(base: &str, el: &El) -> String {
    let mut classes = vec![base.to_string()];
    classes.extend(el.classes());
    classes.join(" ")
}
