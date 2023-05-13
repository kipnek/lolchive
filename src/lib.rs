mod client;
pub mod crawler;
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
            let url = "https://en.wikipedia.org/wiki/Rust_(programming_language)";
            let connection_string = "http://localhost:4444";
            let home_dir = dirs::home_dir().expect("Failed to get home directory");
            let new_dir = format!("{}{}", home_dir.to_str().unwrap(), "/Projects/archive_test");
            let archiver = FantocciniArchiver::new(connection_string).await;

            match archiver.create_archive(url, &new_dir).await {
                Ok(path) => {
                    let _ = archiver.close().await;
                    assert!(path.len() > 0)
                }
                Err(e) => {
                    let _ = archiver.close().await;
                    println!("{:?}", e);
                    assert!(false);
                }
            }
        });
    }

    #[test]
    fn fantoccini_multiple() {
        aw!(async {
            let urls = vec![
                "https://www.reddit.com/r/rust/",
                "https://en.wikipedia.org/wiki/Rust_(programming_language)",
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

    #[test]
    fn test_get_string() {
        aw!(async {
            let url = "https://en.wikipedia.org/w/load.php?lang=en&modules=site.styles&only=styles&skin=vector-2022";
            let home_dir = dirs::home_dir().expect("Failed to get home directory");
            let new_dir = format!("{}{}", home_dir.to_str().unwrap(), "/Projects/archive_test");

            assert!(BasicArchiver::create_archive(url, &new_dir).await.is_ok());
        });
    }
}
