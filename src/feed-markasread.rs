use std::process::exit;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 2 {
        let feed_name = args[1].clone();
        if let Err(e) = feedutil::get_feed_dir(feed_name.clone()) {
            eprintln!("{}: {}", e, feed_name.clone());
            exit(1);
        }

        let database_path = match feedutil::get_database_path() {
            Ok(path) => path,
            Err(e) => {
                eprintln!("{}", e);
                exit(1);
            },
        };

        let modifier = |entries: Vec<feedutil::Entry>| -> Vec<feedutil::Entry> {
            let mut modified_entries: Vec<feedutil::Entry> = Vec::new();
            for mut entry in entries {
                if entry.feed == feed_name {
                    entry.read = true;
                }
                modified_entries.push(entry);
            }
            return modified_entries;
        };
        if let Err(e) = feedutil::modify_database(modifier, database_path) {
            eprintln!("Failed to mark {} as read: {}", feed_name.clone(), e);
            exit(1);
        }
    } else {
        eprintln!("usage: feed-markasread <feed>");
        exit(1);
    }
}
