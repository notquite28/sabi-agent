//! Typed tool approval policy shared by frontends.
//!
//! Ported from:
//! - `pi/packages/coding-agent/src/modes/interactive/interactive-mode.ts`
//! - `pi/packages/coding-agent/src/core/tools/index.ts`
//!
//! Simplifications:
//! - Uses a fixed built-in policy for the current tool set.
//! - No sandbox profiles, per-project trust settings, or persisted approvals yet.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolRiskLevel {
    ReadOnly,
    ExternalNetwork,
    FileMutation,
    Shell,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolApprovalRequest {
    pub id: String,
    pub name: String,
    pub args: Value,
    pub risk_level: ToolRiskLevel,
    pub approval_required: bool,
    pub summary: String,
}

impl ToolApprovalRequest {
    pub fn new(id: impl Into<String>, name: impl Into<String>, args: Value) -> Self {
        let name = name.into();
        let risk_level = risk_level_for(&name);
        Self {
            id: id.into(),
            summary: approval_summary(&name, &args),
            approval_required: approval_required_for(risk_level),
            risk_level,
            name,
            args,
        }
    }
}

pub fn risk_level_for(name: &str) -> ToolRiskLevel {
    match name {
        "bash" => ToolRiskLevel::Shell,
        "write" | "edit" => ToolRiskLevel::FileMutation,
        "web_search" | "exa_search" => ToolRiskLevel::ExternalNetwork,
        _ => ToolRiskLevel::ReadOnly,
    }
}

pub fn approval_required_for(risk_level: ToolRiskLevel) -> bool {
    matches!(
        risk_level,
        ToolRiskLevel::FileMutation | ToolRiskLevel::Shell
    )
}

pub fn tool_requires_approval(name: &str) -> bool {
    approval_required_for(risk_level_for(name))
}

pub fn approval_summary(name: &str, args: &Value) -> String {
    match name {
        "bash" => args
            .get("command")
            .and_then(Value::as_str)
            .map(|command| format!("command={command:?}"))
            .unwrap_or_else(|| args.to_string()),
        "write" | "edit" => args
            .get("path")
            .and_then(Value::as_str)
            .map(|path| format!("path={path:?}"))
            .unwrap_or_else(|| args.to_string()),
        _ => args.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn mutating_and_shell_tools_require_approval() {
        assert!(tool_requires_approval("write"));
        assert!(tool_requires_approval("edit"));
        assert!(tool_requires_approval("bash"));
        assert!(!tool_requires_approval("read"));
        assert!(!tool_requires_approval("ls"));
        assert!(!tool_requires_approval("grep"));
        assert!(!tool_requires_approval("find"));
        assert!(!tool_requires_approval("web_search"));
        assert!(!tool_requires_approval("exa_search"));
    }

    #[test]
    fn approval_request_contains_risk_and_summary() {
        let request = ToolApprovalRequest::new(
            "call-1",
            "bash",
            json!({ "command": "cargo test", "timeout": 120 }),
        );

        assert_eq!(request.id, "call-1");
        assert_eq!(request.name, "bash");
        assert_eq!(request.risk_level, ToolRiskLevel::Shell);
        assert!(request.approval_required);
        assert_eq!(request.summary, "command=\"cargo test\"");
    }

    #[test]
    fn file_tool_summary_uses_path() {
        let request = ToolApprovalRequest::new(
            "call-2",
            "edit",
            json!({ "path": "src/lib.rs", "old_text": "a", "new_text": "b" }),
        );

        assert_eq!(request.risk_level, ToolRiskLevel::FileMutation);
        assert_eq!(request.summary, "path=\"src/lib.rs\"");
    }
}
