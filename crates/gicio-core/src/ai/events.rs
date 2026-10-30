//! Turns Claude `stream-json` lines into short human-readable activity lines.

use serde_json::Value;

pub enum ClaudeEvent {
    Log(String),
    Result(Result<String, String>),
    Nothing,
}

fn short(s: &str, n: usize) -> String {
    let s = s.trim().replace('\n', " ");
    if s.chars().count() > n { format!("{}…", s.chars().take(n).collect::<String>()) } else { s }
}

fn tool_line(name: &str, input: &Value) -> String {
    let arg = ["file_path", "path", "command", "pattern", "skill", "description"]
        .iter()
        .find_map(|k| input.get(*k).and_then(Value::as_str))
        .unwrap_or("");
    format!("🔧 {name} {}", short(arg, 140))
}

pub fn parse_claude(line: &str) -> ClaudeEvent {
    let Ok(v) = serde_json::from_str::<Value>(line) else { return ClaudeEvent::Log(short(line, 300)) };
    match v.get("type").and_then(Value::as_str) {
        Some("assistant") => {
            let parts: Vec<String> = v
                .pointer("/message/content")
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|c| match c.get("type").and_then(Value::as_str) {
                            Some("text") => c.get("text").and_then(Value::as_str).map(|t| format!("💬 {}", short(t, 300))),
                            Some("tool_use") => Some(tool_line(c.get("name").and_then(Value::as_str).unwrap_or("tool"), c.get("input").unwrap_or(&Value::Null))),
                            Some("thinking") => Some("💭 thinking…".to_string()),
                            _ => None,
                        })
                        .collect()
                })
                .unwrap_or_default();
            if parts.is_empty() { ClaudeEvent::Nothing } else { ClaudeEvent::Log(parts.join("\n")) }
        }
        Some("result") => {
            let text = v.get("result").and_then(Value::as_str).unwrap_or_default().to_string();
            if v.get("is_error").and_then(Value::as_bool) == Some(true) { ClaudeEvent::Result(Err(text)) } else { ClaudeEvent::Result(Ok(text)) }
        }
        Some("system") if v.get("subtype").and_then(Value::as_str) == Some("init") => {
            ClaudeEvent::Log(format!("▶ started ({})", v.get("model").and_then(Value::as_str).unwrap_or("claude")))
        }
        _ => ClaudeEvent::Nothing,
    }
}
