use std::process::exit;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        let entry_counts = match feedutil::count_unread_entries() {
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
