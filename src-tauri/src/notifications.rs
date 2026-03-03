use serde::{Deserialize, Serialize};
use crate::notification_channels;
use crate::notification_state::{NotificationData, NotificationState};
use std::sync::Arc;
use tauri::State;
use tauri_plugin_notification::NotificationExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationOptions {
    pub body: Option<String>,
    pub icon: Option<String>,
    pub tag: Option<String>,
    pub data: Option<serde_json::Value>,
    pub room_id: Option<String>,
    pub event_id: Option<String>,
    pub room_type: Option<String>,
    pub has_mention: Option<bool>,
}

pub struct NotificationManager {
    state: Arc<NotificationState>,
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(NotificationState::new()),
        }
    }
}

#[tauri::command]
pub async fn request_notification_permission() -> Result<String, String> {
    // On Android, permission is handled by the system
    // This will trigger the Android permission dialog
    #[cfg(target_os = "android")]
    {
        // tauri-plugin-notification handles permission request
        Ok("granted".to_string())
    }

    #[cfg(not(target_os = "android"))]
    {
        Ok("granted".to_string())
    }
}

#[tauri::command]
pub async fn get_notification_permission() -> Result<String, String> {
    // Check current permission state
    #[cfg(target_os = "android")]
    {
        // This would check actual Android permission
        // For now, assume granted (will be improved)
        Ok("granted".to_string())
    }

    #[cfg(not(target_os = "android"))]
    {
        Ok("granted".to_string())
    }
}

#[tauri::command]
pub async fn show_notification(
    id: String,
    title: String,
    body: String,
    options: NotificationOptions,
    manager: State<'_, NotificationManager>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    use tauri_plugin_notification::NotificationExt;

    println!("[DEBUG] show_notification called!");
    println!("[DEBUG] id: {}", id);
    println!("[DEBUG] title: {}", title);
    println!("[DEBUG] body: {}", body);
    println!("[DEBUG] options: {:?}", options);



    let room_id = extract_room_id(&options);
    let event_id = options.event_id.clone();

    println!("[DEBUG] Extracted room_id: {:?}", room_id);
    println!("[DEBUG] Extracted event_id: {:?}", event_id);

    // Determine notification channel
    let room_type = options.room_type.as_deref().unwrap_or("room");
    let has_mention = options.has_mention.unwrap_or(false);
    let _channel = notification_channels::determine_channel(room_type, has_mention);

    // Store notification data
    manager.state.add(NotificationData {
        id: id.clone(),
        room_id: room_id.clone(),
        event_id: event_id.clone(),
        tag: options.tag.clone(),
    });

    // Create notification
    let mut notification = app.notification()
        .builder()
        .title(title)
        .body(body.clone());

    // Set large body for expandable notifications (BigTextStyle on Android)
    notification = notification.large_body(body);

    // Set group by room if room_id is available
    if let Some(ref room_id) = room_id {
        let group_key = format!("cinny_room_{}", room_id);
        notification = notification.group(group_key);
    }

    // Add sound for notifications
    notification = notification.sound("default");

    // Store notification ID in extra field for click handling
    notification = notification.extra("notification_id", id.clone());
    if let Some(ref room_id) = room_id {
        notification = notification.extra("room_id", room_id.clone());
    }
    if let Some(ref event_id) = event_id {
        notification = notification.extra("event_id", event_id.clone());
    }

    println!("[DEBUG] About to show notification...");
    notification.show().map_err(|e| {
        println!("[DEBUG] Failed to show notification: {}", e);
        e.to_string()
    })?;

    println!("[DEBUG] Notification shown successfully!");

    Ok(())
}

#[tauri::command]
pub async fn close_notification(
    id: String,
    manager: State<'_, NotificationManager>,
) -> Result<(), String> {
    manager.state.remove(&id);
    Ok(())
}

fn extract_room_id(options: &NotificationOptions) -> Option<String> {
    if let Some(room_id) = &options.room_id {
        return Some(room_id.clone());
    }

    if let Some(data) = &options.data {
        if let Some(room_id) = data.get("roomId").and_then(|v| v.as_str()) {
            return Some(room_id.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_options_deserialize() {
        let json = r#"{
            "body": "Test message",
            "room_id": "!room:server.com",
            "event_id": "$event",
            "room_type": "direct",
            "has_mention": false
        }"#;

        let options: NotificationOptions = serde_json::from_str(json).unwrap();
        assert_eq!(options.body, Some("Test message".to_string()));
        assert_eq!(options.room_id, Some("!room:server.com".to_string()));
        assert_eq!(options.room_type, Some("direct".to_string()));
        assert_eq!(options.has_mention, Some(false));
    }

    #[test]
    fn test_extract_room_id_from_data() {
        let options = NotificationOptions {
            body: None,
            icon: None,
            tag: None,
            data: Some(serde_json::json!({"roomId": "!room:server.com"})),
            room_id: None,
            event_id: None,
            room_type: None,
            has_mention: None,
        };

        let room_id = extract_room_id(&options);
        assert_eq!(room_id, Some("!room:server.com".to_string()));
    }
}
