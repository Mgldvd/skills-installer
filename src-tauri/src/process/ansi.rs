/// Strips ANSI escape sequences (CSI sequences like colors/cursor movement,
/// and OSC sequences like terminal-title-setting) from a line of process
/// output before it is ever forwarded to the frontend, so the GUI's
/// installation progress panel never has to deal with raw escape codes.
/// Hand-rolled as a small state machine rather than a regex dependency —
/// the grammar involved (ESC '[' ... final-byte, ESC ']' ... BEL/ST) is
/// simple enough that a dependency would be pure overhead here.
pub fn strip_ansi(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            output.push(c);
            continue;
        }

        match chars.peek() {
            Some('[') => {
                chars.next();
                for next in chars.by_ref() {
                    if ('@'..='~').contains(&next) {
                        break;
                    }
                }
            }
            Some(']') => {
                chars.next();
                loop {
                    match chars.next() {
                        None => break,
                        Some('\u{7}') => break,
                        Some('\u{1b}') => {
                            if chars.peek() == Some(&'\\') {
                                chars.next();
                            }
                            break;
                        }
                        Some(_) => continue,
                    }
                }
            }
            Some(_) => {
                chars.next();
            }
            None => {}
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_sgr_color_codes() {
        assert_eq!(
            strip_ansi("\u{1b}[32m✓ frontend-design\u{1b}[0m"),
            "✓ frontend-design"
        );
    }

    #[test]
    fn strips_cursor_movement() {
        assert_eq!(
            strip_ansi("\u{1b}[2K\u{1b}[1GInstalling..."),
            "Installing..."
        );
    }

    #[test]
    fn strips_osc_title_sequence_terminated_by_bel() {
        assert_eq!(
            strip_ansi("\u{1b}]0;window title\u{7}visible text"),
            "visible text"
        );
    }

    #[test]
    fn leaves_plain_text_untouched() {
        assert_eq!(
            strip_ansi("plain output, no escapes"),
            "plain output, no escapes"
        );
    }

    #[test]
    fn handles_multiple_sequences_in_one_line() {
        assert_eq!(
            strip_ansi("\u{1b}[1m\u{1b}[32mInstalling\u{1b}[0m \u{1b}[33mfrontend-design\u{1b}[0m"),
            "Installing frontend-design"
        );
    }
}
