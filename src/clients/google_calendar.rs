use super::*;

use anyhow::{Context, Result};
use chrono::{Duration, NaiveTime, Utc};
use google_calendar3::api::Event as GCalEvent;
use google_calendar3::hyper_rustls::HttpsConnector;
use google_calendar3::hyper_util::client::legacy::connect::HttpConnector;
use google_calendar3::{hyper_rustls, hyper_util, yup_oauth2, CalendarHub};

use crate::config::Config;
use crate::event::Event;

/// GCalHub is a type alias for the Google Calendar API client.
type GCalHub = CalendarHub<HttpsConnector<HttpConnector>>;

/// GoogleCalendarClient is a client for the Google Calendar API.
pub struct GoogleCalendarClient {
    config: Config,
    hub: GCalHub,
}

impl GoogleCalendarClient {
    /// build creates a new GoogleCalendarClient from the given Config.
    pub async fn build(config: &Config) -> Result<Self> {
        let calendar_hub = Self::auth(config).await?;

        let client = Self {
            config: config.to_owned(),
            hub: calendar_hub,
        };

        Ok(client)
    }

    /// auth authenticated with the Google API and returns an authenticated "hub" object".
    async fn auth(config: &Config) -> Result<GCalHub> {
        let secret = yup_oauth2::read_application_secret(config.credentials_path.clone())
            .await
            .with_context(|| {
                format!(
                    "failed to read supplied credentials file: {}",
                    &config.credentials_path.display(),
                )
            })?;

        let token_storage_path = xdg::BaseDirectories::new()?
            .place_config_file("agendrr/token.json")
            .with_context(|| {
                "failed to cache tokens file at path: $XDG_CONFIG_HOME/agendrr/token.json".to_string()
            })?;

        let auth = yup_oauth2::InstalledFlowAuthenticator::builder(
            secret,
            yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
        )
        .persist_tokens_to_disk(token_storage_path)
        .build()
        .await?;

        let client =
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                .build(
                    hyper_rustls::HttpsConnectorBuilder::new()
                        .with_native_roots()?
                        .https_or_http()
                        .enable_http1()
                        .build(),
                );

        let calendar_hub = CalendarHub::new(client, auth);
        Ok(calendar_hub)
    }

    /// build_agenda_event creates an Event from a Google Calendar event.
    fn build_agenda_event(&self, event: GCalEvent) -> Event {
        let start = event.start.and_then(|s| s.date_time).unwrap_or_default();
        let summary = event.summary.unwrap_or_default();
        let description = event.description.unwrap_or_default();
        let color = event.color_id.unwrap_or_else(|| "none".to_string());

        let attendees = event
            .attendees
            .unwrap_or_default()
            .into_iter()
            .map(|a| a.email.unwrap_or_default())
            .collect();

        Event::build(&self.config, start, summary, description, color, attendees)
    }
}

impl CalendarClient for GoogleCalendarClient {
    /// events returns the events for the current day.
    async fn events(&self) -> Result<Vec<Event>> {
        // Compute the start time for the specified day.
        let time_min = Utc::now()
            .checked_add_signed(Duration::days(self.config.day_offset))
            .context("failed to adjust date with day offset")?
            .with_time(NaiveTime::MIN)
            .single()
            .context("failed to zero the time for target date")?;

        // Compute the end time for the specified day.
        let time_max = time_min
            .checked_add_signed(Duration::hours(24))
            .context("failed to compute end time")?;

        let result = self
            .hub
            .events()
            .list(&self.config.calendar_id)
            .time_min(time_min)
            .time_max(time_max)
            .single_events(true)
            .add_event_types("default")
            .order_by("startTime")
            .doit()
            .await;

        // Maps the received events to the internal representation.
        let events = result?
            .1
            .items
            .unwrap_or(vec![])
            .into_iter()
            .map(|e| self.build_agenda_event(e))
            .collect();

        Ok(events)
    }
}
