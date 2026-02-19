//! Custom style example: Render with custom colors.
//!
//! Run with: `cargo run --example custom_style`

use streamdown_parser::Parser;
use streamdown_render::{RenderStyle, Renderer};

fn main() {
    let markdown = r#"# Custom Styled Output

This example shows how to customize the colors and styling.

## Code Block

```python
def greet(name):
    return f"Hello, {name}!"

print(greet("World"))
```

## Features

- **Bold text** stands out
- *Italic text* is emphasized
- `inline code` is highlighted

> A quote with custom colors!
"#;

    // Create a custom style with different colors
    let custom_style = RenderStyle {
        h1: "#00ff80".to_string(),
        h2: "#00ff80".to_string(),
        h3: "#00ff80".to_string(),
        h4: "#00ff80".to_string(),
        h5: "#00ff80".to_string(),
        h6: "#808080".to_string(),
        code_bg: "#14143c".to_string(),
        code_label: "#00ffff".to_string(),
        bullet: "#ffff00".to_string(),
        table_header_bg: "#14143c".to_string(),
        table_border: "#808080".to_string(),
        blockquote_border: "#808080".to_string(),
        think_border: "#808080".to_string(),
        hr: "#808080".to_string(),
        link_url: "#b4a0dc".to_string(),
        image_marker: "#00ffff".to_string(),
        footnote: "#00ffff".to_string(),
    };

    // Create output buffer
    let mut output = Vec::new();

    // Create parser
    let mut parser = Parser::new();

    // Get terminal width
    let width = streamdown_render::terminal_width();

    {
        // Create renderer with custom style
        let mut renderer = Renderer::with_style(&mut output, width, custom_style);

        // Parse and render
        for line in markdown.lines() {
            let events = parser.parse_line(line);
            for event in events {
                renderer.render_event(&event).unwrap();
            }
        }
    }

    // Print the styled output
    print!("{}", String::from_utf8(output).unwrap());
}
