extern crate xml;
extern crate dirs;

use std::collections::HashMap;
use std::io;
use std::io::{BufWriter, Write, BufReader, BufRead};
use std::fs;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, exit};
use thiserror::Error;
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

fn read_atom<R: std::io::Read>(reader: R, feed: &String) -> Vec<Entry> {
    // Turn an Atom-like XML feed into a vector of entries
    // Data is attempted to be sanitized

    let mut pending_data: Option<String> = None;
    let mut id: Option<String> = None;
    let mut title: Option<String> = None;
    let mut updated: Option<String> = None;
    let mut link: Option<String> = None;

    let mut entries: Vec<Entry> = Vec::new();

    let parser = EventReader::new(reader);
    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                if name.local_name == "link" {
                    for attr in attributes {
                        if attr.name.local_name == "href" {
                            // FIXME: Should sanitize this as an actual URL
                            link = Some(sanitize(attr.value));
                        }
                    }
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

fn open_lockfile(filename: String) -> io::Result<fs::File> {
    // FIXME: Retry on failure, with a delay in case another program is writing

    let f = OpenOptions::new().write(true)
                              .create_new(true)
                              .open(filename);
    return f;
}

#[derive(Error, Debug)]
enum DatabaseReadError {
    #[error("IO error")]
    IoError(#[from] io::Error),
    #[error("Missing {field} field, ignoring entry")]
    MissingField {
        field: String,
    },
}

fn read_entries(filename: String) -> Result<Vec<Entry>, DatabaseReadError> {
    let f = OpenOptions::new().read(true).open(filename)?;
    let reader = BufReader::new(f);

    let mut entries: Vec<Entry> = Vec::new();

    for line in reader.lines().skip(1) {
        match line {
            Ok(line) => {
                let mut fields = line.split("\t");
                // FIXME: Likely should sanitize field values and/or return a better
                //        error message if something went wrong
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
                return Err(DatabaseReadError::IoError(e));
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
        // TODO: Should perhaps assert that here for safety?
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
        path: String,
    },
    #[error("Database read error: {source}")]
    ReadError {
        source: DatabaseReadError,
    },
    #[error("Unable to {operation} database: {source}: {path}")]
    WriteError {
        source: io::Error,
        path: String,
        operation: String,
    },
}

fn modify_database<F>(modifier: F, filename: String) -> Result<(), ModifyDatabaseError>
    where F: FnOnce(Vec<Entry>) -> Vec<Entry>
{
    let lockfilename = filename.clone() + ".lock";
    let mut lockfile = open_lockfile(lockfilename.clone())
                        .map_err(|e| ModifyDatabaseError::LockCreateError{ source: e, path: lockfilename.clone() })?;

    // We need to delete the lockfile on failure!
    let cleanup_file = |e, path| -> ModifyDatabaseError {
        if let Err(err) = fs::remove_file(path) {
            eprintln!("Unable to delete lockfile: {err}");
        }
        e
    };

    let entries = read_entries(filename.clone())
                    .map_err(|e| ModifyDatabaseError::ReadError{ source: e })
                    .map_err(|e| cleanup_file(e, lockfilename.clone()))?;

    let modified_entries = modifier(entries);

    write_entries(&mut lockfile, &modified_entries)
        .map_err(|e| ModifyDatabaseError::WriteError{ source: e, path: lockfilename.clone(), operation: "write".to_string() })
        .map_err(|e| cleanup_file(e, lockfilename.clone()))?;

    lockfile.sync_all()
        .map_err(|e| ModifyDatabaseError::WriteError{ source: e, path: lockfilename.clone(), operation: "sync".to_string() })
        .map_err(|e| cleanup_file(e, lockfilename.clone()))?;

    fs::rename(lockfilename.clone(), filename)
        .map_err(|e| ModifyDatabaseError::WriteError{ source: e, path: lockfilename.clone(), operation: "replace".to_string() })
        .map_err(|e| cleanup_file(e, lockfilename.clone()))?;

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

fn get_database_path() -> String {
    // FIXME: Implement using dirs
    return "test.tsv".to_string();
}

fn get_feed_config_dir() -> Option<PathBuf> {
    return dirs::config_dir().map(|mut path| { path.push("feeds"); path });
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
    // FIXME: Should do some error checking that this directory actually exists?

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

    let feed_entries = read_atom(output.stdout.as_slice(), &feed_name);
    let merge = |entries: Vec<Entry>| -> Vec<Entry> {
        return merge_feed(feed_name, feed_entries, entries);
    };
    return modify_database(merge, get_database_path())
        .map_err(|e| UpdateError::DatabaseError{ source: e });
}

fn mark_entry_as_read(feed_name: String, entry_id: String) -> Result<(), ModifyDatabaseError> {
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
    return modify_database(modifier, get_database_path());
}

#[derive(Error, Debug)]
enum EntryReadError {
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error(transparent)]
    ModifyDatabaseError(#[from] ModifyDatabaseError),
}

fn entry_read(entry: Entry) -> Result<(), EntryReadError> {
    // FIXME: Allow configurable open functions (ie with yes/no prompts)
    // FIXME: Add extra sanitization of the URL here for extra safety
    let status = Command::new("chromium")
                    .args(["--", entry.link.as_str()])
                    .status();
    // FIXME: Implement error handling
    if !status?.success() {
        return Ok(());
    }
    
    return mark_entry_as_read(entry.feed, entry.id).map_err(|e| EntryReadError::ModifyDatabaseError(e));
}

fn get_feed_entries(feed_name: String) -> Vec<Entry> {
    let entries = match read_entries(get_database_path()) {
        Err(e) => {
            eprintln!("Unable to read from database: {}", e);
            exit(1);
        }
        Ok(entries) => entries,
    };

    let mut feed_entries = Vec::new();
    for entry in entries {
        if entry.feed == feed_name {
            feed_entries.push(entry);
        }
    }

    feed_entries.sort();
    return feed_entries;
}

fn count_unread_entries() -> HashMap<String, u32> {
    let entries = match read_entries(get_database_path()) {
        Err(e) => {
            eprintln!("Unable to read from database: {}", e);
            exit(1);
        }
        Ok(entries) => entries,
    };

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

    return feed_entries;
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
        let mut feed_and_unread = Vec::from_iter(count_unread_entries().into_iter());
        feed_and_unread.sort_by(|a, b| (a.1, &a.0).cmp(&(b.1, &b.0)));
        for (feed_name, unread_count) in feed_and_unread {
            println!("\t{} {}", unread_count, feed_name);
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

            // FIXME: Also print saved update errors?
            for entry in get_feed_entries(feed_name) {
                if !entry.read {
                    if let Err(e) = entry_read(entry) {
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
        if let Err(e) = modify_database(modifier, get_database_path()) {
            eprintln!("Failed to mark {} as read: {}", feed_name.clone(), e);
            exit(1);
        }
    } else {
        eprintln!("usage: feed-markasread <feed>");
        exit(1);
    }
}

fn exec_feed_config(args: Vec<String>) {
    const USAGE: &str = "usage: feed-config <database_file | feed_config_dir>";

    if args.len() != 2 {
        eprintln!("{}", USAGE);
        exit(1);
    }

    match args[1].as_str() {
        "database_file" => {
            println!("{}", get_database_path());
        }
        "feed_config_dir" => {
            let dir = get_feed_config_dir();
            match dir {
                Some(path) => {
                    println!("{}", path.to_string_lossy());
                }
                _ => {
                    eprintln!("Unable to find feed configuration directory");
                    exit(1);
                }
            }
        }
        _ => {
            eprintln!("Unknown config string {}", args[1]);
            eprintln!("{}", USAGE);
            exit(1);
        }
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
        Some("feed-config") => exec_feed_config(args),
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
