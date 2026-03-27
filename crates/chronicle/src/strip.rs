//! Content stripping utilities for FTS indexing.
//!
//! Removes system reminders, XML tags, and tool-call markup to produce
//! clean text suitable for full-text search.

use regex::Regex;
use std::sync::LazyLock;

static SYSTEM_REMINDER_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)<system-reminder>.*?</system-reminder>").expect("valid regex")
});

static XML_TAG_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]+>").expect("valid regex"));

static MULTI_WHITESPACE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s{3,}").expect("valid regex"));

/// Strip markup from content for FTS indexing.
///
/// Removes system reminders, XML tags, and collapses excessive whitespace.
/// This is best-effort — some markup may survive.
pub fn strip_markup(content: &str) -> String {
    let without_reminders = SYSTEM_REMINDER_RE.replace_all(content, "");
    let without_tags = XML_TAG_RE.replace_all(&without_reminders, "");
    let collapsed = MULTI_WHITESPACE_RE.replace_all(&without_tags, "\n\n");
    collapsed.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_system_reminders() {
        let input = "hello <system-reminder>secret stuff</system-reminder> world";
        assert_eq!(strip_markup(input), "hello  world");
    }

    #[test]
    fn strips_xml_tags() {
        let input = "some <tool_call>data</tool_call> here";
        assert_eq!(strip_markup(input), "some data here");
    }

    #[test]
    fn collapses_whitespace() {
        let input = "hello\n\n\n\n\nworld";
        assert_eq!(strip_markup(input), "hello\n\nworld");
    }

    #[test]
    fn preserves_clean_text() {
        let input = "this is normal text";
        assert_eq!(strip_markup(input), "this is normal text");
    }
}
