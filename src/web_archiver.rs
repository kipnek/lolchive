use crate::client::*;
use crate::html::HtmlRecord;
use fantoccini::{Client, ClientBuilder};
use image;
use serde_json::{json, map, value, Map, Value};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::time::Duration;
use thirtyfour::Capabilities;
use url::Url;

pub struct FantocciniArchiver {
    pub fclient: Client,
}

impl FantocciniArchiver {
    pub async fn new(connection_string: &str) -> Self {
        let client = ClientBuilder::native()
            .capabilities(get_capabilities())
            .connect(connection_string)
            .await
            .expect(&format!(
                "failed to connect to WebDriver on {}",
                connection_string
            ));
        FantocciniArchiver { fclient: client }
    }

    pub async fn create_archive(&self, url: &str, path: &str) -> Result<String, String> {
        if self.fclient.goto(url).await.is_err() {
            return Err(format!("could not go to url {}", url));
        }
        let _ = self.fclient.wait().at_most(Duration::from_secs(10));
        let screen_shot = self
            .fclient
            .screenshot()
            .await
            .expect("could not screenshot");
        let body = self.fclient.source().await.expect("can't carse to html");

        let record = HtmlRecord::new(url.to_string(), body);
        match save_page(record, path, screen_shot).await {
            Ok(archive_path) => Ok(archive_path),
            Err(e) => Err(e),
        }
    }

    pub async fn create_archives(
        &self,
        urls: Vec<&str>,
        path: &str,
    ) -> Result<Vec<String>, String> {
        let mut path_vector: Vec<String> = vec![];

        for url in urls {
            if self.fclient.goto(url).await.is_err() {
                return Err(format!("could not go to url {}", url));
            }
            let _ = self.fclient.wait().at_most(Duration::from_secs(10));

            let body = self.fclient.source().await.expect("can't carse to html");

            let record = HtmlRecord::new(url.to_string(), body);
            match save_page(record, path, vec![]).await {
                Ok(archive_path) => {
                    path_vector.push(archive_path);
                }
                Err(_) => {}
            }
        }
        Ok(path_vector)
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

pub struct BasicArchiver {}

impl BasicArchiver {
    pub async fn create_archive(url: &str, path: &str) -> Result<String, String> {
        let record = fetch_html_record(url)
            .await
            .expect(&format!("fetch_html_record failed \n url {}", url));

        match save_page(record, path, vec![]).await {
            Ok(archive_path) => Ok(archive_path),
            Err(e) => Err(e),
        }
    }
}

async fn save_page(
    html_document: HtmlRecord,
    base_path: &str,
    screenshot: Vec<u8>,
) -> Result<String, String> {
    let mut body = html_document.body.clone();
    let url = Url::parse(&html_document.origin).unwrap();
    let root_host_name = url.host().unwrap().to_string();
    let path = url.path();
    let mut base_path = base_path.to_string();

    if !base_path.ends_with('/') {
        base_path.push_str("/");
    }

    let mut directory = format!("{}{}", base_path, root_host_name);

    if !path.ends_with('/') {
        directory.push_str(&format!("{}/{}", path, html_document.date_time));
    } else {
        directory.push_str(&format!("{}{}", path, html_document.date_time));
    }

    assert!(fs::create_dir_all(directory.clone()).is_ok());

    if let Some(t_image_links) = html_document.get_image_links() {
        assert!(fs::create_dir_all(format!("{}/images", directory)).is_ok());
        for link in t_image_links {
            let file_name = get_file_name(&link.1);
            if let Ok(img_bytes) = fetch_image_bytes(&link.1).await {
                if let Ok(tmp_image) = image::load_from_memory(&img_bytes) {
                    let fqn = format!("{}/images/{}", directory, file_name);
                    if tmp_image.save(fqn).is_ok() {
                        let body_replacement_text = format!("./images/{}", file_name);
                        body = body.replace(&link.0, &body_replacement_text);
                    }
                }
            }
        }
    }

    //get css
    if let Some(t_css_links) = html_document.get_css_links() {
        assert!(fs::create_dir_all(format!("{}/css", directory)).is_ok());
        for link in t_css_links {
            let file_name = get_file_name(&link.1);
            if let Ok(css) = fetch_string_resource(&link.1).await {
                let fqn = format!("{}/css/{}", directory, file_name);
                let mut output = File::create(fqn).unwrap();
                if output.write(css.as_bytes()).is_ok() {
                    let body_replacement_text = format!("./css/{}", file_name);
                    body = body.replace(&link.0, &body_replacement_text);
                }
            }
        }
    }

    //get js
    if let Some(t_js_links) = html_document.get_js_links() {
        assert!(fs::create_dir(format!("{}/js", directory)).is_ok());
        for t_js_link in t_js_links {
            let file_name = get_file_name(&t_js_link.1);
            if let Ok(css) = fetch_string_resource(&t_js_link.1).await {
                let fqn = format!("{}/js/{}", directory, file_name);

                if let Ok(mut output) = File::create(fqn) {
                    if output.write(css.as_bytes()).is_ok() {
                        let body_replacement_text = format!("./js/{}", file_name);
                        body = body.replace(&t_js_link.0, &body_replacement_text);
                    }
                }
            }
        }
    }
    //write screenshot
    if !screenshot.is_empty() {
        let fqn_png = format!("{}/screenshot.png", directory);
        let mut file_png = File::create(fqn_png.clone()).unwrap();
        assert!(file_png.write(&screenshot).is_ok());
    }

    //write html
    let fqn_html = format!("{}/index.html", directory);
    let mut file_html = File::create(fqn_html.clone()).unwrap();
    if file_html.write(body.as_bytes()).is_ok() {
        Ok(fqn_html.to_string())
    } else {
        Err("error archiving site".to_string())
    }
}

fn get_file_name(link: &str) -> String {
    let urlp = Url::parse(link).unwrap();
    let segment_vector = urlp.path_segments().map(|c| c.collect::<Vec<_>>()).unwrap();
    let segment_file = *segment_vector.last().unwrap();
    segment_file.to_string()
}

fn get_capabilities() -> Map<String, Value> {
    let mut caps = Map::new();
    let mut firefox_options = Map::new();

    let mut firefox_profile = Map::new();
    firefox_profile.insert(
        "browser.privatebrowsing.autostart".to_string(),
        Value::Bool(true),
    );
    /*
    firefox_profile.insert(
        "network.cookie.cookieBehavior".to_string(),
        Value::Number(1.into()),
    );
    firefox_profile.insert(
        "dom.webnotifications.enabled".to_string(),
        Value::Bool(false),
    );
    firefox_profile.insert(
        "media.volume_scale".to_string(),
        Value::String("0.0".to_string()),
    );*/
    //firefox_profile.insert("general.useragent.override".to_string(), Value::String("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36 Edge/16.16299".to_string()));
    //firefox_profile.insert("javascript.enabled".to_string(), Value::Bool(true));
    //firefox_profile.insert("geo.enabled".to_string(), Value::Bool(false));
    /*firefox_profile.insert(
        "geo.provider.network.url".to_string(),
        Value::String("data:,".to_string()),
    );
    firefox_profile.insert(
        "geo.wifi.uri".to_string(),
        Value::String("data:,".to_string()),
    );*/

    firefox_options.insert(
        "args".to_string(),
        json!([
            "--headless",
            "--disable-blink-features=AutomationControlled"
        ]),
    );
    firefox_options.insert("prefs".to_string(), Value::Object(firefox_profile));
    caps.insert(
        "moz:firefoxOptions".to_string(),
        Value::Object(firefox_options),
    );
    caps
}