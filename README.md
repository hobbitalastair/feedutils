# feedutils

A series of utilites for dealing with feeds, aimed mostly at the Atom feed
format.

- `feed-read` - open all the unread entries.
- `feed-update` - update all the feeds.
- `feed-daily` - open all unread entries in feeds tagged as daily.
- `feed-addatom` - add a new atom feed.
- `feed-addrss` - add a new rss feed.
- `feed-delete` - delete an existing feed.
- `rss2atom` - attempt to convert from RSS to Atom.

`feed-read` and `feed-update` both use a "feed" directory, by default
`~/.config/feeds/`.
This contains several subdirectories, each corresponding to a single feed.
Each feed directory contains an `open` executable (the helper program to run
when opening the file), and a `fetch` executable (the program to run to
generate an up-to-date version of the feed).
Optionally, the feed directory can contain a `daily` file, which tags the feed
as daily.
Data on unread and read feeds is stored in a TSV file, by default in
`~/.local/share/feedutils.tsv`.

## Links

- [Atom spec](https://tools.ietf.org/html/rfc4287)
- [URI spec](https://tools.ietf.org/html/rfc3986)
- [W3 feed validator](https://validator.w3.org/feed/check.cgi)
- [RSS (in)compatiability and links to specs](https://web.archive.org/web/20110726002019/http://diveintomark.org/archives/2004/02/04/incompatible-rss)
- [RSS 2.0 spec](https://cyber.harvard.edu/rss/rss.html)
- [RSS 1.0 spec](http://web.resource.org/rss/1.0/spec)

