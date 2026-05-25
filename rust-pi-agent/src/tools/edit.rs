//! Edit tool implementation.
//!
//! Ported from:
//! - `pi/packages/coding-agent/src/core/tools/edit.ts`
//! - `pi/packages/coding-agent/src/core/tools/edit-diff.ts`
//!
//! Simplifications:
//! - Starts with exact text replacement only.
//! - No fuzzy Unicode normalization, preview component, or TUI box rendering.
