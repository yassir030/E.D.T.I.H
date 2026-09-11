use super::{PermissionLevel, ToolDefinition, ToolRegistry, ToolResult, ToolCategory};
use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::path::Path;

pub fn register_filesystem_tools(registry: &mut ToolRegistry) {
    registry.register_tool(ToolDefinition {
        id: "list_directory".to_string(),
        name: "List Directory".to_string(),
        description: "List contents of a directory".to_string(),
        permission_level: PermissionLevel::ReadOnly,
        category: ToolCategory::Filesystem,
        enabled: true,
        platform_supported: true,
    });

    registry.register_tool(ToolDefinition {
        id: "read_file".to_string(),
        name: "Read File".to_string(),
        description: "Read contents of a text file".to_string(),
        permission_level: PermissionLevel::ReadOnly,
        category: ToolCategory::Filesystem,
        enabled: true,
        platform_supported: true,
    });

    registry.register_tool(ToolDefinition {
        id: "create_file".to_string(),
        name: "Create File".to_string(),
        description: "Create a new file with content".to_string(),
        permission_level: PermissionLevel::Destructive,
        category: ToolCategory::Filesystem,
        enabled: true,
        platform_supported: true,
    });

    registry.register_tool(ToolDefinition {
        id: "delete_file".to_string(),
        name: "Delete File".to_string(),
        description: "Delete a file".to_string(),
        permission_level: PermissionLevel::Destructive,
        category: ToolCategory::Filesystem,
        enabled: true,
        platform_supported: true,
    });

    registry.register_tool(ToolDefinition {
        id: "move_file".to_string(),
        name: "Move File".to_string(),
        description: "Move or rename a file".to_string(),
        permission_level: PermissionLevel::Destructive,
        category: ToolCategory::Filesystem,
        enabled: true,
        platform_supported: true,
    });
}

pub fn execute_filesystem_tool(tool_id: &str, args: &Value) -> Result<ToolResult> {
    match tool_id {
        "list_directory" => list_directory(args),
        "read_file" => read_file(args),
        "create_file" => create_file(args),
        "delete_file" => delete_file(args),
        "move_file" => move_file(args),
        _ => Ok(ToolResult {
            success: false,
            message: format!("Unknown filesystem tool: {}", tool_id),
            data: None,
        }),
    }
}

fn list_directory(args: &Value) -> Result<ToolResult> {
    let path = args["path"]
        .as_str()
        .context("Missing 'path' argument")?;

    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Ok(ToolResult {
            success: false,
            message: format!("Directory not found: {}", path),
            data: None,
        });
    }

    if !path_obj.is_dir() {
        return Ok(ToolResult {
            success: false,
            message: format!("Path is not a directory: {}", path),
            data: None,
        });
    }

    let entries = fs::read_dir(path)
        .with_context(|| format!("Failed to read directory: {}", path))?;

    let mut items = Vec::new();
    for entry in entries {
        let entry = entry.with_context(|| "Failed to read directory entry")?;
        let metadata = entry.metadata().with_context(|| "Failed to get file metadata")?;

        let item = serde_json::json!({
            "name": entry.file_name().to_string_lossy().to_string(),
            "path": entry.path().to_string_lossy().to_string(),
            "is_file": metadata.is_file(),
            "is_dir": metadata.is_dir(),
            "size": metadata.len(),
            "modified": metadata.modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
        });
        items.push(item);
    }

    Ok(ToolResult {
        success: true,
        message: format!("Listed {} items", items.len()),
        data: Some(serde_json::json!({ "items": items })),
    })
}

fn read_file(args: &Value) -> Result<ToolResult> {
    let path = args["path"]
        .as_str()
        .context("Missing 'path' argument")?;

    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Ok(ToolResult {
            success: false,
            message: format!("File not found: {}", path),
            data: None,
        });
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path))?;

    // Limit content size to prevent issues
    let max_size = 100_000; // 100KB limit
    let truncated_content = if content.len() > max_size {
        format!("{}... (truncated)", &content[..max_size])
    } else {
        content
    };

    Ok(ToolResult {
        success: true,
        message: format!("Read file: {}", path),
        data: Some(serde_json::json!({ "content": truncated_content })),
    })
}

fn create_file(args: &Value) -> Result<ToolResult> {
    let path = args["path"]
        .as_str()
        .context("Missing 'path' argument")?;
    let content = args["content"]
        .as_str()
        .unwrap_or("");

    fs::write(path, content)
        .with_context(|| format!("Failed to create file: {}", path))?;

    Ok(ToolResult {
        success: true,
        message: format!("Created file: {}", path),
        data: None,
    })
}

fn delete_file(args: &Value) -> Result<ToolResult> {
    let path = args["path"]
        .as_str()
        .context("Missing 'path' argument")?;

    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Ok(ToolResult {
            success: false,
            message: format!("File not found: {}", path),
            data: None,
        });
    }

    if path_obj.is_dir() {
        return Ok(ToolResult {
            success: false,
            message: "Cannot delete directories with delete_file. Use delete_directory instead.".to_string(),
            data: None,
        });
    }

    fs::remove_file(path)
        .with_context(|| format!("Failed to delete file: {}", path))?;

    Ok(ToolResult {
        success: true,
        message: format!("Deleted file: {}", path),
        data: None,
    })
}

fn move_file(args: &Value) -> Result<ToolResult> {
    let source = args["source"]
        .as_str()
        .context("Missing 'source' argument")?;
    let destination = args["destination"]
        .as_str()
        .context("Missing 'destination' argument")?;

    let source_obj = Path::new(source);
    if !source_obj.exists() {
        return Ok(ToolResult {
            success: false,
            message: format!("Source not found: {}", source),
            data: None,
        });
    }

    fs::rename(source, destination)
        .with_context(|| format!("Failed to move file from {} to {}", source, destination))?;

    Ok(ToolResult {
        success: true,
        message: format!("Moved file from {} to {}", source, destination),
        data: None,
    })
}
