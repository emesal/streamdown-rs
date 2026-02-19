//! Text wrapping and formatting utilities.
//!
//! This module provides ANSI-aware text wrapping that preserves escape codes
//! across line breaks, handles CJK characters correctly, and supports various
//! formatting options.

use streamdown_ansi::utils::{ansi_collapse, extract_ansi_codes, visible, visible_length};

/// Result of wrapping text.
#[derive(Debug, Clone)]
pub struct WrappedText {
    /// The wrapped lines
    pub lines: Vec<String>,
    /// Whether any lines were truncated
    pub truncated: bool,
}

impl WrappedText {
    /// Create empty wrapped text.
    pub fn empty() -> Self {
        Self {
            lines: Vec::new(),
            truncated: false,
        }
    }

    /// Check if there are no lines.
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Get the number of lines.
    pub fn len(&self) -> usize {
        self.lines.len()
    }
}

/// Split text into words while preserving ANSI codes.
///
/// This is smarter than a simple split - it keeps ANSI codes attached
/// to the words they modify and handles CSI (SGR) and OSC sequences correctly.
pub fn split_text(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        if ch == '\x1b' {
            // Peek at next char to determine sequence type
            let next = chars.get(i + 1).copied();
            match next {
                Some('[') => {
                    // CSI sequence: \x1b[ ... letter
                    current.push(ch);
                    i += 1;
                    while i < chars.len() {
                        let c = chars[i];
                        current.push(c);
                        i += 1;
                        if c.is_ascii_alphabetic() {
                            break;
                        }
                    }
                }
                Some(']') => {
                    // OSC sequence: \x1b] ... ST  where ST is \x1b\\ or BEL (\x07)
                    current.push(ch);
                    i += 1;
                    while i < chars.len() {
                        let c = chars[i];
                        current.push(c);
                        i += 1;
                        if c == '\x07' {
                            break; // BEL terminator
                        }
                        if c == '\x1b' && chars.get(i).copied() == Some('\\') {
                            current.push('\\');
                            i += 1;
                            break; // ST terminator
                        }
                    }
                }
                _ => {
                    // Unknown escape — pass through single char
                    current.push(ch);
                    i += 1;
                }
            }
            continue;
        }

        if ch.is_whitespace() {
            // Attach trailing whitespace to the preceding word so multi-space
            // sequences are preserved and text_wrap doesn't need to insert gaps.
            current.push(ch);
            i += 1;
            while i < chars.len() && chars[i].is_whitespace() {
                current.push(chars[i]);
                i += 1;
            }
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
        } else {
            current.push(ch);
            i += 1;
        }
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
}

/// Wrap text to fit within a given width.
///
/// This is ANSI-aware and will preserve formatting across line breaks.
///
/// # Arguments
/// * `text` - The text to wrap
/// * `width` - Maximum width in visible characters
/// * `indent` - Indentation for continuation lines
/// * `first_prefix` - Prefix for the first line
/// * `next_prefix` - Prefix for subsequent lines
/// * `force_truncate` - If true, truncate lines that are too long
/// * `preserve_format` - If true, don't reset formatting at end of lines
pub fn text_wrap(
    text: &str,
    width: usize,
    indent: usize,
    first_prefix: &str,
    next_prefix: &str,
    force_truncate: bool,
    preserve_format: bool,
) -> WrappedText {
    if width == 0 {
        return WrappedText::empty();
    }

    let first_prefix_width = visible_length(first_prefix);
    let next_prefix_width = visible_length(next_prefix);

    let words = split_text(text);
    if words.is_empty() {
        return WrappedText::empty();
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_style: Vec<String> = Vec::new();
    let mut truncated = false;
    let resetter = if preserve_format { "" } else { "\x1b[0m" };

    for word in words.iter().chain(std::iter::once(&String::new())) {
        // Extract ANSI codes from the word
        let codes = extract_ansi_codes(word);

        // Check if word starts with an ANSI code
        if !codes.is_empty() && word.starts_with(&codes[0]) {
            current_style.push(codes[0].clone());
        }

        let word_visible_len = visible_length(word);
        let line_visible_len = visible_length(&current_line);

        // Space is now part of each word's trailing chars (from split_text),
        // so no extra gap is inserted here.
        let space_needed = 0;

        let effective_width = if lines.is_empty() {
            width.saturating_sub(first_prefix_width)
        } else {
            width.saturating_sub(next_prefix_width)
        };

        if word_visible_len > 0
            && line_visible_len + word_visible_len + space_needed <= effective_width
        {
            // Word fits (trailing space is part of the word from split_text)
            current_line.push_str(word);
        } else if word_visible_len > 0 {
            // Word doesn't fit, finalize current line
            if !current_line.is_empty() {
                let prefix = if lines.is_empty() {
                    first_prefix
                } else {
                    next_prefix
                };
                let mut line_content = format!("{}{}", prefix, current_line);

                // Force truncate if needed
                if force_truncate && visible_length(&line_content) > width {
                    // Truncate to width-1 columns and add an ellipsis.
                    // Using visible_length (display columns) avoids the
                    // byte-vs-width mismatch that caused an infinite loop
                    // when multi-byte chars (including '…' itself) were present.
                    line_content = truncate_to_visible(&line_content, width.saturating_sub(1));
                    line_content.push('…');
                    truncated = true;
                }

                // Add resetter and padding
                let padding = width.saturating_sub(visible_length(&line_content));
                line_content.push_str(resetter);
                line_content.push_str(&" ".repeat(padding));

                if !visible(&line_content).trim().is_empty() {
                    lines.push(line_content);
                }
            }

            // Start new line with current word
            let indent_str = " ".repeat(indent);
            let style_str: String = current_style.join("");
            current_line = format!("{}{}{}", indent_str, style_str, word);
        }

        // Update style tracking
        for code in codes.iter().skip(
            if word.starts_with(&codes.first().cloned().unwrap_or_default()) {
                1
            } else {
                0
            },
        ) {
            current_style.push(code.clone());
        }
        current_style = ansi_collapse(&current_style, "");
    }

    // Don't forget the last line
    if !current_line.is_empty() && !visible(&current_line).trim().is_empty() {
        let prefix = if lines.is_empty() {
            first_prefix
        } else {
            next_prefix
        };
        let mut line_content = format!("{}{}", prefix, current_line);

        if force_truncate && visible_length(&line_content) > width {
            line_content = truncate_to_visible(&line_content, width.saturating_sub(1));
            line_content.push('…');
            truncated = true;
        }

        line_content.push_str(resetter);
        lines.push(line_content);
    }

    WrappedText { lines, truncated }
}

/// Truncate a string (with ANSI codes) to a visible length.
fn truncate_to_visible(text: &str, max_visible: usize) -> String {
    let mut result = String::new();
    let mut visible_count = 0;
    let mut in_escape = false;

    for ch in text.chars() {
        if in_escape {
            result.push(ch);
            if ch == 'm' {
                in_escape = false;
            }
            continue;
        }

        if ch == '\x1b' {
            in_escape = true;
            result.push(ch);
            continue;
        }

        if visible_count >= max_visible {
            break;
        }

        result.push(ch);
        visible_count += unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);
    }

    result
}

/// Simple text wrap without ANSI awareness (for plain text).
pub fn simple_wrap(text: &str, width: usize) -> Vec<String> {
    if width == 0 || text.is_empty() {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        let word_len = unicode_width::UnicodeWidthStr::width(word);
        let current_len = unicode_width::UnicodeWidthStr::width(current.as_str());

        if current.is_empty() {
            current = word.to_string();
        } else if current_len + 1 + word_len <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(std::mem::take(&mut current));
            current = word.to_string();
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_text() {
        let words = split_text("hello world");
        // Trailing space is attached to the preceding word
        assert_eq!(words, vec!["hello ", "world"]);
    }

    #[test]
    fn test_split_text_with_ansi() {
        let text = "\x1b[1mhello\x1b[0m world";
        let words = split_text(text);
        assert_eq!(words.len(), 2);
        assert!(words[0].contains("\x1b[1m"));
    }

    #[test]
    fn test_simple_wrap() {
        let lines = simple_wrap("hello world foo bar", 10);
        assert!(lines.len() >= 2);
    }

    #[test]
    fn test_text_wrap_basic() {
        let result = text_wrap("hello world", 20, 0, "", "", false, false);
        assert_eq!(result.lines.len(), 1);
    }

    #[test]
    fn test_text_wrap_multiline() {
        let result = text_wrap("hello world foo bar baz", 10, 0, "", "", false, false);
        assert!(result.lines.len() > 1);
    }

    #[test]
    fn test_text_wrap_with_prefix() {
        let result = text_wrap("hello world", 20, 0, "> ", "  ", false, false);
        assert!(!result.lines.is_empty());
        assert!(result.lines[0].starts_with("> "));
    }

    #[test]
    fn test_truncate_to_visible() {
        let text = "hello world";
        let truncated = truncate_to_visible(text, 5);
        assert_eq!(truncated, "hello");
    }

    #[test]
    fn test_truncate_with_ansi() {
        let text = "\x1b[1mhello\x1b[0m world";
        let truncated = truncate_to_visible(text, 5);
        assert!(truncated.contains("\x1b["));
    }

    #[test]
    fn test_truncate_to_visible_box_drawing() {
        // ═ is 3 bytes, 1 display column. Byte-based slicing would corrupt.
        let text = "═══════════";
        assert_eq!(text.len(), 33);
        assert_eq!(text.chars().count(), 11);

        let truncated = truncate_to_visible(text, 5);

        assert_eq!(truncated.chars().count(), 5);
        assert_eq!(truncated, "═════");
    }

    #[test]
    fn test_truncate_to_visible_emojis() {
        // 🎉 = 4 bytes, 2 display columns.
        let text = "🎉🎉🎉🎉🎉";
        assert_eq!(text.len(), 20);
        assert_eq!(text.chars().count(), 5);

        let truncated = truncate_to_visible(text, 4);

        assert_eq!(truncated.chars().count(), 2);
        assert_eq!(truncated, "🎉🎉");
    }

    #[test]
    fn test_truncate_to_visible_zwj_emojis() {
        // 👨‍💻 = 11 bytes, 3 code points. Should not split ZWJ sequence.
        let text = format!("{}{}{}", "👨‍💻", "👨‍💻", "👨‍💻");
        assert_eq!(text.chars().count(), 9);

        let truncated = truncate_to_visible(&text, 4);

        let _ = truncated.chars().count(); // Should not panic
    }

    #[test]
    fn test_simple_wrap_cjk() {
        // CJK: 3 bytes, 2 display columns per char.
        let text = "你好 世界 你好 世界";

        let lines = simple_wrap(text, 10);

        assert_eq!(lines.len(), 2);
        for line in &lines {
            let _ = line.chars().count();
        }
    }

    #[test]
    fn test_simple_wrap_emojis() {
        // 🎉 = 4 bytes, 2 display columns.
        let text = "🎉🎉🎉🎉 🎉🎉🎉🎉";

        let lines = simple_wrap(text, 10);

        assert!(!lines.is_empty());
    }

    #[test]
    fn test_text_wrap_mixed_multibyte() {
        let text = "Hello 你好 ═══ 🎉 world";

        let result = text_wrap(text, 15, 0, "", "", false, false);

        assert!(!result.lines.is_empty());
    }

    #[test]
    fn test_split_text_emoji_integrity() {
        let text = "hello 🎉 world 🌟 test";

        let words = split_text(text);

        assert_eq!(words.len(), 5);
        // Trailing space is attached to the preceding word
        assert!(words[1].starts_with("🎉"));
        assert!(words[3].starts_with("🌟"));
    }

    #[test]
    fn test_split_text_zwj_emoji() {
        // ZWJ emoji 👨‍👩‍👧‍👦 should stay together (7 code points).
        let text = "Family: 👨‍👩‍👧‍👦 done";

        let words = split_text(text);

        assert_eq!(words.len(), 3);
        // Trailing space is attached to the preceding word; emoji itself is 7 code points
        assert!(words[1].starts_with('👨'));
    }

    #[test]
    fn test_force_truncate_terminates_with_multibyte_ellipsis() {
        // Regression: the old while-loop used visible_part.len() (byte count)
        // for target_len. Once '…' (3 bytes, 1 display col) was appended,
        // target_len was too large to shorten the string, causing an infinite loop.
        // The fix uses width directly so truncation always terminates.
        let long_word = "supercalifragilistic"; // 20 chars, all ASCII
        let result = text_wrap(long_word, 8, 0, "", "", true, false);
        assert!(!result.lines.is_empty());
        assert!(result.truncated);
        // Result must fit within width (8 columns)
        for line in &result.lines {
            assert!(
                visible_length(line) <= 8,
                "line too wide: {:?} ({} cols)",
                line,
                visible_length(line)
            );
        }
    }

    #[test]
    fn test_force_truncate_terminates_with_multibyte_content() {
        // '═' is 3 bytes, 1 display column — same class of bug.
        let wide_text = "═══════════════════"; // 19 box-drawing chars
        let result = text_wrap(wide_text, 8, 0, "", "", true, false);
        assert!(!result.lines.is_empty());
        for line in &result.lines {
            assert!(
                visible_length(line) <= 8,
                "line too wide: {:?} ({} cols)",
                line,
                visible_length(line)
            );
        }
    }

    #[test]
    fn test_split_text_preserves_osc_hyperlink() {
        // OSC 8 hyperlink: \x1b]8;;url\x1b\\ + text + \x1b]8;;\x1b\\
        let link = "\x1b]8;;https://example.com\x1b\\Click here\x1b]8;;\x1b\\";
        let words = split_text(link);
        // OSC sequences must not be split mid-sequence; each word may contain ANSI
        let joined = words.join(" ");
        // The OSC sequences must be preserved intact (not split by whitespace scanning)
        assert!(
            joined.contains("\x1b]8;;https://example.com\x1b\\"),
            "opening OSC sequence must be preserved"
        );
        assert!(
            joined.contains("\x1b]8;;\x1b\\"),
            "closing OSC sequence must be preserved"
        );
    }

    #[test]
    fn test_split_text_preserves_multiple_spaces() {
        // Two spaces between words should result in two spaces when rejoined
        let text = "hello  world"; // two spaces
        let words = split_text(text);
        // words[0] should end with a space so the rejoined string has two spaces
        let rejoined = words.join("");
        assert!(
            rejoined.contains("  "),
            "double space should be preserved, got: {:?}",
            rejoined
        );
    }

    #[test]
    fn test_text_wrap_prefix_does_not_overflow() {
        let margin = "│ ";
        let width = 20;
        let text = "hello world foo bar baz qux";
        let result = text_wrap(text, width, 0, margin, margin, false, false);
        for line in &result.lines {
            let total = visible_length(line);
            assert!(
                total <= width,
                "line is {} cols wide, exceeds width {}: {:?}",
                total,
                width,
                line
            );
        }
    }
}
