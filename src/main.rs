mod clients;
mod config;
mod event;
mod filters;
mod handlers;

use anyhow::Result;
use clap::Parser;
use clients::{CalendarClient, GoogleCalendarClient};
use config::Config;
use event::Event;
use filters::default_filters;
use handlers::{EventHandler, default_handlers};

/// A command-line utility to generate a markdown summary of events from Google Calendar.
#[derive(Parser)]
#[command(version, about, long_about)]
struct Cli {
    /// Number of days forwards/backwards to fetch events for.
    #[arg(short, long, default_value = "0")]
    offset: i64,

    /// Path to the credentials file.
    #[arg(long, default_value = "credentials.json")]
    credentials: String,

    /// Path to the configuration file.
    #[arg(short, long, default_value = "agendrr.yaml")]
    config_file: String,

    /// Toggle debug output.
    #[arg(long, default_value = "false")]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Set up the application.
    let args = Cli::parse();
    let config = Config::build(args)?;

    // Build and authenticate the Google Calendar client.
    let client = GoogleCalendarClient::build(&config).await?;

    // Use the default filters and handlers to render the events.
    let filters = default_filters(&config)?;
    let handlers = default_handlers(&config)?;

    // Fetch a vector containing rendered events.
    let filtered_events: Vec<String> = client
        .events()
        .await?
        .into_iter()
        .filter_map(|e| {
            let include = !filters.iter().any(|f| f.exclude(&e));
            if include {
                render_event(&e, &handlers)
            } else {
                None
            }
        })
        .collect();

    // Print the rendered events.
    println!("{}", filtered_events.join("\n"));
    Ok(())
}

// render_event renders an event using the provided handlers.
fn render_event(event: &Event, handlers: &Vec<Box<dyn EventHandler>>) -> Option<String> {
    for h in handlers {
        let result = h.handle(event);
        if result.is_some() {
            return result;
        }
    }
    None
}
