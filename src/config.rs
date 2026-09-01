use anyhow::{Result, bail};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_email::Email;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::Cli;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    /// The day offset specifies how many days ahead/behind to fetch events.
    #[serde(default)]
    pub day_offset: i64,

    /// The path to the calendar credentials file (usually credentials.json).
    #[serde(default)]
    pub credentials_path: PathBuf,

    /// The ID of the calendar to fetch events from.
    pub calendar_id: String,

    /// The email address of the user.
    pub user_email: Email,

    /// The first name / preferred name of the user.
    pub user_preferred_name: String,

    /// The glob pattern for "regular meeting" notes on the filesystem.
    #[serde(default)]
    pub regular_note_glob: String,

    /// A list of suffixes to strip from event names. For example, "- Weekly" or "- Monthly".
    #[serde(default)]
    pub strip_event_suffixes: Vec<String>,

    /// A list of colours to ignore (where colour is the event colour on the calendar).
    #[serde(default)]
    pub ignored_colours: Vec<String>,

    /// A list of regexes to ignore, matching on event names.
    #[serde(with = "serde_regex")]
    #[serde(default)]
    pub ignored_regex: Vec<Regex>,

    /// A map of Event Name -> Note Name for events with odd names.
    #[serde(default)]
    pub mapped_filenames: HashMap<String, String>,

    /// A boolean to toggle debug output.
    #[serde(default)]
    pub debug: bool,
}

impl Config {
    /// Construct the configuration from the filesystem and CLI arguments.
    pub fn build(args: Cli) -> Result<Self> {
        // Check if the config file specified exists.
        if !Path::new(&args.config_file).exists() {
            bail!("config file does not exist: {:?}", args.config_file);
        }

        // Load the configuration from the filesystem.
        let mut cfg: Config = confy::load_path(args.config_file)?;
        // Set the runtime offset from the CLI arguments.
        cfg.day_offset = args.offset;
        cfg.debug = args.debug;

        Ok(Self {
            credentials_path: PathBuf::from(args.credentials),
            day_offset: cfg.day_offset,
            calendar_id: cfg.calendar_id,
            user_email: cfg.user_email,
            user_preferred_name: cfg.user_preferred_name,
            regular_note_glob: cfg.regular_note_glob,
            strip_event_suffixes: cfg.strip_event_suffixes,
            ignored_colours: cfg.ignored_colours,
            ignored_regex: cfg.ignored_regex,
            mapped_filenames: cfg.mapped_filenames,
            debug: cfg.debug,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn config_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("agendrr-{name}-{}.yaml", std::process::id()))
    }

    fn args(config_file: &Path) -> Cli {
        Cli {
            offset: 0,
            credentials: "credentials.json".to_string(),
            config_file: config_file.display().to_string(),
            debug: false,
        }
    }

    #[test]
    fn loads_valid_user_email() {
        let path = config_path("valid-email");
        fs::write(
            &path,
            "calendar-id: calendar\nuser-email: user@example.com\nuser-preferred-name: User\n",
        )
        .unwrap();

        let config = Config::build(args(&path)).unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(config.user_email, Email::new("user@example.com").unwrap());
    }

    #[test]
    fn rejects_invalid_user_email() {
        let path = config_path("invalid-email");
        fs::write(
            &path,
            "calendar-id: calendar\nuser-email: not-an-email\nuser-preferred-name: User\n",
        )
        .unwrap();

        let result = Config::build(args(&path));
        fs::remove_file(path).unwrap();

        assert!(result.is_err());
    }
}
