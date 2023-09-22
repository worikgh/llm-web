/// Make a button that says "cancel" with an image
pub struct CancelButton;

impl CancelButton {
    pub fn inner_html(&self) -> String {
        r#"
<svg width="100" height="100" class="svg_cancel_button">
  <circle cx="50" cy="50" r="40" fill="red" />
  <line x1="21.72" y1="21.72" x2="78.28" y2="78.28" stroke="black" stroke-width="7" />
  <line x1="21.72" y1="78.28" x2="78.28" y2="21.72" stroke="black" stroke-width="7" />
</svg>
"#
        .to_string()
    }
}
