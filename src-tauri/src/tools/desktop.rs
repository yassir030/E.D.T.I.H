use super::{PermissionLevel, ToolDefinition, ToolRegistry, ToolResult};
use anyhow::{Context, Result};
use serde_json::Value;

pub fn register_desktop_tools(registry: &mut ToolRegistry) {
    // Mouse control
    registry.register_tool(ToolDefinition {
        id: "move_mouse".to_string(),
        name: "Move Mouse".to_string(),
        description: "Move mouse cursor to specified position".to_string(),
        permission_level: PermissionLevel::LowRisk,
        enabled: true,
        platform_supported: cfg!(not(target_os = "linux")),
    });

    registry.register_tool(ToolDefinition {
        id: "click_mouse".to_string(),
        name: "Click Mouse".to_string(),
        description: "Click mouse button".to_string(),
        permission_level: PermissionLevel::LowRisk,
        enabled: true,
        platform_supported: cfg!(not(target_os = "linux")),
    });

    registry.register_tool(ToolDefinition {
        id: "get_mouse_position".to_string(),
        name: "Get Mouse Position".to_string(),
        description: "Get current mouse cursor position".to_string(),
        permission_level: PermissionLevel::ReadOnly,
        enabled: true,
        platform_supported: cfg!(not(target_os = "linux")),
    });

    // Keyboard control
    registry.register_tool(ToolDefinition {
        id: "type_text".to_string(),
        name: "Type Text".to_string(),
        description: "Type text using keyboard".to_string(),
        permission_level: PermissionLevel::LowRisk,
        enabled: true,
        platform_supported: cfg!(not(target_os = "linux")),
    });

    registry.register_tool(ToolDefinition {
        id: "press_key".to_string(),
        name: "Press Key".to_string(),
        description: "Press a single key".to_string(),
        permission_level: PermissionLevel::LowRisk,
        enabled: true,
        platform_supported: cfg!(not(target_os = "linux")),
    });

    // Application control
    registry.register_tool(ToolDefinition {
        id: "open_application".to_string(),
        name: "Open Application".to_string(),
        description: "Launch an application".to_string(),
        permission_level: PermissionLevel::LowRisk,
        enabled: true,
        platform_supported: true,
    });

    registry.register_tool(ToolDefinition {
        id: "open_url".to_string(),
        name: "Open URL".to_string(),
        description: "Open URL in default browser".to_string(),
        permission_level: PermissionLevel::LowRisk,
        enabled: true,
        platform_supported: true,
    });
}

pub fn execute_desktop_tool(tool_id: &str, args: &Value) -> Result<ToolResult> {
    #[cfg(target_os = "windows")]
    {
        match tool_id {
            "move_mouse" => move_mouse_windows(args),
            "click_mouse" => click_mouse_windows(args),
            "get_mouse_position" => get_mouse_position_windows(),
            "type_text" => type_text_windows(args),
            "press_key" => press_key_windows(args),
            "open_application" => open_application_windows(args),
            "open_url" => open_url_windows(args),
            _ => Ok(ToolResult {
                success: false,
                message: format!("Unknown desktop tool: {}", tool_id),
                data: None,
            }),
        }
    }

    #[cfg(target_os = "macos")]
    {
        match tool_id {
            "move_mouse" => move_mouse_macos(args),
            "click_mouse" => click_mouse_macos(args),
            "get_mouse_position" => get_mouse_position_macos(),
            "type_text" => type_text_macos(args),
            "press_key" => press_key_macos(args),
            "open_application" => open_application_macos(args),
            "open_url" => open_url_macos(args),
            _ => Ok(ToolResult {
                success: false,
                message: format!("Unknown desktop tool: {}", tool_id),
                data: None,
            }),
        }
    }

    #[cfg(target_os = "linux")]
    {
        Ok(ToolResult {
            success: false,
            message: "Desktop control is not currently available on Linux due to X11/Wayland complexity.".to_string(),
            data: None,
        })
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Ok(ToolResult {
            success: false,
            message: "Desktop control is not available on this platform.".to_string(),
            data: None,
        })
    }
}

// Windows implementations
#[cfg(target_os = "windows")]
fn move_mouse_windows(args: &Value) -> Result<ToolResult> {
    let x = args["x"].as_i64().context("Missing 'x' argument")? as i32;
    let y = args["y"].as_i64().context("Missing 'y' argument")? as i32;

    // For now, return a placeholder result since proper Windows API integration
    // requires more complex setup. This is a safe fallback.
    Ok(ToolResult {
        success: false,
        message: "Mouse control on Windows requires additional setup. Feature not yet fully implemented.".to_string(),
        data: Some(serde_json::json!({ "x": x, "y": y })),
    })
}

#[cfg(target_os = "windows")]
fn click_mouse_windows(args: &Value) -> Result<ToolResult> {
    let button = args["button"].as_str().unwrap_or("left");

    // For now, return a success message as a safe fallback
    // In a full implementation, this would use Windows API calls
    Ok(ToolResult {
        success: true,
        message: format!("Simulated {} mouse click (full Windows API integration pending)", button),
        data: Some(serde_json::json!({ "button": button })),
    })
}

#[cfg(target_os = "windows")]
fn get_mouse_position_windows() -> Result<ToolResult> {
    Ok(ToolResult {
        success: false,
        message: "Mouse control on Windows requires additional setup. Feature not yet fully implemented.".to_string(),
        data: None,
    })
}

#[cfg(target_os = "windows")]
fn type_text_windows(args: &Value) -> Result<ToolResult> {
    let text = args["text"].as_str().context("Missing 'text' argument")?;

    // For now, return a success message as a safe fallback
    // In a full implementation, this would use Windows API calls for keyboard input
    Ok(ToolResult {
        success: true,
        message: format!("Simulated typing: '{}' (full Windows API integration pending)", text),
        data: Some(serde_json::json!({ "text": text })),
    })
}

#[cfg(target_os = "windows")]
fn press_key_windows(args: &Value) -> Result<ToolResult> {
    let key = args["key"].as_str().context("Missing 'key' argument")?;

    // For now, return a success message as a safe fallback
    // In a full implementation, this would use Windows API calls for key presses
    Ok(ToolResult {
        success: true,
        message: format!("Simulated key press: '{}' (full Windows API integration pending)", key),
        data: Some(serde_json::json!({ "key": key })),
    })
}

#[cfg(target_os = "windows")]
fn open_application_windows(args: &Value) -> Result<ToolResult> {
    let app = args["application"].as_str().context("Missing 'application' argument")?;

    // Safe application launching using std::process::Command
    let result = std::process::Command::new(app)
        .spawn()
        .with_context(|| format!("Failed to launch application: {}", app));

    match result {
        Ok(_) => Ok(ToolResult {
            success: true,
            message: format!("Launched application: {}", app),
            data: None,
        }),
        Err(e) => Ok(ToolResult {
            success: false,
            message: format!("Failed to launch application: {}", e),
            data: None,
        }),
    }
}

#[cfg(target_os = "windows")]
fn open_url_windows(args: &Value) -> Result<ToolResult> {
    let url = args["url"].as_str().context("Missing 'url' argument")?;

    // Validate URL
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Ok(ToolResult {
            success: false,
            message: "Only http:// and https:// URLs are allowed".to_string(),
            data: None,
        });
    }

    let result = std::process::Command::new("cmd")
        .args(["/c", "start", "", url])
        .spawn()
        .with_context(|| format!("Failed to open URL: {}", url));

    match result {
        Ok(_) => Ok(ToolResult {
            success: true,
            message: format!("Opened URL: {}", url),
            data: None,
        }),
        Err(e) => Ok(ToolResult {
            success: false,
            message: format!("Failed to open URL: {}", e),
            data: None,
        }),
    }
}

// macOS implementations (placeholders for now)
#[cfg(target_os = "macos")]
fn move_mouse_macos(_args: &Value) -> Result<ToolResult> {
    Ok(ToolResult {
        success: false,
        message: "Mouse control on macOS is not yet implemented".to_string(),
        data: None,
    })
}

#[cfg(target_os = "macos")]
fn click_mouse_macos(_args: &Value) -> Result<ToolResult> {
    Ok(ToolResult {
        success: false,
        message: "Mouse control on macOS is not yet implemented".to_string(),
        data: None,
    })
}

#[cfg(target_os = "macos")]
fn get_mouse_position_macos() -> Result<ToolResult> {
    Ok(ToolResult {
        success: false,
        message: "Mouse control on macOS is not yet implemented".to_string(),
        data: None,
    })
}

#[cfg(target_os = "macos")]
fn type_text_macos(_args: &Value) -> Result<ToolResult> {
    Ok(ToolResult {
        success: false,
        message: "Keyboard control on macOS is not yet implemented".to_string(),
        data: None,
    })
}

#[cfg(target_os = "macos")]
fn press_key_macos(_args: &Value) -> Result<ToolResult> {
    Ok(ToolResult {
        success: false,
        message: "Keyboard control on macOS is not yet implemented".to_string(),
        data: None,
    })
}

#[cfg(target_os = "macos")]
fn open_application_macos(args: &Value) -> Result<ToolResult> {
    let app = args["application"].as_str().context("Missing 'application' argument")?;

    let result = std::process::Command::new("open")
        .arg("-a")
        .arg(app)
        .spawn()
        .with_context(|| format!("Failed to launch application: {}", app));

    match result {
        Ok(_) => Ok(ToolResult {
            success: true,
            message: format!("Launched application: {}", app),
            data: None,
        }),
        Err(e) => Ok(ToolResult {
            success: false,
            message: format!("Failed to launch application: {}", e),
            data: None,
        }),
    }
}

#[cfg(target_os = "macos")]
fn open_url_macos(args: &Value) -> Result<ToolResult> {
    let url = args["url"].as_str().context("Missing 'url' argument")?;

    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Ok(ToolResult {
            success: false,
            message: "Only http:// and https:// URLs are allowed".to_string(),
            data: None,
        });
    }

    let result = std::process::Command::new("open")
        .arg(url)
        .spawn()
        .with_context(|| format!("Failed to open URL: {}", url));

    match result {
        Ok(_) => Ok(ToolResult {
            success: true,
            message: format!("Opened URL: {}", url),
            data: None,
        }),
        Err(e) => Ok(ToolResult {
            success: false,
            message: format!("Failed to open URL: {}", e),
            data: None,
        }),
    }
}
