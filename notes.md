# Design notes

Goals:

- Make Toshiba-friendly (minimize CPU/RAM/disk usage)
- Open all unread sequentially with minimal latency and custom applications
  - Minimize latency by pre-downloading files
  - Reduce RAM usage through custom applications

Wants:

- Cache items for offline reading
- Save old unread items (so I can leave it running when I go away)
- Improve interoperability with Unison and similar filesystem sync

# Architecture notes

- Generators used - output specific "feed"
- Readers used - for reading specific items in feed
- Scripts can be edge-triggered (things created, removed, marked read)
- We only need to store a minimal amount of information!

Note that XSLT would be a natural choice for implementing simple generators,
the utilities, or any scrapers for readers.
Because XSLT only works on well-formed XML this may imply converting from HTML
to XML before processing (eg using [tidy](https://github.com/htacg/tidy-html5)).

Atom feeds are nicer (they have a unique ID!) so convert to that instead of
supporting RSS.

# Implementation

I want to write a simple wrapper script and a whole bunch of utilities to make
the wrapper script feasible.

Wrapper interface:

- `feed-read` - open each unread feed entry
- `feed-update` - update each feed

Known feeds are stored as subdirectories of `~/.config/feeds`, using the format
`~/.config/feeds/name`. This contains a `fetch` executable, `open` executable,
a `cache` executable, and an `entry` directory.
The `entry` directory contains a bunch of files each corresponding to a feed
entry, where the filename is the normalized id of the entry, and the contents
of the file is piped to the `open` executable to load the corresponding entry.
The `fetch` executable is expected to output a feed file, which can be
processed and compared with the existing `feed` file to generate new unread
entries and remove `read` entries that are no longer in the feed.

`feed-read` would iterate through all of the subdirectories of
`~/.config/feeds/` and execute the `open` program for each unread entry,
waiting for the `open` program to finish before continuing.
If the `open` program exits with status 0, then the entry is marked as read by
creating a `read` file in the entries directory.

`feed-update` would iterate through all of the subdirectories of
`~/.config/feeds/`. For each feed directory, it would execute the `fetch`
executable, storing the output in a `feed.XXXX` temporary file. Then the old
`feed` and the new `feed.XXXX` files would be compared, and the new entries
extracted into the `entry` directory. Also, any read entries which aren't in
the new feed should be removed. For each new entry, it would also run the
`cache` executable with the new entry as input. Finally, the `feed.XXXX` file
would be moved over the top of the old `feed` file.
Some care is required here to deal with multiple update operations happening at
once, a broken new feed file, and other forms of operations failing, to avoid
littering the directories with broken or irrelevant files, creating files with
bad filenames, executing arbitrary shell code (be careful when dealing with the
variables!!), dropping or duplicating entries, or worse!

Care *must* be taken when dealing with the feeds to avoid creating
vulnerabilities (eg arbitrary code execution).

# ID escaping scheme

We need to treat any Atom entry:id values as being at best IRIs and at worst
malicious. To avoid dealing with nasty values, we define a bijection between
arbitrary strings and filesystem-safe strings.

To escape an arbitrary string, perform the following per-character tests:

1. If the character is a `\`, output `\`, `\`.
2. If the character is a null byte, output `\`, `0`.
3. If the character is a `/`, output `\`, `_`.
4. If the character is a `\n`, output `\`, `n`.
5. If the character is a `.` and is the first character read, output `\`, `.`.
6. Otherwise, output the character as-is.

To unescape an arbitrary string, perform the following operations, where the
`escaped` flag is a variable initially unset.

1. If the character is a `\` and `escaped` is set, output `\` and unset
  `escaped`.
  Otherwise, set `escaped`.
2. If the character is a `0` and `escaped` is set, output `\0`.
3. If the character is a `_` and `escaped` is set, output `/`.
4. If the character is a `n` and `escaped` is set, output `\n`.
5. If the character is a `.` and `escaped` is set, output `.`.
6. Otherwise, output the character as-is.

