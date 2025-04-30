use super::*;
use anyhow::Result;

/// DefaultEventHandler is used for rendering events in the calendar that aren't excluded, but
/// don't match any other handlers.
pub struct DefaultEventHandler {}

impl DefaultEventHandler {
    /// build creates a new DefaultEventHandler.
    pub fn build() -> Result<Box<Self>> {
        Ok(Box::new(Self {}))
    }
}

impl EventHandler for DefaultEventHandler {
    /// handle returns the event as a string.
    fn handle(&self, event: &Event) -> Option<String> {
        Some(format!(
            "- **{}**: {}",
            &event.start_time.format("%H%M"),
            &event.name
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::prelude::*;

    #[test]
    fn test_default_handler() {
        let handler = DefaultEventHandler::build().unwrap();

        let start_time = Local.with_ymd_and_hms(2024, 12, 5, 9, 00, 00).unwrap();

        let event = Event {
            name: "This is some rando event".to_string(),
            start_time,
            ..Default::default()
        };

        assert_eq!(
            handler.handle(&event),
            Some("- **0900**: This is some rando event".to_string())
        );
    }
}
