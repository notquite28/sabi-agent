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
