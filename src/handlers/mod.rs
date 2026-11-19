mod calendly;
mod default;
mod interview;
mod mapped;
mod one_to_one;
mod regular;

use crate::{config::Config, event::Event};
use anyhow::Result;
pub use calendly::CalendlyEventHandler;
use chrono::{DateTime, Local};
use default::DefaultEventHandler;
use interview::InterviewEventHandler;
use mapped::MappedEventHandler;
use one_to_one::OneToOneEventHandler;
use regular::RegularEventHandler;
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

pub trait EventHandler {
    fn handle(&self, event: &Event) -> Option<String>;
}

/// default_handlers returns a list of all handlers, in optimum order.
pub fn default_handlers(config: &Config) -> Result<Vec<Box<dyn EventHandler>>> {
    let handlers: Vec<Box<dyn EventHandler>> = vec![
        RegularEventHandler::build(&config.regular_note_glob, config.slug_note_names)?,
        MappedEventHandler::build(&config.mapped_filenames, config.slug_note_names)?,
        InterviewEventHandler::build(config.slug_note_names)?,
        OneToOneEventHandler::build(
            &config.user_preferred_name,
            &config.user_email,
            config.slug_note_names,
        )?,
        CalendlyEventHandler::build(&config.user_preferred_name, config.slug_note_names)?,
        DefaultEventHandler::build()?,
    ];

    Ok(handlers)
}

/// linked_agenda_entry returns a markdown-formatted string for a linked agenda entry.
fn linked_agenda_entry(date: &DateTime<Local>, name: &str, alias: &str) -> String {
    format!(
        "- **{}**: [[{}#{}|{}]]",
        date.format("%H%M"),
        name,
        date.format("%Y-%m-%d"),
        alias
    )
}

/// slugify returns the lowercase, dash-separated ASCII form of a name, such that "José García"
/// becomes "jose-garcia". Accents are stripped and any other non-alphanumeric run becomes a dash.
fn slugify(name: &str) -> String {
    let dashed: String = name
        .nfkd()
        .filter(|c| !is_combining_mark(*c))
        .flat_map(char::to_lowercase)
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();

    dashed
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// note_name returns the note name to link to for a given name, slugified if requested.
fn note_name(name: &str, slug: bool) -> String {
    if slug {
        slugify(name)
    } else {
        name.to_string()
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Jane Doe"), "jane-doe");
        assert_eq!(slugify("José García"), "jose-garcia");
        assert_eq!(
            slugify("HR / Engineering Leadership"),
            "hr-engineering-leadership"
        );
        assert_eq!(slugify("Boot - Weekly"), "boot-weekly");
        assert_eq!(slugify("  O'Brien  "), "o-brien");
    }

    #[test]
    fn test_note_name() {
        assert_eq!(note_name("Jane Doe", false), "Jane Doe");
        assert_eq!(note_name("Jane Doe", true), "jane-doe");
    }
}
