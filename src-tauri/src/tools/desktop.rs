use super::{PermissionLevel, ToolDefinition, ToolRegistry, ToolResult, ToolCategory};
use anyhow::{bail, Context, Result};
use serde_json::Value;
use image::{DynamicImage, ImageFormat};
use std::io::Cursor;

use screenshots::Screen;
use base64::{engine::general_purpose, Engine as _};
use std::thread;
use std::time::Duration;

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::POINT;
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
    KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP,
    MOUSEEVENTF_WHEEL, MOUSEINPUT, VK_BACK, VK_CAPITAL, VK_CONTROL, VK_DELETE, VK_DOWN, VK_END,
    VK_ESCAPE, VK_F1, VK_F10, VK_F11, VK_F12, VK_F2, VK_F3, VK_F4, VK_F5, VK_F6, VK_F7, VK_F8,
    VK_F9, VK_HOME, VK_INSERT, VK_LEFT, VK_LWIN, VK_MENU, VK_NEXT, VK_PRIOR, VK_RETURN, VK_RIGHT,
    VK_SHIFT, VK_SNAPSHOT, VK_SPACE, VK_TAB, VK_UP,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{GetCursorPos, SetCursorPos};

pub fn register_desktop_tools(registry: &mut ToolRegistry) {
    // Mouse control
    registry.register_tool(ToolDefinition {
        id: "move_mouse".to_string(),
        name: "Move Mouse".to_string(),
        description: "Move mouse cursor to specified position".to_string(),
        permission_level: PermissionLevel::LowRisk,
        category: ToolCategory::Desktop,
        enabled: true,
        platform_supported: cfg!(not(target_os = "linux")),
    });

    registry.register_tool(ToolDefinition {
        id: "click_mouse".to_string(),
        name: "Click Mouse".to_string(),
        description: "Click mouse button".to_string(),
        permission_level: PermissionLevel::LowRisk,
        category: ToolCategory::Desktop,
        enabled: true,
        platform_supported: cfg!(not(target_os = "linux")),
    });

    registry.register_tool(ToolDefinition {
        id: "get_mouse_position".to_string(),
        name: "Get Mouse Position".to_string(),
        description: "Get current mouse cursor position".to_string(),
        permission_level: PermissionLevel::ReadOnly,
        category: ToolCategory::Desktop,
        enabled: true,
        platform_supported: cfg!(not(target_os = "linux")),
    });

    // Keyboard control
    registry.register_tool(ToolDefinition {
        id: "type_text".to_string(),
        name: "Type Text".to_string(),
        description: "Type text using keyboard".to_string(),
        permission_level: PermissionLevel::LowRisk,
        category: ToolCategory::Desktop,
        enabled: true,
        platform_supported: cfg!(not(target_os = "linux")),
    });

    registry.register_tool(ToolDefinition {
        id: "press_key".to_string(),
        name: "Press Key".to_string(),
        description: "Press a single key".to_string(),
        permission_level: PermissionLevel::LowRisk,
        category: ToolCategory::Desktop,
        enabled: true,
        platform_supported: cfg!(not(target_os = "linux")),
    });

    registry.register_tool(ToolDefinition {
        id: "hotkey".to_string(),
        name: "Hotkey".to_string(),
        description: "Press key combination (e.g., Ctrl+C)".to_string(),
        permission_level: PermissionLevel::LowRisk,
        category: ToolCategory::Desktop,
        enabled: true,
        platform_supported: cfg!(not(target_os = "linux")),
    });

    registry.register_tool(ToolDefinition {
        id: "scroll".to_string(),
        name: "Scroll".to_string(),
        description: "Scroll mouse wheel".to_string(),
        permission_level: PermissionLevel::LowRisk,
        category: ToolCategory::Desktop,
        enabled: true,
        platform_supported: cfg!(not(target_os = "linux")),
    });

    // Application control
    registry.register_tool(ToolDefinition {
        id: "open_application".to_string(),
        name: "Open Application".to_string(),
        description: "Launch an application".to_string(),
        permission_level: PermissionLevel::LowRisk,
        category: ToolCategory::Desktop,
        enabled: true,
        platform_supported: true,
    });

    registry.register_tool(ToolDefinition {
        id: "get_screen_screenshot".to_string(),
        name: "Get Screen Screenshot".to_string(),
        description: "Capture a screenshot of the primary screen".to_string(),
        permission_level: PermissionLevel::ReadOnly,
        category: ToolCategory::Desktop,
        enabled: true,
        platform_supported: true,
    });

    registry.register_tool(ToolDefinition {
        id: "open_url".to_string(),
        name: "Open URL".to_string(),
        description: "Open URL in default browser".to_string(),
        permission_level: PermissionLevel::LowRisk,
        category: ToolCategory::Desktop,
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
            "get_screen_screenshot" => get_screen_screenshot_windows(),
            "type_text" => type_text_windows(args),
            "press_key" => press_key_windows(args),
            "hotkey" => hotkey_windows(args),
            "scroll" => scroll_windows(args),
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
            "hotkey" => hotkey_macos(args),
            "scroll" => scroll_macos(args),
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

// Windows native input helpers
#[cfg(target_os = "windows")]
fn send_inputs(inputs: &[INPUT]) -> Result<()> {
    if inputs.is_empty() {
        return Ok(());
    }

    let sent = unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        )
    };

    if sent != inputs.len() as u32 {
        bail!(
            "SendInput failed: sent {} of {} input events",
            sent,
            inputs.len()
        );
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn create_mouse_input(flags: u32, data: u32) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: data,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

#[cfg(target_os = "windows")]
fn create_key_input(vk: u16, key_up: bool, extended: bool) -> INPUT {
    let mut flags = 0u32;
    if key_up {
        flags |= KEYEVENTF_KEYUP;
    }
    if extended {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }

    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

#[cfg(target_os = "windows")]
fn create_unicode_input(ch: u16, key_up: bool) -> INPUT {
    let mut flags = KEYEVENTF_UNICODE;
    if key_up {
        flags |= KEYEVENTF_KEYUP;
    }

    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: 0,
                wScan: ch,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

#[cfg(target_os = "windows")]
pub fn click_mouse(button: &str) -> Result<()> {
    match button.to_ascii_lowercase().as_str() {
        "left" | "primary" => {
            let inputs = [
                create_mouse_input(MOUSEEVENTF_LEFTDOWN, 0),
                create_mouse_input(MOUSEEVENTF_LEFTUP, 0),
            ];
            send_inputs(&inputs)?;
        }
        "right" | "secondary" => {
            let inputs = [
                create_mouse_input(MOUSEEVENTF_RIGHTDOWN, 0),
                create_mouse_input(MOUSEEVENTF_RIGHTUP, 0),
            ];
            send_inputs(&inputs)?;
        }
        "middle" | "wheel" => {
            let inputs = [
                create_mouse_input(MOUSEEVENTF_MIDDLEDOWN, 0),
                create_mouse_input(MOUSEEVENTF_MIDDLEUP, 0),
            ];
            send_inputs(&inputs)?;
        }
        "double" | "double_click" | "left_double" => {
            let click1 = [
                create_mouse_input(MOUSEEVENTF_LEFTDOWN, 0),
                create_mouse_input(MOUSEEVENTF_LEFTUP, 0),
            ];
            send_inputs(&click1)?;
            std::thread::sleep(std::time::Duration::from_millis(50));
            let click2 = [
                create_mouse_input(MOUSEEVENTF_LEFTDOWN, 0),
                create_mouse_input(MOUSEEVENTF_LEFTUP, 0),
            ];
            send_inputs(&click2)?;
        }
        "right_double" | "double_right" => {
            let click1 = [
                create_mouse_input(MOUSEEVENTF_RIGHTDOWN, 0),
                create_mouse_input(MOUSEEVENTF_RIGHTUP, 0),
            ];
            send_inputs(&click1)?;
            std::thread::sleep(std::time::Duration::from_millis(50));
            let click2 = [
                create_mouse_input(MOUSEEVENTF_RIGHTDOWN, 0),
                create_mouse_input(MOUSEEVENTF_RIGHTUP, 0),
            ];
            send_inputs(&click2)?;
        }
        _ => bail!("Unsupported mouse button: '{}'. Supported: left, right, middle, double", button),
    }

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn get_mouse_position() -> Result<(i32, i32)> {
    let mut pt = POINT { x: 0, y: 0 };
    let success = unsafe { GetCursorPos(&mut pt) };
    if success == 0 {
        bail!("Windows API GetCursorPos failed");
    }
    Ok((pt.x, pt.y))
}

#[cfg(target_os = "windows")]
pub fn type_text(text: &str) -> Result<()> {
    if text.is_empty() {
        return Ok(());
    }

    for ch in text.encode_utf16() {
        let inputs = [
            create_unicode_input(ch, false),
            create_unicode_input(ch, true),
        ];
        send_inputs(&inputs)?;
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn map_key_to_vk(key: &str) -> Result<(u16, bool)> {
    let key_normalized = key.trim().to_ascii_lowercase();

    let (vk, extended) = match key_normalized.as_str() {
        "enter" | "return" => (VK_RETURN, false),
        "escape" | "esc" => (VK_ESCAPE, false),
        "tab" => (VK_TAB, false),
        "backspace" | "back" => (VK_BACK, false),
        "delete" | "del" => (VK_DELETE, true),
        "space" | "spacebar" => (VK_SPACE, false),
        "up" | "arrowup" | "arrow_up" => (VK_UP, true),
        "down" | "arrowdown" | "arrow_down" => (VK_DOWN, true),
        "left" | "arrowleft" | "arrow_left" => (VK_LEFT, true),
        "right" | "arrowright" | "arrow_right" => (VK_RIGHT, true),
        "ctrl" | "control" => (VK_CONTROL, false),
        "shift" => (VK_SHIFT, false),
        "alt" | "menu" => (VK_MENU, false),
        "win" | "windows" | "super" | "meta" | "cmd" => (VK_LWIN, true),
        "f1" => (VK_F1, false),
        "f2" => (VK_F2, false),
        "f3" => (VK_F3, false),
        "f4" => (VK_F4, false),
        "f5" => (VK_F5, false),
        "f6" => (VK_F6, false),
        "f7" => (VK_F7, false),
        "f8" => (VK_F8, false),
        "f9" => (VK_F9, false),
        "f10" => (VK_F10, false),
        "f11" => (VK_F11, false),
        "f12" => (VK_F12, false),
        "pageup" | "page_up" | "pgup" => (VK_PRIOR, true),
        "pagedown" | "page_down" | "pgdn" => (VK_NEXT, true),
        "home" => (VK_HOME, true),
        "end" => (VK_END, true),
        "insert" | "ins" => (VK_INSERT, true),
        "capslock" | "caps_lock" => (VK_CAPITAL, false),
        "printscreen" | "prtscr" | "prtsc" => (VK_SNAPSHOT, true),
        s if s.len() == 1 => {
            let c = s.chars().next().unwrap();
            if c.is_ascii_alphabetic() {
                (c.to_ascii_uppercase() as u16, false)
            } else if c.is_ascii_digit() {
                (c as u16, false)
            } else {
                match c {
                    '+' | '=' => (0xBB, false),
                    '-' | '_' => (0xBD, false),
                    ',' | '<' => (0xBC, false),
                    '.' | '>' => (0xBE, false),
                    '/' | '?' => (0xBF, false),
                    ';' | ':' => (0xBA, false),
                    '[' | '{' => (0xDB, false),
                    '\\' | '|' => (0xDC, false),
                    ']' | '}' => (0xDD, false),
                    '\'' | '"' => (0xDE, false),
                    '`' | '~' => (0xC0, false),
                    _ => bail!("Unknown or unsupported key character: '{}'", c),
                }
            }
        }
        _ => bail!("Unknown key: '{}'", key),
    };

    Ok((vk, extended))
}

#[cfg(target_os = "windows")]
fn map_modifier_to_vk(modifier: &str) -> Result<(u16, bool)> {
    let mod_normalized = modifier.trim().to_ascii_lowercase();
    match mod_normalized.as_str() {
        "ctrl" | "control" => Ok((VK_CONTROL, false)),
        "alt" | "menu" => Ok((VK_MENU, false)),
        "shift" => Ok((VK_SHIFT, false)),
        "win" | "windows" | "super" | "meta" | "cmd" => Ok((VK_LWIN, true)),
        _ => bail!("Unsupported modifier: '{}'. Supported: ctrl, alt, shift, win", modifier),
    }
}

#[cfg(target_os = "windows")]
pub fn press_key(key: &str) -> Result<()> {
    let (vk, extended) = map_key_to_vk(key)?;
    let inputs = [
        create_key_input(vk, false, extended),
        create_key_input(vk, true, extended),
    ];
    send_inputs(&inputs)?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn hotkey(modifiers: &[&str], key: &str) -> Result<()> {
    let mut modifier_vks = Vec::new();
    for modifier in modifiers {
        let (vk, extended) = map_modifier_to_vk(modifier)?;
        modifier_vks.push((vk, extended));
    }

    let (key_vk, key_extended) = map_key_to_vk(key)?;
    let mut inputs = Vec::new();

    for &(vk, extended) in &modifier_vks {
        inputs.push(create_key_input(vk, false, extended));
    }

    inputs.push(create_key_input(key_vk, false, key_extended));
    inputs.push(create_key_input(key_vk, true, key_extended));

    for &(vk, extended) in modifier_vks.iter().rev() {
        inputs.push(create_key_input(vk, true, extended));
    }

    send_inputs(&inputs)?;
    Ok(())
}

// Windows tool handlers
#[cfg(target_os = "windows")]
fn move_mouse_windows(args: &Value) -> Result<ToolResult> {
    let target_x = args["x"].as_i64().context("Missing 'x'")? as i32;
    let target_y = args["y"].as_i64().context("Missing 'y'")? as i32;

    let mut cursor = POINT { x: 0, y: 0 };
    unsafe { GetCursorPos(&mut cursor) };

    let start_x = cursor.x;
    let start_y = cursor.y;
    
    let steps = 30;
    for i in 1..=steps {
        let t = i as f64 / steps as f64;
        let eased = if t < 0.5 {
            4.0 * t * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
        };
        
        let x = start_x + ((target_x - start_x) as f64 * eased) as i32;
        let y = start_y + ((target_y - start_y) as f64 * eased) as i32;
        
        unsafe { SetCursorPos(x, y) };
        thread::sleep(Duration::from_millis(8));
    }
    
    unsafe { SetCursorPos(target_x, target_y) };

    Ok(ToolResult {
        success: true,
        message: format!("Moved to ({}, {})", target_x, target_y),
        data: Some(serde_json::json!({ "x": target_x, "y": target_y })),
    })
}

#[cfg(target_os = "windows")]
fn get_screen_screenshot_windows() -> Result<ToolResult> {
    let screens = Screen::all().map_err(|e| anyhow::anyhow!("Failed to list screens: {}", e))?;
    let screen = screens.first().ok_or_else(|| anyhow::anyhow!("No screen found"))?;
    
    let image = screen.capture().map_err(|e| anyhow::anyhow!("Failed to capture screen: {}", e))?;
    
    let mut buffer = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(image).write_to(&mut buffer, ImageFormat::Png)
        .map_err(|e| anyhow::anyhow!("Failed to encode screenshot: {}", e))?;
    
    let b64 = general_purpose::STANDARD.encode(buffer.into_inner());
    
    Ok(ToolResult {
        success: true,
        message: "Screenshot captured".to_string(),
        data: Some(serde_json::json!({ "image": b64, "mime_type": "image/png" })),
    })
}


#[cfg(target_os = "windows")]
fn click_mouse_windows(args: &Value) -> Result<ToolResult> {
    let button = args["button"].as_str().unwrap_or("left");

    match click_mouse(button) {
        Ok(_) => Ok(ToolResult {
            success: true,
            message: format!("Clicked {} mouse button", button),
            data: None,
        }),
        Err(e) => Ok(ToolResult {
            success: false,
            message: format!("Failed to click mouse: {}", e),
            data: None,
        }),
    }
}

#[cfg(target_os = "windows")]
fn get_mouse_position_windows() -> Result<ToolResult> {
    match get_mouse_position() {
        Ok((x, y)) => Ok(ToolResult {
            success: true,
            message: format!("Mouse position is ({}, {})", x, y),
            data: Some(serde_json::json!({ "x": x, "y": y })),
        }),
        Err(e) => Ok(ToolResult {
            success: false,
            message: format!("Failed to get mouse position: {}", e),
            data: None,
        }),
    }
}

#[cfg(target_os = "windows")]
fn type_text_windows(args: &Value) -> Result<ToolResult> {
    let text = args["text"].as_str().context("Missing 'text' argument")?;

    match type_text(text) {
        Ok(_) => Ok(ToolResult {
            success: true,
            message: format!("Typed text ({} characters)", text.chars().count()),
            data: None,
        }),
        Err(e) => Ok(ToolResult {
            success: false,
            message: format!("Failed to type text: {}", e),
            data: None,
        }),
    }
}

#[cfg(target_os = "windows")]
fn press_key_windows(args: &Value) -> Result<ToolResult> {
    let key = args["key"].as_str().context("Missing 'key' argument")?;

    match press_key(key) {
        Ok(_) => Ok(ToolResult {
            success: true,
            message: format!("Pressed key: {}", key),
            data: None,
        }),
        Err(e) => Ok(ToolResult {
            success: false,
            message: format!("Failed to press key: {}", e),
            data: None,
        }),
    }
}

#[cfg(target_os = "windows")]
fn hotkey_windows(args: &Value) -> Result<ToolResult> {
    let modifiers = args["modifiers"]
        .as_array()
        .context("Missing 'modifiers' array")?
        .iter()
        .map(|v| v.as_str().context("Modifier must be string"))
        .collect::<Result<Vec<_>>>()?;

    let key = args["key"].as_str().context("Missing 'key' argument")?;

    match hotkey(&modifiers, key) {
        Ok(_) => Ok(ToolResult {
            success: true,
            message: format!("Executed hotkey: {}+{}", modifiers.join("+"), key),
            data: None,
        }),
        Err(e) => Ok(ToolResult {
            success: false,
            message: format!("Failed to execute hotkey: {}", e),
            data: None,
        }),
    }
}


#[cfg(target_os = "windows")]
fn scroll_windows(args: &Value) -> Result<ToolResult> {
    let amount = if let Some(a) = args["amount"].as_i64() {
        a as i32
    } else if let Some(d) = args["delta"].as_i64() {
        d as i32
    } else if let Some(dir) = args["direction"].as_str() {
        match dir.to_ascii_lowercase().as_str() {
            "up" => 1,
            "down" => -1,
            _ => bail!("Unknown scroll direction: '{}'. Use 'up' or 'down'", dir),
        }
    } else {
        bail!("Missing 'amount' or 'direction' argument for scroll");
    };

    let wheel_delta = if amount.abs() >= 120 {
        amount
    } else {
        amount * 120
    };

    let input = create_mouse_input(MOUSEEVENTF_WHEEL, wheel_delta as u32);
    send_inputs(&[input])?;

    Ok(ToolResult {
        success: true,
        message: format!("Scrolled by {} units (delta: {})", amount, wheel_delta),
        data: None,
    })
}

#[cfg(target_os = "windows")]
fn open_application_windows(args: &Value) -> Result<ToolResult> {
    let app = args["application"].as_str().context("Missing 'application' argument")?;

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

    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Ok(ToolResult {
            success: false,
            message: "Only http:// and https:// URLs are allowed".to_string(),
            data: None,
        });
    }

    let result = std::process::Command::new("rundll32")
        .args(["url.dll,FileProtocolHandler", url])
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
fn hotkey_macos(_args: &Value) -> Result<ToolResult> {
    Ok(ToolResult {
        success: false,
        message: "Hotkey control on macOS is not yet implemented".to_string(),
        data: None,
    })
}

#[cfg(target_os = "macos")]
fn scroll_macos(_args: &Value) -> Result<ToolResult> {
    Ok(ToolResult {
        success: false,
        message: "Scroll control on macOS is not yet implemented".to_string(),
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
