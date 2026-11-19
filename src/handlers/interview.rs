use super::*;

use crate::event::Event;
use anyhow::Result;
use chrono::{DateTime, Local};
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
pub struct InterviewEventHandler {
    slug: bool,
}

impl InterviewEventHandler {
    pub fn build(slug: bool) -> Result<Box<Self>> {
        Ok(Box::new(Self { slug }))
    }

    /// agenda_entry returns the agenda entry for an interview with the named candidate.
    fn agenda_entry(&self, date: &DateTime<Local>, name: &str) -> String {
        let candidate_file_name = if self.slug {
            slugify(name)
        } else {
            name.to_ascii_lowercase().replace(" ", "-")
        };
        let filename = format!("{}-{}", date.format("%Y%m%d%H%M"), candidate_file_name);

        interview_agenda_entry(date, &filename, &title_case_if_uppercase(name))
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
            return Some(self.agenda_entry(&event.start_time, name));
        }

        // Process events handled by the Greenhouse scheduler
        let matches = match GREENHOUSE_EVENT_NAME_REGEX.captures(&event.name) {
            Some(matches) => matches,
            None => return None,
        };

        let name = matches.extract::<1>().1[0];
        Some(self.agenda_entry(&event.start_time, name))
    }
}

/// title_case_if_uppercase title-cases a name given entirely in capitals, such that "JANE O'NEIL"
/// becomes "Jane O'Neil". Names using any lowercase letters are returned unchanged.
fn title_case_if_uppercase(name: &str) -> String {
    let mut letters = name.chars().filter(|c| c.is_alphabetic()).peekable();
    if letters.peek().is_none() || !letters.all(char::is_uppercase) {
        return name.to_string();
    }

    let mut previous_is_letter = false;
    name.chars()
        .flat_map(|c| {
            let titled: Vec<char> = if previous_is_letter {
                c.to_lowercase().collect()
            } else {
                c.to_uppercase().collect()
            };
            previous_is_letter = c.is_alphabetic();
            titled
        })
        .collect()
}

/// interview_agenda_entry returns the formatted string for the interview event.
fn interview_agenda_entry(date: &DateTime<Local>, filename: &str, candidate_name: &str) -> String {
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
        let start_time = Local.with_ymd_and_hms(2024, 12, 5, 9, 00, 00).unwrap();

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
        let handler = InterviewEventHandler::build(false).unwrap();

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
        let handler = InterviewEventHandler::build(false).unwrap();

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
        let handler = InterviewEventHandler::build(false).unwrap();
        let event = create_event("Some other event", "", vec!["schedule@rose.greenhouse.io"]);
        let result = handler.handle(&event);
        assert_eq!(result, None);
    }

    #[test]
    fn test_handle_greenhouse_event_slug() {
        let handler = InterviewEventHandler::build(true).unwrap();

        let event = create_event(
            "Please interview José O'Neil for Software Engineer",
            "",
            vec!["schedule@rose.greenhouse.io"],
        );

        assert_eq!(
            handler.handle(&event),
            Some(
                "- **0900**: [[202412050900-jose-o-neil|José O'Neil Interview Notes]]".to_string()
            )
        );
    }

    #[test]
    fn test_handle_uppercase_candidate_name() {
        let handler = InterviewEventHandler::build(false).unwrap();

        let event = create_event(
            "Please interview JOHN DOE for Software Engineer",
            "",
            vec!["schedule@rose.greenhouse.io"],
        );

        assert_eq!(
            handler.handle(&event),
            Some("- **0900**: [[202412050900-john-doe|John Doe Interview Notes]]".to_string())
        );
    }

    #[test]
    fn test_title_case_if_uppercase() {
        assert_eq!(title_case_if_uppercase("JANE O'NEIL"), "Jane O'Neil");
        assert_eq!(title_case_if_uppercase("Jane McDonald"), "Jane McDonald");
        assert_eq!(title_case_if_uppercase("JANE McDONALD"), "JANE McDONALD");
        assert_eq!(title_case_if_uppercase(""), "");
    }
}
