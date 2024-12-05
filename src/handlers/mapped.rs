use super::*;
use anyhow::Result;
use std::collections::HashMap;

/// MappedEventHandler is used for handling events where the name of the event is mapped to a note
/// with a different name on the filesystem
pub struct MappedEventHandler {
    notes: HashMap<String, String>,
}

impl MappedEventHandler {
    /// build creates a new MappedEventHandler from the given notes map.
    pub fn build(notes: &HashMap<String, String>) -> Result<Box<Self>> {
        Ok(Box::new(Self {
            notes: notes.clone(),
        }))
    }

    /// valid_for returns true if the event's name is in the notes map
    fn valid_for(&self, event: &Event) -> bool {
        self.notes.contains_key(&event.name)
    }
}

impl EventHandler for MappedEventHandler {
    /// handle returns the rendered event as a string.
    fn handle(&self, event: &Event) -> Option<String> {
        if !self.valid_for(event) {
            return None;
        }

        let event_name = self.notes.get(&event.name).unwrap_or(&event.name);

        Some(linked_agenda_entry(
            &event.start_time,
            event_name,
            event_name,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::prelude::*;

    #[test]
    fn test_mapped_event() {
        let notes = HashMap::from([("event1".to_string(), "note1".to_string())]);
        let handler = MappedEventHandler::build(&notes).unwrap();

        let event = Event {
            name: "event1".to_string(),
            start_time: Utc.with_ymd_and_hms(2024, 12, 5, 9, 00, 00).unwrap(),
            ..Default::default()
        };

        let result = handler.handle(&event).unwrap();

        assert_eq!(result, "- **0900**: [[note1#2024-12-05|note1]]");
    }

    #[test]
    fn test_unmapped_event() {
        let notes = HashMap::new();
        let handler = MappedEventHandler::build(&notes).unwrap();

        let event = Event {
            name: "event1".to_string(),
            ..Default::default()
        };

        let result = handler.handle(&event);

        assert!(result.is_none());
    }
}
