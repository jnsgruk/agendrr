use std::sync::LazyLock;

use crate::event::Event;
use anyhow::Result;
use regex::{Captures, Regex};

use super::*;

/// Matches event names created by Calendly.
static CALENDLY_EVENT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(.+) and Jon Seager").unwrap());

/// CalendlyEventHandler is used for handling Calendly events.
pub struct CalendlyEventHandler {
    user_name: String,
}

impl CalendlyEventHandler {
    /// build creates a new CalendlyEventHandler with the given user name.
    pub fn build(user_name: &str) -> Result<Box<Self>> {
        Ok(Box::new(Self {
            user_name: user_name.to_string(),
        }))
    }

    /// valid_for returns the captures if the event is a Calendly event.
    fn valid_for<'a>(&self, event: &'a Event) -> Option<Captures<'a>> {
        CALENDLY_EVENT_REGEX.captures(&event.name)
    }
}

impl EventHandler for CalendlyEventHandler {
    /// handle returns the formatted string for the event if it is a Calendly event.
    fn handle(&self, event: &Event) -> Option<String> {
        let captures = self.valid_for(event);
        // Bail early if there are not any matches from the regex
        captures.as_ref()?;

        if let Some(matches) = captures {
            // Extract the full name from the matched group in the regular expression
            let full_name = matches.extract::<1>().1[0];
            // Try to get just the first name for the alias, falling back to the full name
            let first_name = full_name.split_once(" ").map_or(full_name, |v| v.0);
            // Construct the alias from the user name and the first name
            let alias = format!("{}/{}", self.user_name, first_name);

            return Some(linked_agenda_entry(&event.start_time, full_name, &alias));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::prelude::*;

    #[test]
    fn test_handles_calendly_event() {
        let handler = CalendlyEventHandler::build("Jon").unwrap();

        let start_time = Utc.with_ymd_and_hms(2024, 12, 5, 9, 00, 00).unwrap();

        let event = Event {
            name: "John Smith and Jon Seager".to_string(),
            start_time,
            ..Default::default()
        };

        let result = handler.handle(&event).unwrap();

        assert_eq!(result, "- **0900**: [[John Smith#2024-12-05|Jon/John]]");
    }

    #[test]
    fn test_ignores_non_calendly_event() {
        let handler = CalendlyEventHandler::build("Jon").unwrap();
        let event = Event {
            name: "Regular Meeting".to_string(),
            start_time: Utc::now(),
            ..Default::default()
        };
        assert!(handler.handle(&event).is_none());
    }

    #[test]
    fn test_handles_single_name() {
        let handler = CalendlyEventHandler::build("Jon").unwrap();

        let start_time = Utc.with_ymd_and_hms(2024, 12, 5, 9, 00, 00).unwrap();

        let event = Event {
            name: "Mohammad and Jon Seager".to_string(),
            start_time,
            ..Default::default()
        };

        let result = handler.handle(&event).unwrap();
        assert_eq!(result, "- **0900**: [[Mohammad#2024-12-05|Jon/Mohammad]]");
    }
}
