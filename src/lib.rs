mod client;
pub mod crawler;
pub mod html;
pub mod web_archiver;

macro_rules! aw {
    ($e:expr) => {
        tokio_test::block_on($e)
    };
}

//archiver tests
#[cfg(test)]
mod tests {
    use crate::web_archiver::{BasicArchiver, FantocciniArchiver};
    use dirs;

    #[test]
    fn fantoccini_single() {
        aw!(async {
            let url = "https://funnyjunk.com";
            let connection_string = "http://localhost:4444";
            let home_dir = dirs::home_dir().expect("Failed to get home directory");
            let new_dir = format!("{}{}", home_dir.to_str().unwrap(), "/Projects/archive_test");
            let archiver = FantocciniArchiver::new(connection_string).await;
            let path = archiver.create_archive(url, &new_dir).await.unwrap();
            let _ = archiver.close().await;

            assert!(path.len() > 0)
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
            let paths = archiver.create_archives(urls, &new_dir).await.unwrap();
            let _ = archiver.close().await;

            assert!(paths.len() > 0);
        });
    }

    #[test]
    fn basic() {
        aw!(async {
            let url = "https://www.rust-lang.org/";
            let home_dir = dirs::home_dir().expect("Failed to get home directory");
            let new_dir = format!("{}{}", home_dir.to_str().unwrap(), "/Projects/archive_test");
            println!("{:?}", new_dir);
            assert!(BasicArchiver::create_archive(url, &new_dir).await.is_ok());
        });
    }
}
