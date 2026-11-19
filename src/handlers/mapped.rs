use super::*;
use anyhow::Result;
use std::collections::HashMap;

/// MappedEventHandler is used for handling events where the name of the event is mapped to a note
/// with a different name on the filesystem
pub struct MappedEventHandler {
    notes: HashMap<String, String>,
    slug: bool,
}

impl MappedEventHandler {
    /// build creates a new MappedEventHandler from the given notes map.
    pub fn build(notes: &HashMap<String, String>, slug: bool) -> Result<Box<Self>> {
        Ok(Box::new(Self {
            notes: notes.clone(),
            slug,
        }))
    }
}

impl EventHandler for MappedEventHandler {
    /// handle returns the rendered event as a string. When slugs are enabled, the mapped name is
    /// used as the link alias, and its slugified form as the link target.
    fn handle(&self, event: &Event) -> Option<String> {
        let mapped_name = self.notes.get(&event.name)?;

        Some(linked_agenda_entry(
            &event.start_time,
            &note_name(mapped_name, self.slug),
            mapped_name,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::prelude::*;

    fn event(name: &str) -> Event {
        Event {
            name: name.to_string(),
            start_time: Local.with_ymd_and_hms(2024, 12, 5, 9, 00, 00).unwrap(),
            ..Default::default()
        }
    }

    #[test]
    fn test_mapped_event() {
        let notes = HashMap::from([("event1".to_string(), "note1".to_string())]);
        let handler = MappedEventHandler::build(&notes, false).unwrap();

        let result = handler.handle(&event("event1")).unwrap();

        assert_eq!(result, "- **0900**: [[note1#2024-12-05|note1]]");
    }

    #[test]
    fn test_mapped_event_slug() {
        let notes = HashMap::from([(
            "HR / Engineering Leadership - 1/2".to_string(),
            "HR Engineering Leadership".to_string(),
        )]);
        let handler = MappedEventHandler::build(&notes, true).unwrap();

        let result = handler
            .handle(&event("HR / Engineering Leadership - 1/2"))
            .unwrap();

        assert_eq!(
            result,
            "- **0900**: [[hr-engineering-leadership#2024-12-05|HR Engineering Leadership]]"
        );
    }

    #[test]
    fn test_unmapped_event() {
        let notes = HashMap::new();
        let handler = MappedEventHandler::build(&notes, false).unwrap();

        let result = handler.handle(&event("event1"));

        assert!(result.is_none());
    }
}
