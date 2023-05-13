use std::time::Duration;

use crate::client;
use crate::html::HtmlRecord;
use crate::web_archiver::{get_capabilities, replace_encoded_chars, save_page};
use fantoccini::{Client, ClientBuilder};

pub struct FantocciniCrawler {
    pub fclient: Client,
}

impl FantocciniCrawler {
    pub async fn new(connection_string: &str) -> Result<Self, String> {
        let client = ClientBuilder::native()
            .capabilities(get_capabilities())
            .connect(connection_string)
            .await
            .unwrap_or_else(|_| panic!("failed to connect to WebDriver on {}", connection_string));

        Ok(FantocciniCrawler { fclient: client })
    }

    pub async fn save_crawl(
        &self,
        url: &str,
        directory: &str,
        num_of_pages: usize,
    ) -> Result<Vec<String>, String> {
        let mut visited: Vec<String> = vec![url.to_string()];
        let mut i: usize = 0;
        let mut ret_vec: Vec<String> = vec![];

        while i < num_of_pages && i < visited.len() {
            println!("{:?}", visited[i]);
            if self.fclient.goto(&visited[i]).await.is_err() {
                i += 1;
                continue;
            }
            let _ = self.fclient.wait().at_most(Duration::from_secs(10));

            if let Ok(body) = self.fclient.source().await {
                let body = replace_encoded_chars(body);
                let record = HtmlRecord::new(visited[i].to_string(), body);
                if let Some(links) = record.domain_anchors() {
                    for link in links {
                        if !visited.contains(&link) {
                            visited.push(link)
                        }
                    }
                }

                if let Ok(image) = self.fclient.screenshot().await {
                    if let Ok(path) = save_page(record, directory, Some(image)).await {
                        ret_vec.push(path);
                    }
                } else if let Ok(path) = save_page(record, directory, None).await {
                    ret_vec.push(path);
                }
            }
            i += 1;
        }
        Ok(ret_vec)
    }
    pub async fn close(self) -> Result<(), String> {
        if self.fclient.close().await.is_ok() {
            Ok(())
        } else {
            Err(
                "closing the archiver did not work, exit the program and restart the geckodriver"
                    .to_string(),
            )
        }
    }
}

pub struct BasicCrawler {}

impl BasicCrawler {
    pub async fn save_crawl(
        url: &str,
        directory: &str,
        num_of_pages: usize,
    ) -> Result<Vec<String>, String> {
        let mut visited: Vec<String> = vec![url.to_string()];
        let mut i: usize = 0;
        let mut ret_vec: Vec<String> = vec![];

        while i < num_of_pages && i < visited.len() {
            if let Ok(record) = client::fetch_html_record(&visited[i]).await {
                if let Some(links) = record.domain_anchors() {
                    for link in links {
                        if !visited.contains(&link) {
                            visited.push(link)
                        }
                    }
                }
                if let Ok(path) = save_page(record, directory, None).await {
                    ret_vec.push(path);
                }
            }
            i += 1;
        }
        Ok(ret_vec)
    }
}
