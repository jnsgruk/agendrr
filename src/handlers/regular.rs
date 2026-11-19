use super::*;
use anyhow::Result;

/// RegularEventHandler is used for handling regular meeting events that have named notes on the
/// filesystem.
pub struct RegularEventHandler {
    notes: Vec<String>,
    slug: bool,
}

impl RegularEventHandler {
    /// build creates a new RegularEventHandler, taking a glob that matches a list of named notes.
    pub fn build(glob: &str, slug: bool) -> Result<Box<Self>> {
        Ok(Box::new(Self {
            notes: fs_note_list(glob)?,
            slug,
        }))
    }

    /// note_for returns the name of the note matching the event's name, if there is one. When
    /// slugs are enabled, the slugified event name is also matched.
    fn note_for(&self, event: &Event) -> Option<String> {
        [event.name.clone(), note_name(&event.name, self.slug)]
            .into_iter()
            .find(|name| self.notes.contains(name))
    }
}

impl EventHandler for RegularEventHandler {
    /// handle returns the rendered event as a string.
    fn handle(&self, event: &Event) -> Option<String> {
        let note = self.note_for(event)?;

        Some(linked_agenda_entry(&event.start_time, &note, &event.name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::prelude::*;

    fn handler(slug: bool) -> RegularEventHandler {
        RegularEventHandler {
            notes: vec![
                "Some Event".to_string(),
                "Some Other Event".to_string(),
                "team-sync".to_string(),
            ],
            slug,
        }
    }

    fn event(name: &str) -> Event {
        Event {
            name: name.to_string(),
            start_time: Local.with_ymd_and_hms(2024, 12, 5, 9, 00, 00).unwrap(),
            ..Default::default()
        }
    }

    #[test]
    fn test_valid_regular_event() {
        let result = handler(false).handle(&event("Some Event")).unwrap();

        assert_eq!(result, "- **0900**: [[Some Event#2024-12-05|Some Event]]");
    }

    #[test]
    fn test_invalid_regular_event() {
        let result = handler(false).handle(&event("Jon / Joe - Weekly"));

        assert!(result.is_none());
    }

    #[test]
    fn test_slugged_note_requires_slug_option() {
        assert!(handler(false).handle(&event("Team Sync")).is_none());
    }

    #[test]
    fn test_valid_regular_event_slug() {
        let result = handler(true).handle(&event("Team Sync")).unwrap();

        assert_eq!(result, "- **0900**: [[team-sync#2024-12-05|Team Sync]]");
    }

    #[test]
    fn test_exact_match_preferred_with_slug() {
        let result = handler(true).handle(&event("Some Event")).unwrap();

        assert_eq!(result, "- **0900**: [[Some Event#2024-12-05|Some Event]]");
    }
}
