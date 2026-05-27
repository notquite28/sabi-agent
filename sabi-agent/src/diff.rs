//! Diff generation and terminal diff rendering.
//!
//! Ported from:
//! - `pi/packages/coding-agent/src/core/tools/edit-diff.ts`
//! - `pi/packages/coding-agent/src/modes/interactive/components/diff.ts`
//!
//! Simplifications:
//! - Will use the `similar` crate instead of porting all TypeScript diff logic directly.
//! - No TUI box rendering or collapsible output.

use similar::{ChangeTag, TextDiff};

pub fn unified_patch(path: &str, old: &str, new: &str) -> String {
    TextDiff::from_lines(old, new)
        .unified_diff()
        .header(&format!("a/{path}"), &format!("b/{path}"))
        .to_string()
}

pub fn render_terminal_diff(old: &str, new: &str) -> String {
    let diff = TextDiff::from_lines(old, new);
    let mut rendered = String::new();

    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => '-',
            ChangeTag::Insert => '+',
            ChangeTag::Equal => ' ',
        };
        rendered.push(sign);
        rendered.push_str(&change.to_string());
    }

    rendered
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unified_patch_contains_header_and_changes() {
        let old = "line one\nline two\n";
        let new = "line one\nline two modified\n";
        let patch = unified_patch("file.txt", old, new);
        assert!(patch.contains("--- a/file.txt"));
        assert!(patch.contains("+++ b/file.txt"));
        assert!(patch.contains("-line two\n"));
        assert!(patch.contains("+line two modified\n"));
    }

    #[test]
    fn render_terminal_diff_shows_plus_and_minus() {
        let old = "keep\nremove\n";
        let new = "keep\nadd\n";
        let rendered = render_terminal_diff(old, new);
        assert!(rendered.contains("-remove\n"));
        assert!(rendered.contains("+add\n"));
        assert!(rendered.contains(" keep\n"));
    }

    #[test]
    fn render_terminal_diff_empty_when_identical() {
        let text = "same\nlines\n";
        let rendered = render_terminal_diff(text, text);
        assert!(rendered.contains(" same\n"));
        assert!(rendered.contains(" lines\n"));
        assert!(!rendered.contains('+'));
        assert!(!rendered.contains('-'));
    }
}
