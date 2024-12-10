# agendrr

A small utility for automating the creation of my "Daily Notes" in Obsidian. I wrote more about
this setup at https://jnsgr.uk/uses.

This utility reads my calendar, and generates a short snippet of Markdown for the current day,
linking to the correct notes in my Obsidian vault.

I previously used some shonky Go code for this task, and before that some even shonkier Python.
This implementation was the second Rust application I wrote, and mostly used as a learning
exercise.

Unless you want to set up your calendar, and note taking applications to be _just like me_, it's
unlikely this will be very useful to you!

## Usage

```
A command-line utility to generate a markdown summary of events from Google Calendar.

Usage: agendrr [OPTIONS]

Options:
  -o, --offset <OFFSET>
          Number of days forwards/backwards to fetch events for

          [default: 0]

      --credentials <CREDENTIALS>
          Path to the credentials file

          [default: credentials.json]

  -c, --config-file <CONFIG_FILE>
          Path to the configuration file

          [default: agendrr.yaml]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Configuration

There is an example configuration file at [./agendrr.example.yaml], the schema is as follows:

```yaml
# (Required) The ID of the calendar in Google Calendar.
calendar-id: joe.bloggs@example.com
# (Required) The email address associated with your calendar.
user-email: joe.bloggs@example.com
# (Required} Preferred name for agenda summary generation.
user-preferred-name: Joey

# (Optional) A glob which selects a list of notes representing a set of "Regular Meeting" notes.
regular-note-glob: "/home/joe/notes/meetings/regulars/*.md"

# (Optional) A list of event name suffixes to be absent from the agenda summary.
strip-event-suffixes:
  - " - Weekly"
  - " - Monthly"
  - " - Fortnightly"

# (Optional) A list of colours to be ignored, if events are coloured in your calendar.
ignored-colours:
  - "8"

# (Optional) A list of regular expressions that match the titles of events you'd like to ignore.
ignored-regex:
  - "^Some Meeting Name$"
  - "^[C|D]EFG"

# (Optional) Map event names with a particular name to a particular note on your filesystem.
mapped-filenames:
  "Some Calendar Event with a Long/Annoying Name": "Some Meeting"
```

## Credentials

The script will look for a `credentials.json` file in the same directory as `agendrr`.

A credential can be downloaded once a [desktop OAuth app] has been created for the [Google Calendar
API].

## Running `agendrr`.

```bash
# With nix
nix run .#agendrr
# With cargo
cargo run .
```

This will create a `~/.config/agendrr/token.json` file, which will be used on subsequent runs to
get access to the API without reauthorising.

[desktop OAuth app]: https://developers.google.com/workspace/guides/create-credentials#desktop-app
[Google Calendar API]: https://developers.google.com/calendar/api/guides/overview
