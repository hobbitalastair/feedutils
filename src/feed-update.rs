use std::process::exit;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let feeds = match args.len() {
        1 => {
            match feedutil::get_all_feed_names() {
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
        let res = feedutil::update(feed_name);
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
