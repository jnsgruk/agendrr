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
