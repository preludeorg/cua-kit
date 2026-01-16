//! Direct Claude API client
//!
//! This module provides direct HTTP access to the Claude API
//! without requiring the CLI wrapper.

use crate::ClaudeResult;

// Reserved for future TLS implementation
#[allow(dead_code)]
const API_HOST: &str = "api.anthropic.com";
#[allow(dead_code)]
const API_PORT: u16 = 443;
#[allow(dead_code)]
const API_VERSION: &str = "2023-06-01";
const MODEL: &str = "claude-sonnet-4-20250514";

/// Execute Claude via direct API call
pub fn execute_claude_api(
    _prompt: &str,
    _session_id: Option<&str>,
    _api_key: &str,
) -> ClaudeResult {
    // For a full implementation, we would need:
    // 1. TLS support (rustls or native-tls)
    // 2. HTTP client implementation
    // 3. Conversation history management for sessions

    // Since we want to keep dependencies minimal for BOF compatibility,
    // we'll use the Windows native HTTP APIs in a future version.

    // For now, provide a stub that returns an informative error
    ClaudeResult::error(
        "Direct API mode not yet implemented. Use wrapper mode (-w) or set up claude in PATH."
            .to_string(),
    )
}

/// Build the API request body
#[allow(dead_code)]
fn build_request_body(prompt: &str, _session_id: Option<&str>) -> String {
    // Simple request body for Claude API
    let body = serde_json::json!({
        "model": MODEL,
        "max_tokens": 4096,
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ]
    });

    body.to_string()
}

/// Parse API response
#[allow(dead_code)]
fn parse_api_response(response: &str) -> ClaudeResult {
    match serde_json::from_str::<serde_json::Value>(response) {
        Ok(json) => {
            // Check for error response
            if let Some(error) = json.get("error") {
                let message = error
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown API error");
                return ClaudeResult::error(message.to_string());
            }

            // Extract content from successful response
            let content = json
                .get("content")
                .and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|item| item.get("text"))
                .and_then(|t| t.as_str())
                .unwrap_or("");

            let id = json
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            ClaudeResult::new(id, content.to_string(), false)
        }
        Err(e) => ClaudeResult::error(format!("Failed to parse API response: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_request_body() {
        let body = build_request_body("Hello", None);
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json["model"], MODEL);
        assert_eq!(json["messages"][0]["content"], "Hello");
    }

    #[test]
    fn test_parse_api_response_success() {
        let response = r#"{
            "id": "msg_123",
            "content": [{"type": "text", "text": "Hello!"}],
            "model": "claude-sonnet-4-20250514"
        }"#;
        let result = parse_api_response(response);
        assert_eq!(result.session_id, "msg_123");
        assert_eq!(result.result, "Hello!");
        assert!(!result.is_error);
    }

    #[test]
    fn test_parse_api_response_error() {
        let response = r#"{"error": {"message": "Invalid API key"}}"#;
        let result = parse_api_response(response);
        assert!(result.is_error);
        assert!(result.result.contains("Invalid API key"));
    }
}
