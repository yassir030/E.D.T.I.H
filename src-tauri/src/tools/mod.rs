mod desktop;
mod filesystem;

// Re-export only the registration functions
pub use desktop::register_desktop_tools;
pub use filesystem::register_filesystem_tools;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Internal tool routing function
fn execute_tool_by_id(registry: &ToolRegistry, tool_id: &str, args: &serde_json::Value) -> anyhow::Result<ToolResult> {
    validate_desktop_action(tool_id, args)?;
    let tool = registry.get_tool(tool_id).ok_or_else(|| anyhow::anyhow!("Tool not found"))?;
    
    match tool.category {
        ToolCategory::Filesystem => filesystem::execute_filesystem_tool(tool_id, args),
        ToolCategory::Desktop => desktop::execute_desktop_tool(tool_id, args),
    }
}

pub fn validate_desktop_action(tool_id: &str, args: &serde_json::Value) -> anyhow::Result<()> {
    let forbidden = [
        "submit",
        "verzenden",
        "inleveren",
        "send",
        "finish",
        "complete",
        "confirm",
        "publish",
        "post",
    ];

    let args_str = args.to_string().to_ascii_lowercase();
    let tool_id_lower = tool_id.to_ascii_lowercase();

    for word in &forbidden {
        if tool_id_lower.contains(word) || args_str.contains(word) {
            eprintln!("[EDITH][SAFETY] Action blocked by E.D.I.T.H. safety policy: tool={} args={}", tool_id, args);
            anyhow::bail!("Action blocked by E.D.I.T.H. safety policy.");
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionLevel {
    ReadOnly,
    LowRisk,
    Destructive,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolCategory {
    Desktop,
    Filesystem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub permission_level: PermissionLevel,
    pub category: ToolCategory,
    pub enabled: bool,
    pub platform_supported: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRequest {
    pub tool_id: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct ToolRegistry {
    tools: HashMap<String, ToolDefinition>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };
        register_filesystem_tools(&mut registry);
        register_desktop_tools(&mut registry);
        registry
    }

    fn register_tool(&mut self, tool: ToolDefinition) {
        self.tools.insert(tool.id.clone(), tool);
    }

    pub fn get_tool(&self, id: &str) -> Option<&ToolDefinition> {
        self.tools.get(id)
    }

    pub fn list_tools(&self) -> Vec<ToolDefinition> {
        self.tools.values().cloned().collect()
    }

    pub fn requires_confirmation(&self, tool_id: &str) -> bool {
        match self.get_tool(tool_id) {
            Some(tool) => matches!(
                tool.permission_level,
                PermissionLevel::Destructive | PermissionLevel::System
            ),
            None => true,
        }
    }

    pub fn execute_tool(&self, tool_id: &str, args: &serde_json::Value) -> anyhow::Result<ToolResult> {
        execute_tool_by_id(self, tool_id, args)
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
