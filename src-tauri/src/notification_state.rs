use std::collections::HashMap;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationData {
    pub id: String,
    pub room_id: Option<String>,
    pub event_id: Option<String>,
    pub tag: Option<String>,
}

pub struct NotificationState {
    notifications: Mutex<HashMap<String, NotificationData>>,
}

impl NotificationState {
    pub fn new() -> Self {
        Self {
            notifications: Mutex::new(HashMap::new()),
        }
    }

    pub fn add(&self, data: NotificationData) {
        let mut notifications = self.notifications.lock().unwrap();
        notifications.insert(data.id.clone(), data);
    }

    pub fn get(&self, id: &str) -> Option<NotificationData> {
        let notifications = self.notifications.lock().unwrap();
        notifications.get(id).cloned()
    }

    pub fn remove(&self, id: &str) -> Option<NotificationData> {
        let mut notifications = self.notifications.lock().unwrap();
        notifications.remove(id)
    }

    pub fn get_by_room(&self, room_id: &str) -> Vec<NotificationData> {
        let notifications = self.notifications.lock().unwrap();
        notifications
            .values()
            .filter(|n| n.room_id.as_deref() == Some(room_id))
            .cloned()
            .collect()
    }

    pub fn clear(&self) {
        let mut notifications = self.notifications.lock().unwrap();
        notifications.clear();
    }
}

impl Default for NotificationState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_notification() {
        let state = NotificationState::new();
        let data = NotificationData {
            id: "notif1".to_string(),
            room_id: Some("!room:server.com".to_string()),
            event_id: Some("$event".to_string()),
            tag: Some("tag1".to_string()),
        };

        state.add(data.clone());
        let retrieved = state.get("notif1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().room_id, data.room_id);
    }

    #[test]
    fn test_remove_notification() {
        let state = NotificationState::new();
        let data = NotificationData {
            id: "notif1".to_string(),
            room_id: Some("!room:server.com".to_string()),
            event_id: None,
            tag: None,
        };

        state.add(data);
        state.remove("notif1");
        assert!(state.get("notif1").is_none());
    }

    #[test]
    fn test_get_by_room() {
        let state = NotificationState::new();
        state.add(NotificationData {
            id: "notif1".to_string(),
            room_id: Some("!room1:server.com".to_string()),
            event_id: None,
            tag: None,
        });
        state.add(NotificationData {
            id: "notif2".to_string(),
            room_id: Some("!room1:server.com".to_string()),
            event_id: None,
            tag: None,
        });
        state.add(NotificationData {
            id: "notif3".to_string(),
            room_id: Some("!room2:server.com".to_string()),
            event_id: None,
            tag: None,
        });

        let room1_notifs = state.get_by_room("!room1:server.com");
        assert_eq!(room1_notifs.len(), 2);
    }
}
