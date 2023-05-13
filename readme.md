# lolchive

local liminal page archiver

## Use

the fantoccini archiver uses fantoccini which for these purposes use the
geckodriver the basic archiver just uses reqwest

doesn't run on windows yet.

FantocciniArchiver (Fantoccini and a running geckodriver)

```rust
    use lolchive::web_archiver::FantocciniArchiver

    let url = "https://www.merriam-webster.com/dictionary/fantoccini";

    //use the connection string to pass in, this is where geckodriver is running
    let connection_string = "http://localhost:4444";

    //set up absolute path to where you want it to store archive
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

Basic Archiver (Just uses reqwest)

```rust
    use lolchive::web_archiver::BasicArchiver

    let url = "https://www.rust-lang.org/";
    let home_dir = dirs::home_dir().expect("Failed to get home directory");
    let new_dir = format!("{}{}", home_dir.to_str().unwrap(), "/Projects/archive_test");
    println!("{:?}", new_dir);
    let path = BasicArchiver::create_archive(url, &new_dir).await;
    println!("{:?}", path);
```
