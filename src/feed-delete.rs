use std::fs;
use std::process::exit;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 2 {
        let feed_name = args[1].clone();

        let feed_dir = match feedutil::get_feed_dir(feed_name.clone()) {
            Ok(feed_dir) => feed_dir,
            Err(e) => {
                eprintln!("{}: {}", e, feed_name.clone());
                exit(1);
            },
        };

        let database_path = match feedutil::get_database_path() {
            Ok(path) => path,
            Err(e) => {
                eprintln!("{}", e);
                exit(1);
            },
        };

        let modifier = |entries: Vec<feedutil::Entry>| -> Vec<feedutil::Entry> {
            let mut modified_entries: Vec<feedutil::Entry> = Vec::new();
            for entry in entries {
                if entry.feed != feed_name {
                    modified_entries.push(entry);
                }
            }
            return modified_entries;
        };
        if let Err(e) = feedutil::modify_database(modifier, database_path) {
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
