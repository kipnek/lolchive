mod client;
pub mod html;
pub mod web_archiver;

//archiver tests
#[cfg(test)]
mod tests {
    use crate::web_archiver::{BasicArchiver, FantocciniArchiver};
    use dirs;

    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    #[test]
    fn fantoccini_single() {
        aw!(async {
            let url = "https://funnyjunk.com";
            let connection_string = "http://localhost:4444";
            let home_dir = dirs::home_dir().expect("Failed to get home directory");
            let new_dir = format!("{}{}", home_dir.to_str().unwrap(), "/Projects/archive_test");

            let archiver = FantocciniArchiver::new(connection_string).await;

            let path = archiver.create_archive(url, &new_dir).await;
            println!("{:?}", path);
            let _ = archiver.close().await;
        });
    }

    #[test]
    fn fantoccini_multiple() {
        aw!(async {
            let urls = vec![
                "https://www.reddit.com/r/rust/",
                "https://www.rust-lang.org/",
            ];
            let connection_string = "http://localhost:4444";
            let home_dir = dirs::home_dir().expect("Failed to get home directory");
            let new_dir = format!("{}{}", home_dir.to_str().unwrap(), "/Projects/archive_test");

            let archiver = FantocciniArchiver::new(connection_string).await;

            let paths = archiver.create_archives(urls, &new_dir).await;
            println!("{:?}", paths);

            let _ = archiver.close().await;
        });
    }

    #[test]
    fn basic() {
        aw!(async {
            let url = "https://www.rust-lang.org/";
            let home_dir = dirs::home_dir().expect("Failed to get home directory");
            let new_dir = format!("{}{}", home_dir.to_str().unwrap(), "/Projects/archive_test");
            println!("{:?}", new_dir);
            let path = BasicArchiver::create_archive(url, &new_dir).await;
            println!("{:?}", path);
        });
    }
}
