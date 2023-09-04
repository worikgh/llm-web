#[cfg(test)]
mod tests {
    use crate::manipulate_css::clear_css;
    use crate::manipulate_css::get_css_rules;
    use crate::manipulate_css::CssRules;
    use wasm_bindgen_test::wasm_bindgen_test;
    #[wasm_bindgen_test]
    fn test_add_css() {
        let document = Document::new().expect("Making Document");
        clear_css(&document).expect("cleared CSS");
        let css_rules: CssRules = get_css_rules();
        assert!(css_rules.selector_rules.length() == 0);
    }

    #[wasm_bindgen_test]
    fn another_test_function() {
        // Test assertions and code go here
    }
}
