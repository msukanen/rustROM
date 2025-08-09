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
    let mut last_index = 0;

    // This new loop manually finds '<' and '>' pairs, which is more robust
    // than using `split`.
    while let Some(tag_start) = input[last_index..].find('<') {
        let absolute_tag_start = last_index + tag_start;

        // 1. Append the plain text found before this tag.
        let text_before = &input[last_index..absolute_tag_start];
        if let Some(current_style) = style_stack.last() {
            output.push_str(&current_style.paint(text_before).to_string());
        }

        // 2. Find the closing '>' for this tag.
        if let Some(tag_end) = input[absolute_tag_start..].find('>') {
            let absolute_tag_end = absolute_tag_start + tag_end;
            let tag_content = &input[absolute_tag_start + 1..absolute_tag_end];
            let tag_parts: Vec<&str> = tag_content.split_whitespace().collect();

            let mut tag_processed = false;
            if let Some(tag_name) = tag_parts.first() {
                let is_closing = tag_name.starts_with('/');
                let actual_tag = if is_closing { &tag_name[1..] } else { *tag_name };

                if actual_tag == "c" || actual_tag == "bg" {
                    tag_processed = true;
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

            // If the tag was not valid (e.g., "<not a tag>"), treat it as literal text.
            if !tag_processed {
                 if let Some(current_style) = style_stack.last() {
                    let literal_text = &input[absolute_tag_start..=absolute_tag_end];
                    output.push_str(&current_style.paint(literal_text).to_string());
                }
            }

            last_index = absolute_tag_end + 1;
        } else {
            // Malformed tag (e.g., "<..."), treat the rest of the string as literal text.
            break;
        }
    }

    // 3. Append any remaining text after the last tag.
    if last_index < input.len() {
        let final_text = &input[last_index..];
        if let Some(current_style) = style_stack.last() {
            output.push_str(&current_style.paint(final_text).to_string());
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
