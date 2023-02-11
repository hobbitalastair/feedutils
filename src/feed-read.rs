use std::process::exit;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() >= 2 {
        let mut ok = true;
        for feed_name in args.into_iter().skip(1) {
            if let Err(e) = feedutil::get_feed_dir(feed_name.clone()) {
                eprintln!("{}: {}", e, feed_name.clone());
                ok = false;
                continue;
            }

            let entries = match feedutil::get_feed_entries(feed_name.clone()) {
                Ok(entries) => entries,
                Err(e) => {
                    eprintln!("{}", e);
                    exit(1);
                },
            };
            for entry in entries {
                if !entry.read {
                    if let Err(e) = feedutil::read_entry(feed_name.clone(), entry) {
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
