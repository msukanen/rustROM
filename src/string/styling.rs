use std::fmt::Display;

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
pub fn format_color<S: Display>(input: S) -> String
{
    let input = input.to_string();
    let mut output = String::new();
    let mut style_stack = vec![Style::new()];

    let mut parts = input.split('<');

    // --- THIS IS THE FIX ---
    // The first part of the split can never be a tag, it's always just text.
    // We handle it separately to avoid misinterpreting any '>' characters it might contain.
    if let Some(first_part) = parts.next() {
        if let Some(current_style) = style_stack.last() {
            output.push_str(&current_style.paint(first_part).to_string());
        }
    }

    // Now, loop through the rest of the parts. Each of these was preceded by a '<'.
    for part in parts {
        // If a part doesn't contain '>', it's a malformed tag like "<abc"
        // with no closing ">". We'll treat it as literal text.
        if !part.contains('>') {
            if let Some(current_style) = style_stack.last() {
                let original_text = format!("<{}", part);
                output.push_str(&current_style.paint(original_text).to_string());
            }
            continue;
        }

        if let Some((tag_content, text_after_tag)) = part.split_once('>') {
            let tag_parts: Vec<&str> = tag_content.split_whitespace().collect();

            let mut is_valid_tag = false;
            if let Some(tag_name) = tag_parts.first() {
                let is_closing = tag_name.starts_with('/');
                let actual_tag = if is_closing { &tag_name[1..] } else { *tag_name };

                if actual_tag == "c" || actual_tag == "bg" {
                    is_valid_tag = true;
                    if is_closing {
                        if style_stack.len() > 1 {
                            style_stack.pop();
                        }
                    } else {
                        let mut new_style = style_stack.last().cloned().unwrap_or_else(Style::new);
                        if let Some(color_name) = tag_parts.get(1) {
                            if let Some(color) = parse_color(color_name) {
                                match *tag_name {
                                    "c" => new_style = new_style.fg(color),
                                    "bg" => new_style = new_style.on(color),
                                    _ => {}
                                }
                            }
                        }
                        style_stack.push(new_style);
                    }
                }
            }

            if is_valid_tag {
                // If it was a valid tag, just paint the text that followed.
                if let Some(current_style) = style_stack.last() {
                    output.push_str(&current_style.paint(text_after_tag).to_string());
                }
            } else {
                // If it was NOT a valid tag, it must be plain text.
                // Reconstruct the original string, including the '<' from the split.
                let original_text = format!("<{}", part);
                if let Some(current_style) = style_stack.last() {
                    output.push_str(&current_style.paint(original_text).to_string());
                }
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
