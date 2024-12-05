mod google_calendar;
use crate::event::Event;
use anyhow::Result;
pub use google_calendar::GoogleCalendarClient;

pub trait CalendarClient {
    async fn events(&self) -> Result<Vec<Event>>;
}
