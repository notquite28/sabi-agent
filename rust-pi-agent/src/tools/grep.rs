//! Grep tool implementation.
//!
//! Ported from:
//! - `pi/packages/coding-agent/src/core/tools/grep.ts`
//!
//! Simplifications:
//! - Uses installed `rg` instead of downloading or embedding ripgrep.
//! - No custom remote operations or JSON event streaming cache yet.
