mod calendly;
mod default;
mod interview;
mod mapped;
mod one_to_one;
mod regular;

use crate::{config::Config, event::Event};
use anyhow::Result;
pub use calendly::CalendlyEventHandler;
use chrono::{DateTime, Utc};
use default::DefaultEventHandler;
use interview::InterviewEventHandler;
use mapped::MappedEventHandler;
use one_to_one::OneToOneEventHandler;
use regular::RegularEventHandler;

pub trait EventHandler {
    fn handle(&self, event: &Event) -> Option<String>;
}

/// default_handlers returns a list of all handlers, in optimum order.
pub fn default_handlers(config: &Config) -> Result<Vec<Box<dyn EventHandler>>> {
    let handlers: Vec<Box<dyn EventHandler>> = vec![
        RegularEventHandler::build(&config.regular_note_glob)?,
        MappedEventHandler::build(&config.mapped_filenames)?,
        InterviewEventHandler::build()?,
        OneToOneEventHandler::build(&config.user_preferred_name, &config.user_email)?,
        CalendlyEventHandler::build(&config.user_preferred_name)?,
        DefaultEventHandler::build()?,
    ];

    Ok(handlers)
}

/// linked_agenda_entry returns a markdown-formatted string for a linked agenda entry.
fn linked_agenda_entry(date: &DateTime<Utc>, name: &str, alias: &str) -> String {
    format!(
        "- **{}**: [[{}#{}|{}]]",
        date.format("%H%M"),
        name,
        date.format("%Y-%m-%d"),
        alias
    )
}

/// fs_note_list returns a list of note names from the filesystem.
fn fs_note_list(glob: &str) -> Result<Vec<String>> {
    let pattern = glob::glob(glob)?;

    let note_names = pattern
        .filter_map(|entry| {
            let path = match entry {
                Ok(path) => path,
                Err(_) => return None,
            };

            path.file_stem()
                .map(|file_stem| file_stem.to_string_lossy().to_string())
        })
        .collect();

    Ok(note_names)
}
