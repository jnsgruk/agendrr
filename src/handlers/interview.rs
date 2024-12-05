use super::*;

use crate::event::Event;
use anyhow::Result;
use chrono::{DateTime, Utc};
use regex::Regex;
use std::sync::LazyLock;

/// Matches event names created by the Canonical auto-scheduler.
static SCHEDULER_EVENT_NAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^Please interview a candidate for .+$").unwrap());

/// Matches the description of events created by the Canonical auto-scheduler.
static SCHEDULER_EVENT_DESCRIPTION_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^Please interview (.+)[.]{1}$").unwrap());

/// Matches event names created by the Greenhouse scheduler.
static GREENHOUSE_EVENT_NAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^Please interview (.+) for .+$").unwrap());

/// InterviewEventHandler is used for handling interview events.
pub struct InterviewEventHandler {}

impl InterviewEventHandler {
    pub fn build() -> Result<Box<Self>> {
        Ok(Box::new(Self {}))
    }

    // valid_for returns true if the event is an interview event.
    fn valid_for(&self, event: &Event) -> bool {
        event
            .attendees
            .contains(&"schedule@rose.greenhouse.io".to_string())
    }
}

impl EventHandler for InterviewEventHandler {
    // handle returns the formatted string for the event if it is an interview event.
    fn handle(&self, event: &Event) -> Option<String> {
        if !self.valid_for(event) {
            return None;
        }

        // Process events handled by the Canonical auto-scheduler
        if SCHEDULER_EVENT_NAME_REGEX.is_match(&event.name) {
            let desc_matches = SCHEDULER_EVENT_DESCRIPTION_REGEX.captures(&event.description);
            let matches = match desc_matches {
                Some(matches) => matches,
                None => return None,
            };

            let name = matches.extract::<1>().1[0];
            let candidate_file_name = name.to_ascii_lowercase().replace(" ", "-");
            let date = event.start_time.format("%Y%m%d%H%M");
            let filename = format!("{}-{}", date, candidate_file_name);

            return Some(interview_agenda_entry(&event.start_time, &filename, name));
        }

        // Process events handled by the Greenhouse scheduler
        let matches = match GREENHOUSE_EVENT_NAME_REGEX.captures(&event.name) {
            Some(matches) => matches,
            None => return None,
        };

        let name = matches.extract::<1>().1[0];
        let candidate_file_name = name.to_ascii_lowercase().replace(" ", "-");
        let date = event.start_time.format("%Y%m%d%H%M");
        let filename = format!("{}-{}", date, candidate_file_name);

        Some(interview_agenda_entry(&event.start_time, &filename, name))
    }
}

/// interview_agenda_entry returns the formatted string for the interview event.
fn interview_agenda_entry(date: &DateTime<Utc>, filename: &str, candidate_name: &str) -> String {
    format!(
        "- **{}**: [[{}|{} Interview Notes]]",
        date.format("%H%M"),
        filename,
        candidate_name
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::prelude::*;

    fn create_event(name: &str, description: &str, attendees: Vec<&str>) -> Event {
        let start_time = Utc.with_ymd_and_hms(2024, 12, 5, 9, 00, 00).unwrap();

        Event {
            name: name.to_string(),
            description: description.to_string(),
            start_time,
            attendees: attendees.into_iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        }
    }

    #[test]
    fn test_handle_scheduler_event() {
        let handler = InterviewEventHandler::build().unwrap();

        let event = create_event(
            "Please interview a candidate for Software Engineer",
            "Please interview John Doe.",
            vec!["schedule@rose.greenhouse.io"],
        );

        let result = handler.handle(&event);
        assert_eq!(
            result,
            Some("- **0900**: [[202412050900-john-doe|John Doe Interview Notes]]".to_string())
        );
    }

    #[test]
    fn test_handle_greenhouse_event() {
        let handler = InterviewEventHandler::build().unwrap();

        let event = create_event(
            "Please interview John Doe for Software Engineer",
            "",
            vec!["schedule@rose.greenhouse.io"],
        );

        let result = handler.handle(&event);

        assert_eq!(
            result,
            Some("- **0900**: [[202412050900-john-doe|John Doe Interview Notes]]".to_string())
        );
    }

    #[test]
    fn test_handle_invalid_event() {
        let handler = InterviewEventHandler::build().unwrap();
        let event = create_event("Some other event", "", vec!["schedule@rose.greenhouse.io"]);
        let result = handler.handle(&event);
        assert_eq!(result, None);
    }
}
