use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushPayload {
    pub room_id: Option<String>,
    pub event_id: Option<String>,
    pub sender: Option<String>,
    pub sender_display_name: Option<String>,
    pub room_name: Option<String>,
    pub content: Option<serde_json::Value>,
    pub counts: Option<serde_json::Value>,
}

#[tauri::command]
pub async fn register_unifiedpush() -> Result<String, String> {
    // Stub implementation - will be completed in future task
    println!("UnifiedPush registration requested (stub)");
    Ok("UnifiedPush registration not yet implemented".to_string())
}

#[tauri::command]
pub async fn unregister_unifiedpush() -> Result<(), String> {
    // Stub implementation - will be completed in future task
    println!("UnifiedPush unregistration requested (stub)");
    Ok(())
}

pub fn parse_push_payload(payload_str: &str) -> Result<PushPayload, String> {
    serde_json::from_str(payload_str)
        .map_err(|e| format!("Failed to parse push payload: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_payload() {
        let json = r#"{
            "room_id": "!room:server.com",
            "event_id": "$event",
            "sender": "@alice:server.com",
            "sender_display_name": "Alice",
            "room_name": "General",
            "content": {"body": "Hello"},
            "counts": {"unread": 5}
        }"#;

        let payload = parse_push_payload(json).unwrap();
        assert_eq!(payload.room_id, Some("!room:server.com".to_string()));
        assert_eq!(payload.sender, Some("@alice:server.com".to_string()));
    }

    #[test]
    fn test_parse_minimal_payload() {
        let json = r#"{
            "room_id": "!room:server.com"
        }"#;

        let payload = parse_push_payload(json).unwrap();
        assert_eq!(payload.room_id, Some("!room:server.com".to_string()));
        assert_eq!(payload.sender, None);
    }
}
