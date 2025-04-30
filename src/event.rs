use chrono::{DateTime, Local};

use crate::config::Config;

#[derive(Debug, Clone, Default)]
pub struct Event {
    /// Start time of the event. For all day events, this is set to the Unix epoch.
    pub start_time: DateTime<Local>,
    /// Name of the event.
    pub name: String,
    /// Description of the event.
    pub description: String,
    /// Color index of the event in the calendar. This is a string representation of a number.
    pub color: String,
    /// List of attendee emails for the event, excluding the user.
    pub attendees: Vec<String>,
}

impl Event {
    /// Build a new event in the context of the current configuration.
    pub fn build(
        config: &Config,
        start: DateTime<chrono::Local>,
        name: String,
        description: String,
        color: String,
        attendees: Vec<String>,
    ) -> Self {
        // Include only attendees that aren't the user.
        let user_email = config.user_email.to_string();
        let attendees = attendees.into_iter().filter(|a| *a != user_email).collect();

        // Strip suffixes (such as '- Weekly') from event names as per the configuration.
        let mut name = name;
        for sfx in &config.strip_event_suffixes {
            name = name.trim_end_matches(sfx).to_string();
        }

        Self {
            start_time: start,
            name,
            description,
            color,
            attendees,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_email::Email;

    use super::*;

    #[test]
    fn test_event_build_remove_suffixes() {
        let config = Config {
            user_email: Email::from_str("user@example.com").unwrap(),
            strip_event_suffixes: vec![" - Weekly".to_string(), " - Monthly".to_string()],
            ..Default::default()
        };

        let event = Event::build(
            &config,
            Default::default(),
            "Team Meeting - Weekly".to_string(),
            Default::default(),
            Default::default(),
            vec!["colleague@example.com".to_string()],
        );

        // Event name should have the suffix stripped.
        assert_eq!(event.name, "Team Meeting");
    }

    #[test]
    fn test_event_build_remove_user_from_attendees() {
        let config = Config {
            user_email: Email::from_str("user@example.com").unwrap(),
            strip_event_suffixes: vec![" - Weekly".to_string(), " - Monthly".to_string()],
            ..Default::default()
        };

        let event = Event::build(
            &config,
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            vec![
                "user@example.com".to_string(),
                "colleague@example.com".to_string(),
            ],
        );

        // User email should be stripped from the attendees list.
        assert_eq!(event.attendees, vec!["colleague@example.com"]);
    }
}
