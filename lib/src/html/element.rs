use scraper::CaseSensitivity;

pub fn has_classes(element: ElementRef, class_names: &[&str]) -> bool {
    class_names
        .iter()
        .all(|cn| element.value().has_class(cn, CaseSensitivity.CaseSensitive))
}
