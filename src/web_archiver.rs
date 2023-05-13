use crate::client::*;
use crate::html::HtmlRecord;
use fantoccini::{Client, ClientBuilder};
use image;
use rand::{distributions::Alphanumeric, Rng};
use serde_json::{json, Map, Value};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::time::Duration;
use url::Url;

pub struct FantocciniArchiver {
    pub fclient: Client,
}

impl FantocciniArchiver {
    pub async fn new(connection_string: &str) -> Result<Self, String> {
        let client = ClientBuilder::native()
            .capabilities(get_capabilities())
            .connect(connection_string)
            .await
            .unwrap_or_else(|_| panic!("failed to connect to WebDriver on {}", connection_string));

        Ok(FantocciniArchiver { fclient: client })
    }

    pub async fn create_archive(&self, url: &str, path: &str) -> Result<String, String> {
        if self.fclient.goto(url).await.is_err() {
            return Err(format!("could not go to url {}", url));
        }
        let _ = self.fclient.wait().at_most(Duration::from_secs(10));

        let body = self
            .fclient
            .source()
            .await
            .unwrap_or_else(|_| panic!("can't carse to html {}", url));

        let body = replace_encoded_chars(body);

        let record = HtmlRecord::new(url.to_string(), body);

        if let Ok(screen_shot) = self.fclient.screenshot().await {
            match save_page(record, path, Some(screen_shot)).await {
                Ok(archive_path) => Ok(archive_path),
                Err(e) => Err(e),
            }
        } else {
            match save_page(record, path, None).await {
                Ok(archive_path) => Ok(archive_path),
                Err(e) => Err(e),
            }
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

            let body: String;

            if let Ok(str) = self.fclient.source().await {
                body = replace_encoded_chars(str);
            } else {
                continue;
            }

            let record = HtmlRecord::new(url.to_string(), body);

            if let Ok(image) = self.fclient.screenshot().await {
                if let Ok(archive_path) = save_page(record, path, Some(image)).await {
                    path_vector.push(archive_path);
                }
            } else if let Ok(archive_path) = save_page(record, path, None).await {
                path_vector.push(archive_path);
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
            .unwrap_or_else(|_| panic!("fetch_html_record failed \n url {}", url));

        match save_page(record, path, None).await {
            Ok(archive_path) => Ok(archive_path),
            Err(e) => Err(e),
        }
    }
}

pub async fn save_page(
    html_record: HtmlRecord,
    base_path: &str,
    screenshot: Option<Vec<u8>>,
) -> Result<String, String> {
    let mut body = html_record.body.clone();
    let url = Url::parse(&html_record.origin).unwrap();
    let root_host_name = url.host().unwrap().to_string();
    let mut a_path = url.path().to_string();
    let mut base_path = base_path.to_string();

    if !base_path.ends_with('/') {
        base_path.push('/');
    }
    if !a_path.ends_with('/') {
        a_path.push('/');
    }

    let directory = format!(
        "{}{}{}{}",
        base_path, root_host_name, a_path, html_record.date_time
    );

    assert!(fs::create_dir_all(directory.clone()).is_ok());

    if let Some(t_image_links) = html_record.get_image_links() {
        assert!(fs::create_dir_all(format!("{}/images", directory)).is_ok());
        for link in t_image_links {
            if let Ok(img_bytes) = fetch_image_bytes(&link.1).await {
                if let Ok(tmp_image) = image::load_from_memory(&img_bytes) {
                    if let Some(file_name) = get_file_name(&link.1) {
                        let fqn = format!("{}/images/{}", directory, file_name);
                        if tmp_image.save(fqn).is_ok() {
                            let body_replacement_text = format!("./images/{}", file_name);
                            body = body.replace(&link.0, &body_replacement_text);
                        }
                    } else {
                        let mut file_name = random_name_generator();
                        file_name.push_str(".png");
                        let fqn = format!("{}/images/{}", directory, file_name);

                        if tmp_image
                            .save_with_format(fqn, image::ImageFormat::Png)
                            .is_ok()
                        {
                            let body_replacement_text = format!("./images/{}", file_name);
                            body = body.replace(&link.0, &body_replacement_text);
                        }
                    }
                }
            }
        }
    }

    //get css
    if let Some(t_css_links) = html_record.get_css_links() {
        assert!(fs::create_dir_all(format!("{}/css", directory)).is_ok());
        for link in t_css_links {
            let file_name = match get_file_name(&link.1) {
                Some(e) => e,
                None => {
                    let mut file = random_name_generator();
                    file.push_str(".css");
                    file
                }
            };
            if let Ok(css) = fetch_string_resource(&link.1).await {
                let fqn = format!("{}/css/{}", directory, file_name);
                let mut file = File::create(fqn).unwrap();
                if file.write(css.as_bytes()).is_ok() {
                    let body_replacement_text = format!("./css/{}", file_name);
                    body = body.replace(&link.0, &body_replacement_text);
                }
            }
        }
    }

    //get js
    if let Some(t_js_links) = html_record.get_js_links() {
        assert!(fs::create_dir(format!("{}/js", directory)).is_ok());
        for link in t_js_links {
            let file_name = match get_file_name(&link.1) {
                Some(e) => e,
                None => {
                    let mut file = random_name_generator();
                    file.push_str(".js");
                    file
                }
            };
            if let Ok(css) = fetch_string_resource(&link.1).await {
                let fqn = format!("{}/js/{}", directory, file_name);

                if let Ok(mut output) = File::create(fqn) {
                    if output.write(css.as_bytes()).is_ok() {
                        let body_replacement_text = format!("./js/{}", file_name);
                        body = body.replace(&link.0, &body_replacement_text);
                    }
                }
            }
        }
    }
    //write screenshot
    if let Some(image) = screenshot {
        let fqn_png = format!("{}/screenshot.png", directory);
        let mut file_png = File::create(fqn_png).unwrap();
        assert!(file_png.write(&image).is_ok());
    }

    //write html
    let fqn_html = format!("{}/index.html", directory);
    let mut file_html = File::create(fqn_html.clone()).unwrap();
    if file_html.write(body.as_bytes()).is_ok() {
        Ok(fqn_html)
    } else {
        Err("error archiving site".to_string())
    }
}

fn get_file_name(link: &str) -> Option<String> {
    let urlp = Url::parse(link).unwrap();

    if urlp.query().is_some() {
        None
    } else {
        let segment_vector = urlp.path_segments().map(|c| c.collect::<Vec<_>>()).unwrap();
        let segment_file = *segment_vector.last().unwrap();
        Some(segment_file.to_string())
    }
}

pub fn get_capabilities() -> Map<String, Value> {
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

fn random_name_generator() -> String {
    let s: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();

    s
}

pub fn replace_encoded_chars(body: String) -> String {
    body.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&amp;", "&")
        .replace("&apos", "\'")
}
