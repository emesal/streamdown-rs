//! ANSI text processing utilities.
//!
//! This module provides functions for working with ANSI-formatted text,
//! including visible length calculation, escape code extraction, and
//! code deduplication.

use regex::Regex;
use std::sync::LazyLock;
use unicode_width::UnicodeWidthStr;

/// Regex pattern for basic ANSI escape sequences (SGR codes).
pub const ESCAPE: &str = r"\x1b\[[0-9;]*[mK]";

/// Regex pattern for all ANSI escape sequences including OSC.
/// Matches:
/// - CSI sequences: \x1b[...letter
/// - OSC sequences: \x1b]...;\
/// - Simple escapes: \x1b)
pub const ANSIESCAPE: &str = r"\x1b(?:\[[0-9;?]*[a-zA-Z]|\][0-9]*;;.*?\\|\))";

/// Compiled regex for ESCAPE pattern.
static ESCAPE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(ESCAPE).unwrap());

/// Compiled regex for ANSIESCAPE pattern.
static ANSIESCAPE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(ANSIESCAPE).unwrap());

/// Compiled regex for splitting text into ANSI/non-ANSI segments.
static SPLIT_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\x1b[^m]*m|[^\x1b]+)").unwrap());

/// Remove all ANSI escape sequences from text.
///
/// Returns only the visible text content.
///
/// # Arguments
///
/// * `text` - Text potentially containing ANSI escape sequences
///
/// # Example
///
/// ```
/// use streamdown_ansi::utils::visible;
/// let text = "\x1b[1mBold\x1b[0m text";
/// assert_eq!(visible(text), "Bold text");
/// ```
pub fn visible(text: &str) -> String {
    ANSIESCAPE_RE.replace_all(text, "").to_string()
}

/// Calculate the visible display width of text.
///
/// This removes all ANSI escape sequences and calculates the
/// width using Unicode character widths (handling CJK characters, etc.).
///
/// # Arguments
///
/// * `text` - Text potentially containing ANSI escape sequences
///
/// # Returns
///
/// The display width in terminal columns.
///
/// # Example
///
/// ```
/// use streamdown_ansi::utils::visible_length;
/// let text = "\x1b[1mHello\x1b[0m";
/// assert_eq!(visible_length(text), 5);
///
/// // CJK characters are typically double-width
/// let cjk = "你好";
/// assert_eq!(visible_length(cjk), 4);
/// ```
pub fn visible_length(text: &str) -> usize {
    visible(text).width()
}

/// Extract all ANSI escape codes from text.
///
/// Returns a vector of all ANSI escape sequences found in the text.
///
/// # Arguments
///
/// * `text` - Text potentially containing ANSI escape sequences
///
/// # Example
///
/// ```
/// use streamdown_ansi::utils::extract_ansi_codes;
/// let text = "\x1b[1mBold\x1b[0m";
/// let codes = extract_ansi_codes(text);
/// assert_eq!(codes, vec!["\x1b[1m", "\x1b[0m"]);
/// ```
pub fn extract_ansi_codes(text: &str) -> Vec<String> {
    ESCAPE_RE
        .find_iter(text)
        .map(|m| m.as_str().to_string())
        .collect()
}

/// Remove specific ANSI codes from text.
///
/// # Arguments
///
/// * `line` - The text to process
/// * `code_list` - List of ANSI codes to remove
///
/// # Example
///
/// ```
/// use streamdown_ansi::utils::remove_ansi;
/// let text = "\x1b[1mBold\x1b[0m";
/// let codes = vec!["\x1b[1m".to_string()];
/// let result = remove_ansi(text, &codes);
/// assert_eq!(result, "Bold\x1b[0m");
/// ```
pub fn remove_ansi(line: &str, code_list: &[String]) -> String {
    let mut result = line.to_string();
    for code in code_list {
        result = result.replace(code, "");
    }
    result
}

/// Split text into ANSI escape sequences and regular text segments.
///
/// # Arguments
///
/// * `line` - Text to split
///
/// # Returns
///
/// A vector of segments, alternating between ANSI codes and text.
///
/// # Example
///
/// ```
/// use streamdown_ansi::utils::split_up;
/// let text = "\x1b[1mBold\x1b[0m text";
/// let parts = split_up(text);
/// assert_eq!(parts, vec!["\x1b[1m", "Bold", "\x1b[0m", " text"]);
/// ```
pub fn split_up(line: &str) -> Vec<String> {
    SPLIT_RE
        .find_iter(line)
        .map(|m| m.as_str().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Check if a string is an ANSI escape code.
///
/// # Arguments
///
/// * `s` - String to check
///
/// # Example
///
/// ```
/// use streamdown_ansi::utils::is_ansi_code;
/// assert!(is_ansi_code("\x1b[1m"));
/// assert!(!is_ansi_code("hello"));
/// ```
pub fn is_ansi_code(s: &str) -> bool {
    s.starts_with("\x1b[")
}

/// Parse SGR (Select Graphic Rendition) parameters from an ANSI code.
///
/// # Arguments
///
/// * `code` - An ANSI SGR code like "\x1b[1;4;38;2;255;0;0m"
///
/// # Returns
///
/// Vector of parameter values.
///
/// # Example
///
/// ```
/// use streamdown_ansi::utils::parse_sgr_params;
/// let params = parse_sgr_params("\x1b[1;4m");
/// assert_eq!(params, vec![1, 4]);
/// ```
pub fn parse_sgr_params(code: &str) -> Vec<u32> {
    let inner = code
        .trim_start_matches("\x1b[")
        .trim_end_matches('m')
        .trim_end_matches('K');

    if inner.is_empty() {
        return vec![0]; // Reset
    }

    inner.split(';').filter_map(|s| s.parse().ok()).collect()
}

/// Collapse redundant ANSI codes in a sequence.
///
/// Deduplicates and preserves ANSI escape sequences, including truecolor,
/// 256-color, legacy 16-color, and text attributes. Unknown codes pass through.
///
/// # Arguments
///
/// * `code_list` - List of ANSI codes applied in order
/// * `_inp` - Unused (reserved for context-aware optimizations)
///
/// # Returns
///
/// Optimized list of ANSI codes.
///
/// # Example
///
/// ```
/// use streamdown_ansi::utils::ansi_collapse;
/// let codes = vec![
///     "\x1b[1m".to_string(),  // bold on
///     "\x1b[1m".to_string(),  // bold on (redundant)
///     "\x1b[22m".to_string(), // bold off
///     "\x1b[1m".to_string(),  // bold on again
/// ];
/// let collapsed = ansi_collapse(&codes, "");
/// // Should have fewer codes after optimization
/// ```
pub fn ansi_collapse(code_list: &[String], _inp: &str) -> Vec<String> {
    if code_list.is_empty() {
        return Vec::new();
    }

    // Find the last reset code index — codes before it are moot.
    let last_reset = code_list.iter().rposition(|c| {
        let params = parse_sgr_params(c);
        params == vec![0]
    });

    // Only keep codes after the last reset (or all codes if no reset).
    let effective: &[String] = if let Some(reset_idx) = last_reset {
        &code_list[reset_idx + 1..]
    } else {
        code_list
    };

    // Deduplicate: for codes we understand, keep only the last one of each category.
    // For anything else, pass through.
    let mut bold: Option<String> = None;
    let mut dim: Option<String> = None;
    let mut italic: Option<String> = None;
    let mut underline: Option<String> = None;
    let mut strikeout: Option<String> = None;
    let mut fg: Option<String> = None;
    let mut bg: Option<String> = None;
    let mut passthrough: Vec<String> = Vec::new();

    for code in effective {
        let params = parse_sgr_params(code);
        if params.is_empty() {
            continue;
        }
        match params[0] {
            0 => { /* reset — already handled above */ }
            1 => bold = Some(code.clone()),
            2 => dim = Some(code.clone()),
            3 => italic = Some(code.clone()),
            4 => underline = Some(code.clone()),
            9 => strikeout = Some(code.clone()),
            22 => {
                bold = None;
                dim = None;
            }
            23 => italic = None,
            24 => underline = None,
            29 => strikeout = None,
            38 => fg = Some(code.clone()), // both 38;2 (truecolor) and 38;5 (256-color)
            39 => fg = None,
            48 => bg = Some(code.clone()), // both 48;2 (truecolor) and 48;5 (256-color)
            49 => bg = None,
            // Legacy 16-color foreground (30-37, 90-97)
            30..=37 | 90..=97 => fg = Some(code.clone()),
            // Legacy 16-color background (40-47, 100-107)
            40..=47 | 100..=107 => bg = Some(code.clone()),
            // Everything else: blink, reverse, conceal, etc. — pass through
            _ => passthrough.push(code.clone()),
        }
    }

    let mut result = Vec::new();
    // Collect known text attributes into a combined SGR
    let mut attrs: Vec<&str> = Vec::new();
    if bold.is_some() {
        attrs.push("1");
    }
    if dim.is_some() {
        attrs.push("2");
    }
    if italic.is_some() {
        attrs.push("3");
    }
    if underline.is_some() {
        attrs.push("4");
    }
    if strikeout.is_some() {
        attrs.push("9");
    }
    if !attrs.is_empty() {
        result.push(format!("\x1b[{}m", attrs.join(";")));
    }
    if let Some(c) = fg {
        result.push(c);
    }
    if let Some(c) = bg {
        result.push(c);
    }
    result.extend(passthrough);
    result
}

/// Wrap text to a specified width, preserving ANSI codes.
///
/// # Arguments
///
/// * `text` - Text to wrap (may contain ANSI codes)
/// * `width` - Maximum line width in columns
///
/// # Returns
///
/// Vector of wrapped lines.
///
/// # Example
///
/// ```
/// use streamdown_ansi::utils::wrap_ansi;
/// let text = "Hello world, this is a long line";
/// let lines = wrap_ansi(text, 15);
/// assert!(lines.len() > 1);
/// ```
pub fn wrap_ansi(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;
    let mut active_codes: Vec<String> = Vec::new();

    for segment in split_up(text) {
        if is_ansi_code(&segment) {
            // Track active formatting
            let params = parse_sgr_params(&segment);
            if params.contains(&0) {
                active_codes.clear();
            } else {
                active_codes.push(segment.clone());
            }
            current_line.push_str(&segment);
        } else {
            // Regular text - may need wrapping
            for word in segment.split_inclusive(' ') {
                let word_width = word.width();

                if current_width + word_width > width && current_width > 0 {
                    // Need to wrap
                    // Reset codes at end of line
                    if !active_codes.is_empty() {
                        current_line.push_str(crate::codes::RESET);
                    }
                    lines.push(current_line);

                    // Start new line with active codes
                    current_line = active_codes.join("");
                    current_width = 0;
                }

                current_line.push_str(word);
                current_width += word_width;
            }
        }
    }

    if !current_line.is_empty() || lines.is_empty() {
        lines.push(current_line);
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visible() {
        assert_eq!(visible("\x1b[1mBold\x1b[0m"), "Bold");
        assert_eq!(visible("No codes"), "No codes");
        assert_eq!(visible("\x1b[38;2;255;0;0mRed\x1b[0m"), "Red");
    }

    #[test]
    fn test_visible_length() {
        assert_eq!(visible_length("\x1b[1mHello\x1b[0m"), 5);
        assert_eq!(visible_length("Hello"), 5);
        // Test empty
        assert_eq!(visible_length(""), 0);
    }

    #[test]
    fn test_extract_ansi_codes() {
        let codes = extract_ansi_codes("\x1b[1mBold\x1b[0m");
        assert_eq!(codes.len(), 2);
        assert_eq!(codes[0], "\x1b[1m");
        assert_eq!(codes[1], "\x1b[0m");
    }

    #[test]
    fn test_remove_ansi() {
        let text = "\x1b[1mBold\x1b[0m";
        let result = remove_ansi(text, &["\x1b[1m".to_string()]);
        assert_eq!(result, "Bold\x1b[0m");
    }

    #[test]
    fn test_split_up() {
        let parts = split_up("\x1b[1mBold\x1b[0m text");
        assert!(parts.contains(&"\x1b[1m".to_string()));
        assert!(parts.contains(&"Bold".to_string()));
    }

    #[test]
    fn test_is_ansi_code() {
        assert!(is_ansi_code("\x1b[1m"));
        assert!(is_ansi_code("\x1b[0m"));
        assert!(!is_ansi_code("hello"));
        assert!(!is_ansi_code(""));
    }

    #[test]
    fn test_parse_sgr_params() {
        assert_eq!(parse_sgr_params("\x1b[1m"), vec![1]);
        assert_eq!(parse_sgr_params("\x1b[1;4m"), vec![1, 4]);
        assert_eq!(
            parse_sgr_params("\x1b[38;2;255;0;0m"),
            vec![38, 2, 255, 0, 0]
        );
    }

    #[test]
    fn test_ansi_collapse_removes_duplicates() {
        let codes = vec!["\x1b[1m".to_string(), "\x1b[1m".to_string()];
        let collapsed = ansi_collapse(&codes, "");
        assert_eq!(collapsed.len(), 1);
        assert!(collapsed[0].contains("1")); // Bold
    }

    #[test]
    fn test_wrap_ansi_basic() {
        let text = "Hello world";
        let lines = wrap_ansi(text, 6);
        assert!(lines.len() >= 2);
    }

    #[test]
    fn test_ansi_collapse_preserves_256_color() {
        // 256-color foreground code
        let codes = vec!["\x1b[38;5;196m".to_string()]; // bright red
        let collapsed = ansi_collapse(&codes, "");
        assert!(
            collapsed.iter().any(|c| c.contains("38;5;196")),
            "256-color fg code should be preserved, got: {:?}",
            collapsed
        );
    }

    #[test]
    fn test_ansi_collapse_preserves_legacy_color() {
        let codes = vec!["\x1b[31m".to_string()]; // legacy red fg
        let collapsed = ansi_collapse(&codes, "");
        assert!(
            collapsed.iter().any(|c| c == "\x1b[31m"),
            "legacy color should be preserved, got: {:?}",
            collapsed
        );
    }
}
