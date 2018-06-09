# TODO

- Implement a test suite with any pathological examples I can find.
  Perhaps do some fuzzing too?
- Check escaping code in rss2atom.
- Implement support for the RSS `<image>` tag.
- Check the return codes of printf, fprintf, or putc.
- Implement support for filtering and tagging feeds.
- Allow filtering feed-read and feed-update.
- Implement support for batching updates (eg only read if the oldest unread
  is older than a month).
- Provide a curl wrapper recognizing a 200-code response.
- Provide some example open, fetch scripts.
- Add helpers for detecting old feeds.
- Remove unparseable feed files.

- .config is the wrong place to store everything
  (use .local/share for entries and status, .cache for cached downloads).
- Implement a few different feed read methods to batch some updates or tagging
  or similar instead.
