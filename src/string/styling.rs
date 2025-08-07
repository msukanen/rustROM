/*
This example demonstrates a simple, stack-based parser for a custom
color markup language, using the `ansi_term` library.

--- How to Run ---
1. Add `ansi_term` to your Cargo.toml dependencies:
   `ansi_term = "0.12"`
2. Run `cargo run`. It will parse the example string and print the
   colored output to your terminal.
*/

use ansi_term::{Colour, Style};

/// Parses a color name string into an ansi_term::Colour.
fn parse_color(name: &str) -> Option<Colour> {
    match name.to_lowercase().as_str() {
        "black" => Some(Colour::Black),
        "red" => Some(Colour::Red),
        "green" => Some(Colour::Green),
        "yellow" => Some(Colour::Yellow),
        "blue" => Some(Colour::Blue),
        "purple" => Some(Colour::Purple),
        "cyan" => Some(Colour::Cyan),
        "white" => Some(Colour::White),
        // You can add more colors or fixed RGB values here
        _ => None,
    }
}

/// Formats a string with custom color tags into an ANSI-colored string.
pub fn format_color(input: &str) -> String {
    let mut output = String::new();
    // The style stack. The style at the top is the current style.
    // We start with a default, plain style.
    let mut style_stack = vec![Style::new()];

    // We split the input by the '<' character to separate text from tags.
    for part in input.split('<') {
        // If a part doesn't contain '>', it's just plain text that came
        // before the first tag.
        if !part.contains('>') {
            if let Some(current_style) = style_stack.last() {
                output.push_str(&current_style.paint(part).to_string());
            }
            continue;
        }

        // If it does contain '>', we split it into the tag and the text that follows.
        if let Some((tag_content, text_after_tag)) = part.split_once('>') {
            let tag_parts: Vec<&str> = tag_content.split_whitespace().collect();

            if let Some(tag_name) = tag_parts.first() {
                // --- Handle Closing Tags ---
                if tag_name.starts_with('/') {
                    // Only pop if we have more than the base style on the stack.
                    if style_stack.len() > 1 {
                        style_stack.pop();
                    }
                }
                // --- Handle Opening Tags ---
                else {
                    // Get the current style to build upon it.
                    let mut new_style = style_stack.last().cloned().unwrap_or_else(Style::new);

                    if let Some(color_name) = tag_parts.get(1) {
                        if let Some(color) = parse_color(color_name) {
                            match *tag_name {
                                "c" => new_style = new_style.fg(color),
                                "bg" => new_style = new_style.on(color),
                                _ => {} // Ignore unknown tags
                            }
                        }
                    }
                    // Push the new, modified style onto the stack.
                    style_stack.push(new_style);
                }
            }
            
            // Paint the text that followed this tag with the new current style.
            if let Some(current_style) = style_stack.last() {
                output.push_str(&current_style.paint(text_after_tag).to_string());
            }
        }
    }

    output
}

#[cfg(test)]
mod ansi_tests {
    use super::*;
    #[test]
    fn tagging() {
        let input_string = "This is <c yellow>Yellow text <bg cyan>on cyan bg</bg> which continues as yellow</c>, until it doesn't.";
        
        println!("--- Input String ---");
        println!("{}", input_string);
        
        println!("\n--- Formatted Output ---");
        let formatted = format_color(input_string);
        println!("{}", formatted);
        panic!("flush");
    }
}
