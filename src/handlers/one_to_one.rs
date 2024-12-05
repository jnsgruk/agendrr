use super::*;
use anyhow::{bail, Context, Result};
use inflector::Inflector;
use serde_email::Email;

/// OneToOneEventHandler is an EventHandler that generates an agenda entry for a one-to-one meeting.
pub struct OneToOneEventHandler {
    user_email: Email,
    user_first_name: String,
}

impl OneToOneEventHandler {
    /// build creates a new OneToOneEventHandler with the given user name and email.
    pub fn build(user_first_name: &str, user_email: &Email) -> Result<Box<Self>> {
        Ok(Box::new(Self {
            user_email: user_email.clone(),
            user_first_name: user_first_name.to_string(),
        }))
    }

    /// valid_for returns true if the event is a one-to-one meeting.
    fn valid_for(&self, event: &Event) -> bool {
        event.attendees.len() == 1
    }

    /// parse_name_from_email extracts the first and full name from an email address.
    fn parse_name_from_email(&self, email: &str) -> Result<(String, String)> {
        let user_email = self.user_email.to_string();

        let Some(home_domain) = user_email.split("@").last() else {
            bail!("failed to extract home domain from user email");
        };

        // Parse the email and get the String representation
        let email = Email::from_str(email)?.to_string();

        let Some((local_part, domain)) = email.split_once("@") else {
            bail!("failed to extract local and domain parts from email");
        };

        if domain != home_domain {
            bail!("refusing to parse external email address for name");
        }

        let (first_name, last_name) = local_part
            .split_once('.')
            .context("failed to extract first/last name from email")?;

        Ok((
            first_name.to_title_case().to_string(),
            format!("{} {}", first_name, last_name).to_title_case(),
        ))
    }
}

impl EventHandler for OneToOneEventHandler {
    /// handle returns the formatted string for the event if it is a one-to-one meeting.
    fn handle(&self, event: &Event) -> Option<String> {
        if !self.valid_for(event) {
            return None;
        }

        let result = match event.attendees.first() {
            Some(email) => self.parse_name_from_email(email),
            None => return None,
        };

        match result {
            Ok((first_name, full_name)) => {
                let alias = format!("{}/{}", self.user_first_name, first_name);
                Some(linked_agenda_entry(&event.start_time, &full_name, &alias))
            }
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::prelude::*;

    fn one_to_one_handler() -> Box<OneToOneEventHandler> {
        OneToOneEventHandler::build("John", &Email::from_str("john.doe@example.com").unwrap())
            .unwrap()
    }

    #[test]
    fn test_valid_one_to_one() {
        let handler = one_to_one_handler();

        let event = Event {
            attendees: vec!["jane.doe@example.com".to_string()],
            start_time: Utc.with_ymd_and_hms(2024, 12, 5, 9, 00, 00).unwrap(),
            ..Default::default()
        };

        let result = handler.handle(&event);

        assert_eq!(
            result,
            Some("- **0900**: [[Jane Doe#2024-12-05|John/Jane]]".to_string())
        );
    }

    #[test]
    fn test_invalid_one_to_one() {
        let handler = one_to_one_handler();

        let event = Event {
            attendees: vec![
                "jane.doe@example.com".to_string(),
                "joe.bloggs@example.com".to_string(),
            ],
            start_time: Utc.with_ymd_and_hms(2024, 12, 5, 9, 00, 00).unwrap(),
            ..Default::default()
        };

        assert!(!handler.valid_for(&event));
    }

    #[test]
    fn test_parse_name_from_email() {
        let handler = one_to_one_handler();

        let result = handler.parse_name_from_email("jane.doe@example.com");
        let (first_name, full_name) = result.unwrap();

        assert_eq!(first_name, "Jane");
        assert_eq!(full_name, "Jane Doe");
    }

    #[test]
    fn test_parse_name_from_external_email() {
        let handler = one_to_one_handler();
        let result = handler.parse_name_from_email("jane.doe@another.com");
        assert!(result.is_err());
    }
}
