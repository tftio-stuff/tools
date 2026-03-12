//! Output utilities for consistent terminal formatting.

use colored::Colorize;
use is_terminal::IsTerminal;
use std::io;

/// Check if stdout is a TTY (terminal).
///
/// Returns `true` if stdout is connected to a terminal, `false` if piped/redirected.
/// This is used to determine whether to use colored output and fancy formatting.
#[must_use]
pub fn is_tty() -> bool {
    io::stdout().is_terminal()
}

/// Format a success message with green checkmark.
///
/// Returns colored output if stdout is a TTY, plain text otherwise.
#[must_use]
pub fn success(msg: &str) -> String {
    if is_tty() {
        format!("{} {}", "✅".green(), msg.green())
    } else {
        format!("[OK] {msg}")
    }
}

/// Format an error message with red X.
///
/// Returns colored output if stdout is a TTY, plain text otherwise.
#[must_use]
pub fn error(msg: &str) -> String {
    if is_tty() {
        format!("{} {}", "❌".red(), msg.red().bold())
    } else {
        format!("[ERROR] {msg}")
    }
}

/// Format a warning message with yellow warning sign.
///
/// Returns colored output if stdout is a TTY, plain text otherwise.
#[must_use]
pub fn warning(msg: &str) -> String {
    if is_tty() {
        format!("{} {}", "⚠️".yellow(), msg.yellow())
    } else {
        format!("[WARNING] {msg}")
    }
}

/// Format an info message with blue info sign.
///
/// Returns colored output if stdout is a TTY, plain text otherwise.
#[must_use]
pub fn info(msg: &str) -> String {
    if is_tty() {
        format!("{} {}", "ℹ️".blue(), msg.blue())
    } else {
        format!("[INFO] {msg}")
    }
}

/// Format a header with separator line.
///
/// Returns colored output if stdout is a TTY, plain text otherwise.
#[must_use]
pub fn header(title: &str, width: usize) -> String {
    if is_tty() {
        format!("{}\n{}", title.bold().cyan(), "=".repeat(width).cyan())
    } else {
        format!("{title}\n{}", "=".repeat(width))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_tty_returns_bool() {
        // Just verify it returns a boolean without panicking
        let _result = is_tty();
        // Function executed successfully if we get here
    }

    #[test]
    fn test_success_format() {
        let msg = success("test message");
        // Should contain the message
        assert!(msg.contains("test message"));
        // Should contain either emoji or [OK]
        assert!(msg.contains("✅") || msg.contains("[OK]"));
    }

    #[test]
    fn test_error_format() {
        let msg = error("test error");
        assert!(msg.contains("test error"));
        assert!(msg.contains("❌") || msg.contains("[ERROR]"));
    }

    #[test]
    fn test_warning_format() {
        let msg = warning("test warning");
        assert!(msg.contains("test warning"));
        assert!(msg.contains("⚠️") || msg.contains("[WARNING]"));
    }

    #[test]
    fn test_info_format() {
        let msg = info("test info");
        assert!(msg.contains("test info"));
        assert!(msg.contains("ℹ️") || msg.contains("[INFO]"));
    }

    #[test]
    fn test_header_format() {
        let msg = header("Test Header", 20);
        assert!(msg.contains("Test Header"));
        assert!(msg.contains("===================="));
    }
}
