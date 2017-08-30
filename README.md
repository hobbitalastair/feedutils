# atomutils

A series of utilites for dealing with feeds, aimed mostly at the Atom feed
format.

- `rss2atom` - attempt to convert from RSS to Atom.
- `atom-list` - print the id of every entry in a feed.
- `atom-extract` - print the entry with the given id in a feed.
- `atom-exec` - run a child, exporting the tags in the feed as environment
                variables.
- `feed-read` - open all the unread entries.
- `feed-update` - update all the feeds.
- `feed-unescape` - unescape a given escaped entry id.

`feed-read` and `feed-update` both use a "feed" directory, by default
`~/.config/feeds/`.
This contains several subdirectories, each corresponding to a single feed.
Each feed directory contains an `unread` directory, a `read` directory, a
`open` executable (the helper program to run when opening the file), and a
`fetch` executable (the program to run to generate an up-to-date version of
the feed).
Optionally, the feed directory can contain a `cache` executable, which
pre-fetches interesting data from the entry.

## Links

- [Atom spec](https://tools.ietf.org/html/rfc4287)
- [URI spec](https://tools.ietf.org/html/rfc3986)
- [W3 feed validator](https://validator.w3.org/feed/check.cgi)
- [RSS (in)compatiability and links to specs](https://web.archive.org/web/20110726002019/http://diveintomark.org/archives/2004/02/04/incompatible-rss)
- [RSS 2.0 spec](https://cyber.harvard.edu/rss/rss.html)
- [RSS 1.0 spec](http://web.resource.org/rss/1.0/spec)

- [expat documentation](http://soc.if.usp.br/manual/libexpat1-dev/expat.html/index.html)

