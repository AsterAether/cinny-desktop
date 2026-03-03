/// Determines the notification channel based on room type and mention status
pub fn determine_channel(room_type: &str, has_mention: bool) -> &'static str {
    if has_mention {
        return "mentions";
    }
    match room_type {
        "direct" => "direct_messages",
        "invite" => "invites",
        _ => "room_messages",
    }
}

/// Notification channel definitions
pub struct NotificationChannel {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub importance: &'static str,
}

pub const CHANNELS: &[NotificationChannel] = &[
    NotificationChannel {
        id: "direct_messages",
        name: "Direct Messages",
        description: "Notifications for direct messages",
        importance: "high",
    },
    NotificationChannel {
        id: "mentions",
        name: "Mentions",
        description: "Notifications when you are mentioned",
        importance: "high",
    },
    NotificationChannel {
        id: "room_messages",
        name: "Room Messages",
        description: "Notifications for room messages",
        importance: "default",
    },
    NotificationChannel {
        id: "invites",
        name: "Room Invites",
        description: "Notifications for room invitations",
        importance: "high",
    },
    NotificationChannel {
        id: "other",
        name: "Other",
        description: "Other notifications",
        importance: "low",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_channel_with_mention() {
        let channel = determine_channel("room", true);
        assert_eq!(channel, "mentions");
    }

    #[test]
    fn test_determine_channel_direct_message() {
        let channel = determine_channel("direct", false);
        assert_eq!(channel, "direct_messages");
    }

    #[test]
    fn test_determine_channel_invite() {
        let channel = determine_channel("invite", false);
        assert_eq!(channel, "invites");
    }

    #[test]
    fn test_determine_channel_regular_room() {
        let channel = determine_channel("room", false);
        assert_eq!(channel, "room_messages");
    }
}
