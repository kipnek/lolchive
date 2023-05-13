use crate::{html::HtmlRecord, web_archiver::replace_encoded_chars};
use bytes::Bytes;
use url::Url;

///public function
/// takes in a url string (complete "https://example.com")
/// returns an HtmlDocument if good, reqwest error if bad
/// basically just assembles after making a client side call
pub async fn fetch_html_record(url_str: &str) -> Result<HtmlRecord, reqwest::Error> {
    let url_parsed = Url::parse(url_str).expect("cannot parse");
    let res = reqwest::get(url_parsed.as_str()).await?;
    let _status_value = res.status().as_u16();
    let body = res.text().await.expect("unable to parse html text");
    let body = replace_encoded_chars(body);
    let record: HtmlRecord = HtmlRecord::new(url_parsed.to_string(), body);

    Ok(record)
}

pub async fn fetch_image_bytes(url_str: &str) -> Result<Bytes, String> {
    let url_parsed = Url::parse(url_str).expect("cannot parse");
    let res = reqwest::get(url_parsed.as_str())
        .await
        .expect("error getting requested image");

    let status_value = res.status().as_u16();

    if status_value == 200 {
        let image_value = res.bytes().await.expect("unable to parse html text");
        Ok(image_value)
    } else {
        Err("status on image call not a 200 OKAY".to_string())
    }
}

pub async fn fetch_string_resource(url_str: &str) -> Result<String, String> {
    let url_parsed = Url::parse(url_str).expect("cannot parse");
    let res = reqwest::get(url_parsed.as_str())
        .await
        .expect("error getting requested image");
    let status_value = res.status().as_u16();

    if status_value == 200 {
        let css = res.text().await.expect("unable to parse html text");
        Ok(css)
    } else {
        Err("status on css call not a 200 OKAY".to_string())
    }
}
