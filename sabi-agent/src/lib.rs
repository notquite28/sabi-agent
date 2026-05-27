//! Reusable library entry point for the Sabi Agent engine.
//!
//! Ported from:
//! - `pi/packages/agent/src/agent.ts`
//! - `pi/packages/coding-agent/src/main.ts`
//!
//! Simplifications:
//! - Exposes the same modules used by the CLI before adding a stable embedding API.
//! - No SDK, RPC server, extension host, or desktop-specific API yet.

pub mod agent;
pub mod app;
pub mod approval;
pub mod config;
pub mod desktop;
pub mod diff;
pub mod events;
pub mod llm;
pub mod messages;
pub mod onboarding;
pub mod session;
pub mod skills;
pub mod slash;
pub mod system_prompt;
pub mod tools;
