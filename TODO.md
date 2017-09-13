# TODO

- Implement a test suite with any pathological examples I can find.
  Perhaps do some fuzzing too?
- Implement proper date/time parsing and printing.
- Check escaping code in rss2atom.
- Implement support for the RSS `<image>` tag.
- Check the return codes of printf, fprintf, or putc.
- Implement support for filtering and tagging feeds.
- Implement support for batching updates (eg only read if the oldest unread
  is older than a month).
- Provide a curl wrapper recognizing a 200-code response.
- Provide some example open, fetch scripts.
- Allow filtering feed-read and feed-update.
- Add helpers for detecting old feeds.
- Remove unparseable feed files.
- Order unread entries by date, falling back to name if needed.

