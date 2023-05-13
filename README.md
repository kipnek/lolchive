# lolchive

local liminal page archiver

## doesn't work on windows yet

this will save webpages to your computer to the path you specify
so

```
google.com/path/to/this
```
is
```
google.com/
        |_/path
            |_/to
                |_/this
                    |_/date
                        |/css
                        |/images
                        |/js
                        |_index.html

```
will be the folder path.

## Use

the fantoccini archiver uses fantoccini which for these purposes use the
geckodriver the basic archiver just uses reqwest

FantocciniArchiver

```rust
    use lolchive::web_archiver::FantocciniArchiver
    use dirs;

    let url = "https://www.merriam-webster.com/dictionary/fantoccini";

    //use the connection string to pass in, this is where geckodriver is running
    let connection_string = "http://localhost:4444";

    //set up absolute pathe to where you want it to store archive
    let home_dir = dirs::home_dir().expect("Failed to get home directory");
    let new_dir = format!("{}{}", home_dir.to_str().unwrap(), "/Projects/archive_test");

    //create archiver
    let archiver = FantocciniArchiver::new(connection_string).await;

    //archive
    let path = archiver.create_archive(url, &new_dir).await;

    //path to the archive returned
    println!("{:?}", path);

    //close archiver
    let _ = archiver.close().await;
```

Basic Archiver
the basic archiver just uses reqwest
```rust
    use lolchive::web_archiver::BasicArchiver
    use dirs;

    let url = "https://www.rust-lang.org/";
    let home_dir = dirs::home_dir().expect("Failed to get home directory");
    let new_dir = format!("{}{}", home_dir.to_str().unwrap(), "/Projects/archive_test");
    println!("{:?}", new_dir);
    let path = BasicArchiver::create_archive(url, &new_dir).await;
    println!("{:?}", path);
```

## Crawler

Fantoccini Crawler - uses fantoccini and the gecko webdriver

```rust
            use lolchive::crawler::FantocciniCrawler;
            use dirs;
            
            let url = "https://en.wikipedia.org/wiki/Rust_(programming_language)";
            let connection_string = "http://localhost:4444";
            let home_dir = dirs::home_dir().expect("Failed to get home directory");
            let new_dir = format!("{}{}", home_dir.to_str().unwrap(), "/Projects/archive_test");
            let fcrawler = FantocciniCrawler::new(connection_string).await.unwrap();
            let paths = fcrawler.save_crawl(url, &new_dir, 2).await.unwrap();
            let _ = fcrawler.close().await;

            println!("{:?}", paths);
            assert!(paths.len() == 2);

```

Basic Crawler - uses reqwest

```rust
            use lolchive::crawler::BasicCrawler;
            use dirs;

            let url = "https://www.rust-lang.org/";
            let home_dir = dirs::home_dir().expect("Failed to get home directory");
            let new_dir = format!("{}{}", home_dir.to_str().unwrap(), "/Projects/archive_test");
            let paths = BasicCrawler::save_crawl(url, &new_dir, 2).await.unwrap();

            println!("{:?}", paths);
            assert!(paths.len() == 2);

```
