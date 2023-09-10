/// Filter HTML text so it is displayed properly.
pub fn filter_html(input: &str) -> String {
    let mut result = String::new();
    for c in input.chars() {
        match c {
            '\t' => result.push_str("&nbsp;&nbsp;&nbsp;&nbsp;"),
            ' ' => result.push_str("&nbsp;"),
            '\n' => result.push_str("<br/>"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '&' => result.push_str("&amp;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#39;"),
            _ => result.push(c),
        }
    }
    result
}
