use super::*;
use anyhow::Result;

/// RegularEventHandler is used for handling regular meeting events that have named notes on the
/// filesystem.
pub struct RegularEventHandler {
    notes: Vec<String>,
}

impl RegularEventHandler {
    /// build creates a new RegularEventHandler, taking a glob that matches a list of named notes.
    pub fn build(glob: &str) -> Result<Box<Self>> {
        Ok(Box::new(Self {
            notes: fs_note_list(glob)?,
        }))
    }

    /// valid_for returns true if the event's name is in the notes list.
    fn valid_for(&self, event: &Event) -> bool {
        self.notes.contains(&event.name)
    }
}

impl EventHandler for RegularEventHandler {
    /// handle returns the rendered event as a string.
    fn handle(&self, event: &Event) -> Option<String> {
        if !self.valid_for(event) {
            return None;
        }

        Some(linked_agenda_entry(
            &event.start_time,
            &event.name,
            &event.name,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::prelude::*;

    #[test]
    fn test_valid_regular_event() {
        let handler = RegularEventHandler {
            notes: vec!["Some Event".to_string(), "Some Other Event".to_string()],
        };

        let event = Event {
            name: "Some Event".to_string(),
            start_time: Utc.with_ymd_and_hms(2024, 12, 5, 9, 00, 00).unwrap(),
            ..Default::default()
        };

        let result = handler.handle(&event).unwrap();

        assert_eq!(result, "- **0900**: [[Some Event#2024-12-05|Some Event]]");
    }

    #[test]
    fn test_invalid_regular_event() {
        let handler = RegularEventHandler {
            notes: vec!["Some Event".to_string(), "Some Other Event".to_string()],
        };

        let event = Event {
            name: "Jon / Joe - Weekly".to_string(),
            start_time: Utc.with_ymd_and_hms(2024, 12, 5, 9, 00, 00).unwrap(),
            ..Default::default()
        };

        let result = handler.handle(&event);

        assert!(result.is_none());
    }
}
