extern crate url;
extern crate xml;

use std::collections::HashMap;
use std::env;
use std::io;
use std::io::{BufWriter, Write, BufReader, BufRead};
use std::fs;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, exit};
use std::thread;
use std::time;

use chrono::DateTime;
use thiserror::Error;
use url::Url;
use xml::reader::{EventReader, XmlEvent};

const ENTRY_DATABASE_HEADER: &str = "feed\tid\tupdated\ttitle\tlink\tread\n";

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
struct Entry {
    feed: String,
    id: String,
    title: String,
    updated: String,
    link: String,
    read: bool,
}

fn sanitize(data: String) -> String {
    // Remove control characters - this should prevent the worst issues when
    // trying to deal with the resulting data stream.
    // Note that this removes *characters*, not *bytes*, so I'm assuming that
    // the code reading the result handles UTF-8 properly.

    let mut sanitized_data = String::new();

    for c in data.chars() {
        if !c.is_control() {
            sanitized_data.push(c);
        }
    }

    return sanitized_data;
}

fn handle_rss_pub_date(pub_date: Option<String>) -> String {
    // RSS publication date is both optional and often not proper RFC2822.
    // In order to be generous, try to parse it but fall back where that isn't
    // possible.

    match pub_date {
        Some(pub_date) => {
            let mut rfc2822_pub_date = pub_date;
            // Some feeds have improper RFC2822 dates, specifying UTC not UT.
            // Replace with GMT - we don't care about the timezone.
            rfc2822_pub_date = rfc2822_pub_date.replace("UTC", "GMT");
            let updated = DateTime::parse_from_rfc2822(&rfc2822_pub_date);
            match updated {
                Ok(updated) => updated.to_rfc3339(),
                Err(_) => {
                    // Couldn't parse, fall back to current datetime.
                    chrono::offset::Utc::now().to_rfc3339()
                },
            }
        },
        None => {
            // Strictly speaking the publication date is optional... fall back
            // to the current datetime.
            chrono::offset::Utc::now().to_rfc3339()
        },
    }
}

fn parse_rss<R: std::io::Read>(parser: xml::reader::Events<R>, feed: &String) -> Vec<Entry> {
    // Turn an RSS-like XML feed into a vector of entries
    // Data is attempted to be sanitized

    let mut pending_data: Option<String> = None;
    let mut id: Option<String> = None;
    let mut title: Option<String> = None;
    let mut pub_date: Option<String> = None;
    let mut link: Option<String> = None;

    let mut entries: Vec<Entry> = Vec::new();

    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { name, .. }) => {
                pending_data = None;
                if name.local_name == "item" {
                    id = None;
                    title = None;
                    pub_date = None;
                    link = None;
                }
            }
            Ok(XmlEvent::EndElement { name }) => {
                match name.local_name.as_str() {
                    "guid" => {
                        id = pending_data.take();
                    }
                    "title" => {
                        title = pending_data.take();
                    }
                    "pubDate" => {
                        pub_date = pending_data.take();
                    }
                    "link" => {
                        link = pending_data.take();
                    }
                    "item" => {
                        if link.is_none() {
                            eprintln!("Ignoring incomplete entry, missing link field");
                            continue;
                        }
                        if id.is_none() {
                            // Fallback to the link if no GUID is specified
                            id = Some(link.clone().unwrap());
                        }
                        if title.is_none() {
                            // Empty title is not great but OK; ignore
                            title = Some("Untitled".to_string());
                        }

                        let entry = Entry {
                            feed: feed.clone(),
                            id: id.take().unwrap(),
                            title: title.take().unwrap(),
                            updated: handle_rss_pub_date(pub_date.take()),
                            link: link.take().unwrap(),
                            read: false,
                        };
                        entries.push(entry);
                    }
                    _ => {}
                }
            }
            Ok(XmlEvent::CData(data)) => {
                pending_data = Some(sanitize(data));
            }
            Ok(XmlEvent::Characters(data)) => {
                pending_data = Some(sanitize(data));
            }
            Err(e) => {
                eprintln!("Error parsing XML: {}", e);
                break;
            }
            _ => {}
        }
    }
    return entries;
}

fn parse_atom<R: std::io::Read>(parser: xml::reader::Events<R>, feed: &String) -> Vec<Entry> {
    // Turn an Atom-like XML feed into a vector of entries
    // Data is attempted to be sanitized

    let mut pending_data: Option<String> = None;
    let mut id: Option<String> = None;
    let mut title: Option<String> = None;
    let mut updated: Option<String> = None;
    let mut link: Option<String> = None;

    let mut entries: Vec<Entry> = Vec::new();

    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                pending_data = None;
                if name.local_name == "link" {
                    for attr in attributes {
                        if attr.name.local_name == "href" {
                            let url = sanitize(attr.value);
                            match Url::parse(&url) {
                                Ok(_) => {
                                    link = Some(url);
                                },
                                Err(e) => {
                                    eprintln!("Ignoring invalid URL: {e}");
                                },
                            }
                        }
                    }
                } else if name.local_name == "entry" {
                    id = None;
                    title = None;
                    updated = None;
                    link = None;
                }
            }
            Ok(XmlEvent::EndElement { name }) => {
                match name.local_name.as_str() {
                    "id" => {
                        id = pending_data.take();
                    }
                    "title" => {
                        title = pending_data.take();
                    }
                    "updated" => {
                        updated = pending_data.take();
                    }
                    "entry" => {
                        if id.is_none() {
                            eprintln!("Ignoring incomplete entry, missing id field");
                        } else if title.is_none() {
                            eprintln!("Ignoring entry as missing title field: {}", id.take().unwrap());
                        } else if updated.is_none() {
                            eprintln!("Ignoring incomplete entry, missing updated field");
                        } else if link.is_none() {
                            eprintln!("Ignoring incomplete entry, missing link field");
                        } else {
                            let entry = Entry {
                                feed: feed.clone(),
                                id: id.take().unwrap(),
                                title: title.take().unwrap(),
                                updated: updated.take().unwrap(),
                                link: link.take().unwrap(),
                                read: false,
                            };
                            entries.push(entry);
                        }
                    }
                    _ => {}
                }
            }
            Ok(XmlEvent::CData(data)) => {
                pending_data = Some(sanitize(data));
            }
            Ok(XmlEvent::Characters(data)) => {
                pending_data = Some(sanitize(data));
            }
            Err(e) => {
                eprintln!("Error parsing XML: {}", e);
                break;
            }
            _ => {}
        }
    }
    return entries;
}

fn parse_feed<R: std::io::Read>(reader: R, feed: &String) -> Vec<Entry> {
    // Turn an XML feed into a vector of entries.
    // Format is attempted to be autodetected, either Atom or RSS.
    // Data is attempted to be sanitized.

    let mut parser = EventReader::new(reader).into_iter();
    while let Some(e) = parser.next() {
        match e {
            Ok(XmlEvent::StartElement { name, .. }) => {
                if name.local_name == "rss" {
                    // Probably an RSS feed
                    return parse_rss(parser, feed);
                }
                if name.local_name == "feed" {
                    // Probably an Atom feed
                    return parse_atom(parser, feed);
                }
            },
            Err(e) => {
                eprintln!("Error parsing XML: {}", e);
                break;
            },
            _ => {},
        }
    }

    eprintln!("Doesn't seem to be either an Atom or an RSS feed?");
    return Vec::new();
}

fn open_lockfile(filename: PathBuf) -> io::Result<fs::File> {
    // Attempt to acquire a lockfile
    // Will block until it exists, or the attempt times out

    let mut delay = time::Duration::from_millis(50);
    let timeout = time::Duration::from_millis(2000); // 2s seems more than long enough

    loop {
        let result = OpenOptions::new().write(true)
                                       .create_new(true)
                                       .open(filename.clone());
        match result {
            Ok(f) => return Ok(f),
            Err(e) => {
                if e.kind() != io::ErrorKind::AlreadyExists {
                    return Err(e);
                } else if delay >= timeout {
                    // Timed out - since delays double each time, so far we've
                    // waited just under `delay` time (using triangle formula).
                    return Err(e);
                } else {
                    thread::sleep(delay);
                    delay *= 2;
                }
            }
        }

    }
}

#[derive(Error, Debug)]
enum DatabaseReadError {
    #[error("{source}: {path}")]
    IoError {
        source: io::Error,
        path: PathBuf,
    },
    #[error("Missing {field} field, ignoring entry")]
    MissingField {
        field: String,
    },
}

fn read_entries(filename: PathBuf) -> Result<Vec<Entry>, DatabaseReadError> {
    let f = OpenOptions::new().read(true).open(&filename)
            .map_err(|e| DatabaseReadError::IoError{ source: e, path: filename.clone() })?;
    let reader = BufReader::new(f);

    let mut entries: Vec<Entry> = Vec::new();

    for line in reader.lines().skip(1) {
        match line {
            Ok(line) => {
                let mut fields = line.split("\t");
                let entry = Entry {
                    feed: fields.next().ok_or(DatabaseReadError::MissingField{ field: "feed".to_string() })?.to_string(),
                    id: fields.next().ok_or(DatabaseReadError::MissingField{ field: "id".to_string() })?.to_string(),
                    updated: fields.next().ok_or(DatabaseReadError::MissingField{ field: "updated".to_string() })?.to_string(),
                    title: fields.next().ok_or(DatabaseReadError::MissingField{ field: "title".to_string() })?.to_string(),
                    link: fields.next().ok_or(DatabaseReadError::MissingField{ field: "link".to_string() })?.to_string(),
                    read: fields.next().ok_or(DatabaseReadError::MissingField{ field: "read".to_string() })? == "read",
                };
                entries.push(entry);
            },
            Err(e) => {
                return Err(DatabaseReadError::IoError{ source: e, path: filename.clone() });
            }
        }
    }

    return Ok(entries);
}

fn write_entries(f: &mut fs::File, entries: &Vec<Entry>) -> io::Result<()> {
    let mut writer = BufWriter::new(f);

    writer.write_all(ENTRY_DATABASE_HEADER.as_bytes())?;

    for e in entries {
        // Can safely use tabs and newlines as delimiters as removed earlier
        let line = [
            e.feed.clone(),
            e.id.clone(),
            e.updated.clone(),
            e.title.clone(),
            e.link.clone(),
            if e.read { "read".to_string() } else { "unread".to_string() },
        ].join("\t") + "\n";
        writer.write_all(line.as_bytes())?;
    }

    return Ok(());
}

#[derive(Error, Debug)]
enum ModifyDatabaseError {
    #[error("Unable to lock database: {source}: {path}")]
    LockCreateError {
        source: io::Error,
        path: PathBuf,
    },
    #[error("Database read error: {source}")]
    ReadError {
        source: DatabaseReadError,
    },
    #[error("Unable to {operation} database: {source}: {path}")]
    WriteError {
        source: io::Error,
        path: PathBuf,
        operation: String,
    },
}

fn modify_database<F>(modifier: F, database_path: PathBuf) -> Result<(), ModifyDatabaseError>
    where F: FnOnce(Vec<Entry>) -> Vec<Entry>
{
    // We assume that database_path here has a filename, which should be true since it always comes
    // from get_database_path... but in that case maybe we should wrap it into here instead of
    // getting it as an argument?
    let mut lock_file_name = database_path.file_name().unwrap().to_os_string();
    lock_file_name.push(std::ffi::OsStr::new(".lock"));
    let mut lockfile_path = database_path.clone();
    lockfile_path.set_file_name(lock_file_name);

    let mut lockfile = open_lockfile(lockfile_path.clone())
                        .map_err(|e| ModifyDatabaseError::LockCreateError{ source: e, path: lockfile_path.clone() })?;

    // We need to delete the lockfile on failure!
    let cleanup_file = |e, path| -> ModifyDatabaseError {
        if let Err(err) = fs::remove_file(path) {
            eprintln!("Unable to delete lockfile: {err}");
        }
        e
    };

    let entries = read_entries(database_path.clone())
                    .map_err(|e| ModifyDatabaseError::ReadError{ source: e })
                    .map_err(|e| cleanup_file(e, lockfile_path.clone()))?;

    let modified_entries = modifier(entries);

    write_entries(&mut lockfile, &modified_entries)
        .map_err(|e| ModifyDatabaseError::WriteError{ source: e, path: lockfile_path.clone(), operation: "write".to_string() })
        .map_err(|e| cleanup_file(e, lockfile_path.clone()))?;

    lockfile.sync_all()
        .map_err(|e| ModifyDatabaseError::WriteError{ source: e, path: lockfile_path.clone(), operation: "sync".to_string() })
        .map_err(|e| cleanup_file(e, lockfile_path.clone()))?;

    fs::rename(lockfile_path.clone(), database_path)
        .map_err(|e| ModifyDatabaseError::WriteError{ source: e, path: lockfile_path.clone(), operation: "replace".to_string() })
        .map_err(|e| cleanup_file(e, lockfile_path.clone()))?;

    return Ok(());
}

fn merge_feed(feed_name: String, feed_entries: Vec<Entry>, database_entries: Vec<Entry>) -> Vec<Entry> {
    // Merging a feed:
    // - entries in the feed but not in the database are added
    // - read entries in the database but not in the feed are removed

    // Treat all entries from the feed as new initially
    let mut new_feed_entries = HashMap::new();
    for entry in feed_entries.into_iter() {
        new_feed_entries.insert(entry.id.clone(), entry);
    }

    let mut modified_database_entries: Vec<Entry> = Vec::new();
    for entry in database_entries {
        if entry.feed != feed_name {
            // For a different feed; retain
            modified_database_entries.push(entry);
        } else if new_feed_entries.contains_key(&entry.id) {
            // Not actually a new entry
            new_feed_entries.remove(&entry.id);
            modified_database_entries.push(entry);
        } else if !entry.read {
            // Not in the feed, but not yet read; keep
            modified_database_entries.push(entry);
        }
    }

    // Add the actually new entries
    for entry in new_feed_entries.into_values() {
        modified_database_entries.push(entry);
    }

    return modified_database_entries;
}

#[derive(Error, Debug)]
enum DatabasePathError {
    #[error("No env var set for database path")]
    NoEnvVar,
}

fn get_database_path() -> Result<PathBuf, DatabasePathError> {
    // Database path; check possible settings env vars in sequence.
    // Does not check if the directory or file actually exists.
    
    if let Some(path) = env::var_os("FEEDUTILS_DB") {
        return Ok(PathBuf::from(path));
    }
    if let Some(path) = env::var_os("XDG_DATA_HOME") {
        return Ok(PathBuf::from(path).join("feedutils.tsv"));
    }
    if let Some(path) = env::var_os("HOME") {
        return Ok(PathBuf::from(path).join(".local/share/feedutils.tsv"));
    }

    return Err(DatabasePathError::NoEnvVar);
}

fn get_feed_config_dir() -> Option<PathBuf> {
    // Feed configuration directory path; check possible settings env vars in sequence.
    // Does not check if the directory actually exists.

    if let Some(path) = env::var_os("FEEDUTILS_CONFIGDIR") {
        return Some(PathBuf::from(path));
    }
    if let Some(path) = env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(path).join("feeds"));
    }
    if let Some(path) = env::var_os("HOME") {
        return Some(PathBuf::from(path).join(".config/feeds"));
    }

    return None;
}

#[derive(Error, Debug)]
enum FeedDirError {
    #[error("Cannot read feed config")]
    CannotReadConfigDir,
    #[error("Feed does not exist")]
    FeedDoesNotExist,
}

fn get_feed_dir(feed_name: String) -> Result<PathBuf, FeedDirError> {
    let config_dir = get_feed_config_dir().ok_or(FeedDirError::CannotReadConfigDir)?;
    fs::metadata(config_dir.clone()).map_err(|_| FeedDirError::CannotReadConfigDir)?;
    let feed_dir = config_dir.join(feed_name);
    fs::metadata(feed_dir.clone()).map_err(|_| FeedDirError::FeedDoesNotExist)?;
    return Ok(feed_dir);
}

fn get_all_feed_names() -> io::Result<Vec<String>> {
    let mut feeds = Vec::new();
    let config_dir = get_feed_config_dir().ok_or(io::ErrorKind::Other)?;
    for entry in fs::read_dir(config_dir)? {
        let entry = entry?;
        let filetype = entry.file_type()?;
        if filetype.is_dir() {
            // We sanitize the feed name as it may be used later in the database
            let sanitized_name = sanitize(entry.file_name().to_string_lossy().into_owned());
            feeds.push(sanitized_name);
        }
    }
    feeds.sort();
    return Ok(feeds);
}

#[derive(Error, Debug)]
enum UpdateError {
    #[error(transparent)]
    DatabasePathError(#[from] DatabasePathError),
    #[error(transparent)]
    FeedDirError(#[from] FeedDirError),
    #[error("Failed to launch fetch executable: {source}: {path}")]
    ExecError {
        source: io::Error,
        path: String
    },
    #[error("Failed to run fetch, got {status}")]
    FetchError {
        stderr: Vec<u8>,
        status: ExitStatus,
    },
    #[error("Failed to update database: {source}")]
    DatabaseError {
        source: ModifyDatabaseError,
    },
}

fn update(feed_name: String) -> Result<(), UpdateError> {
    let feed_dir_path = get_feed_dir(feed_name.clone())?;

    let exec_path = feed_dir_path.clone().join("fetch");
    let error_path = feed_dir_path.join("error.log");

    let output = Command::new(exec_path.clone()).output()
                .map_err(|e| UpdateError::ExecError{ source: e, path: exec_path.display().to_string() })?;
    // On failure, save the error into a file so that a later interactive
    // program can tell the user about the program. On success, delete any
    // such error files. We don't really care if that fails though.
    if !output.status.success() {
        let _ = fs::write(error_path, output.stderr.clone());
        return Err(UpdateError::FetchError{ stderr: output.stderr, status: output.status });
    } else {
        // If an old error file exists, delete it
        let _ = fs::remove_file(error_path);
    }

    let feed_entries = parse_feed(output.stdout.as_slice(), &feed_name);
    let merge = |entries: Vec<Entry>| -> Vec<Entry> {
        return merge_feed(feed_name, feed_entries, entries);
    };
    let database_path = get_database_path().map_err(|e| UpdateError::DatabasePathError(e))?;
    return modify_database(merge, database_path)
        .map_err(|e| UpdateError::DatabaseError{ source: e });
}

#[derive(Error, Debug)]
enum MarkEntryAsReadError {
    #[error(transparent)]
    DatabasePathError(#[from] DatabasePathError),
    #[error(transparent)]
    ModifyDatabaseError(#[from] ModifyDatabaseError),
}

fn mark_entry_as_read(feed_name: String, entry_id: String) -> Result<(), MarkEntryAsReadError> {
    let modifier = |entries: Vec<Entry>| -> Vec<Entry> {
        let mut modified_entries: Vec<Entry> = Vec::new();
        for mut entry in entries {
            if entry.feed == feed_name && entry.id == entry_id {
                entry.read = true;
            }
            modified_entries.push(entry);
        }
        return modified_entries;
    };
    let database_path = get_database_path()
                        .map_err(|e| MarkEntryAsReadError::DatabasePathError(e))?;
    return modify_database(modifier, database_path)
           .map_err(|e| MarkEntryAsReadError::ModifyDatabaseError(e));
}

#[derive(Error, Debug)]
enum EntryReadError {
    #[error(transparent)]
    FeedDirError(#[from] FeedDirError),
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error(transparent)]
    MarkEntryAsReadError(#[from] MarkEntryAsReadError),
    #[error("Failed to launch open executable: {source}: {path}")]
    ExecError {
        source: io::Error,
        path: PathBuf,
    },
}

fn read_entry(feed_name: String, entry: Entry) -> Result<(), EntryReadError> {
    let feed_dir_path = get_feed_dir(feed_name.clone())?;

    let exec_path = feed_dir_path.clone().join("open");
    Command::new(exec_path.clone())
        .env("TITLE", entry.title.as_str())
        .env("LINK", entry.link.as_str())
        .status()
        .map_err(|e| EntryReadError::ExecError{ source: e, path: exec_path })?;
    
    return mark_entry_as_read(entry.feed, entry.id).map_err(|e| EntryReadError::MarkEntryAsReadError(e));
}

#[derive(Error, Debug)]
enum GetEntriesError {
    #[error(transparent)]
    DatabasePathError(#[from] DatabasePathError),
    #[error(transparent)]
    DatabaseReadError(#[from] DatabaseReadError),
}

fn get_feed_entries(feed_name: String) -> Result<Vec<Entry>, GetEntriesError> {
    let database_path = get_database_path()
        .map_err(|e| GetEntriesError::DatabasePathError(e))?;
    let entries = read_entries(database_path)
        .map_err(|e| GetEntriesError::DatabaseReadError(e))?;

    let mut feed_entries = Vec::new();
    for entry in entries {
        if entry.feed == feed_name {
            feed_entries.push(entry);
        }
    }

    // Sort in date order, falling back to ID, so that when reading
    // entries the oldest is opened first.
    feed_entries.sort_by(|a, b| (&a.updated, &a.id).cmp(&(&b.updated, &b.id)));
    return Ok(feed_entries);
}

fn count_unread_entries() -> Result<HashMap<String, u32>, GetEntriesError> {
    let database_path = get_database_path()
        .map_err(|e| GetEntriesError::DatabasePathError(e))?;
    let entries = read_entries(database_path)
        .map_err(|e| GetEntriesError::DatabaseReadError(e))?;

    let mut feed_entries: HashMap<String, u32> = HashMap::new();
    for entry in entries {
        if !entry.read {
            let count = match feed_entries.get(&entry.feed) {
                Some(i) => i,
                None => &0,
            };
            feed_entries.insert(entry.feed.clone(), count + 1);
        }
    }

    return Ok(feed_entries);
}

fn exec_feed_update(args: Vec<String>) {
    let feeds = match args.len() {
        1 => {
            match get_all_feed_names() {
                Ok(feeds) => feeds,
                Err(e) => {
                    eprintln!("Cannot list feeds: {}", e);
                    exit(1);
                }
            }
        },
        2 => vec![args[1].clone()],
        _ => {
            eprintln!("usage: feed-update [<feed>]");
            exit(1);
        }
    };

    let mut ok = true;
    for feed_name in feeds {
        println!("Updating feed {}", feed_name);
        let res = update(feed_name);
        match res {
            Err(e) => {
                eprintln!("{}", e);
                ok = false;
            },
            _ => {}
        }
    }
    if !ok {
        exit(1);
    }
}

fn exec_feed_unread(args: Vec<String>) {
    if args.len() == 1 {
        let entry_counts = match count_unread_entries() {
            Ok(entry_counts) => entry_counts,
            Err(e) => {
                eprintln!("{}", e);
                exit(1);
            },
        };
        let mut feed_and_unread = Vec::from_iter(entry_counts.into_iter());
        feed_and_unread.sort_by(|a, b| (a.1, &a.0).cmp(&(b.1, &b.0)));
        for (feed_name, unread_count) in feed_and_unread {
            println!("{: >4} {}", unread_count, feed_name);
        }
    } else {
        eprintln!("usage: feed-unread");
        exit(1);
    }
}

fn exec_feed_read(args: Vec<String>) {
    if args.len() >= 2 {
        let mut ok = true;
        for feed_name in args.into_iter().skip(1) {
            if let Err(e) = get_feed_dir(feed_name.clone()) {
                eprintln!("{}: {}", e, feed_name.clone());
                ok = false;
                continue;
            }

            let entries = match get_feed_entries(feed_name.clone()) {
                Ok(entries) => entries,
                Err(e) => {
                    eprintln!("{}", e);
                    exit(1);
                },
            };
            for entry in entries {
                if !entry.read {
                    if let Err(e) = read_entry(feed_name.clone(), entry) {
                        eprintln!("{}", e);
                        ok = false;
                    }
                }
            }
        }
        if !ok {
            exit(1);
        }
    } else {
        eprintln!("usage: feed-read <feed> [<feed> ...]");
        exit(1);
    }
}

fn exec_feed_markasread(args: Vec<String>) {
    if args.len() == 2 {
        let feed_name = args[1].clone();
        if let Err(e) = get_feed_dir(feed_name.clone()) {
            eprintln!("{}: {}", e, feed_name.clone());
            exit(1);
        }

        let database_path = match get_database_path() {
            Ok(path) => path,
            Err(e) => {
                eprintln!("{}", e);
                exit(1);
            },
        };

        let modifier = |entries: Vec<Entry>| -> Vec<Entry> {
            let mut modified_entries: Vec<Entry> = Vec::new();
            for mut entry in entries {
                if entry.feed == feed_name {
                    entry.read = true;
                }
                modified_entries.push(entry);
            }
            return modified_entries;
        };
        if let Err(e) = modify_database(modifier, database_path) {
            eprintln!("Failed to mark {} as read: {}", feed_name.clone(), e);
            exit(1);
        }
    } else {
        eprintln!("usage: feed-markasread <feed>");
        exit(1);
    }
}

fn exec_feed_delete(args: Vec<String>) {
    if args.len() == 2 {
        let feed_name = args[1].clone();

        let feed_dir = match get_feed_dir(feed_name.clone()) {
            Ok(feed_dir) => feed_dir,
            Err(e) => {
                eprintln!("{}: {}", e, feed_name.clone());
                exit(1);
            },
        };

        let database_path = match get_database_path() {
            Ok(path) => path,
            Err(e) => {
                eprintln!("{}", e);
                exit(1);
            },
        };

        let modifier = |entries: Vec<Entry>| -> Vec<Entry> {
            let mut modified_entries: Vec<Entry> = Vec::new();
            for entry in entries {
                if entry.feed != feed_name {
                    modified_entries.push(entry);
                }
            }
            return modified_entries;
        };
        if let Err(e) = modify_database(modifier, database_path) {
            eprintln!("Failed to delete entries: {}", e);
            exit(1);
        }

        if let Err(e) = fs::remove_dir_all(feed_dir) {
            eprintln!("Failed to delete feed configuration: {}", e);
            exit(1);
        }
    } else {
        eprintln!("usage: feed-delete <feed>");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 0 {
        eprintln!("{} must have the first exec arg as the executable name", env!("CARGO_PKG_NAME"));
        std::process::exit(1);
    }

    let executable_name = std::path::Path::new(&args[0]).file_name().and_then(std::ffi::OsStr::to_str);
    match executable_name {
        Some("feed-update") => exec_feed_update(args),
        Some("feed-unread") => exec_feed_unread(args),
        Some("feed-read") => exec_feed_read(args),
        Some("feed-markasread") => exec_feed_markasread(args),
        Some("feed-delete") => exec_feed_delete(args),
        Some(executable_name) => {
            eprintln!("Executable name {} not recognized", executable_name);
            std::process::exit(1);
        }
        None => {
            eprintln!("Executable name {} not recognized", args[0]);
            std::process::exit(1);
        }
    }
}
