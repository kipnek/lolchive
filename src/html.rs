use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use scraper::{Html, Selector};
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use substring::Substring;
use url::{ParseError, Url};

#[derive(Debug)]
pub struct HtmlRecord {
    pub origin: String,
    pub date_time: String,
    pub body: String,
    pub html: Html,
}

/*

HTML DOCUMENT

*/

impl HtmlRecord {
    pub fn new(origin: String, body: String) -> HtmlRecord {
        HtmlRecord {
            origin,
            date_time: Utc::now().format("%d-%m-%Y-%H:%M:%S").to_string(),
            html: Html::parse_document(&body),
            body,
        }
    }

    /// public function,
    /// input is a reference to self.
    /// returns a HashSet<String>,
    /// transforms body of HtmlRecord to collect all anchors.
    pub fn anchors(&self) -> Option<HashSet<String>> {
        let mut ret_vec: Vec<String> = vec![];
        let selector = Selector::parse("a").unwrap();
        for element in self.html.select(&selector) {
            match element.value().attr("href") {
                Some(link) => {
                    if let Ok(parse_link) = HtmlRecord::check_link(&self.origin, link) {
                        ret_vec.push(parse_link)
                    }
                }
                None => continue,
            };
        }

        let link_hashset: HashSet<String> = ret_vec.iter().cloned().collect();

        if link_hashset.is_empty() {
            None
        } else {
            Some(link_hashset)
        }
    }

    /// public function:
    /// input is a reference to self.
    /// returns an Option<HashSet<String>>,
    /// transforms body of HtmlRecord to collect domain related anchors.
    pub fn domain_anchors(&self) -> Option<HashSet<String>> {
        let mut ret_vec: Vec<String> = vec![];
        let selector = Selector::parse("a").unwrap();
        for element in self.html.select(&selector) {
            match element.value().attr("href") {
                Some(link) => {
                    if let Ok(parsed_link) = HtmlRecord::check_link(&self.origin, link) {
                        if HtmlRecord::is_host_related(&self.origin, &parsed_link)
                            && HtmlRecord::is_http(&parsed_link)
                            && !HtmlRecord::has_extension(&parsed_link)
                        {
                            ret_vec.push(parsed_link)
                        }
                    }
                }
                None => continue,
            };
        }

        let link_hashset: HashSet<String> = ret_vec.iter().cloned().collect();

        if link_hashset.is_empty() {
            None
        } else {
            Some(link_hashset)
        }
    }

    /// public function:
    /// input is a reference to self.
    /// returns a HashSet<String>
    /// transforms body of HtmlRecord to collect non domain related anchors
    pub fn non_domain_anchors(&self) -> Option<HashSet<String>> {
        let mut ret_vec: Vec<String> = vec![];
        let selector = Selector::parse("a").unwrap();

        for element in self.html.select(&selector) {
            match element.value().attr("href") {
                Some(link) => {
                    if let Ok(link) = HtmlRecord::check_link(&self.origin, link) {
                        if !HtmlRecord::is_host_related(&self.origin, &link)
                            && HtmlRecord::is_http(&link)
                            && !HtmlRecord::has_extension(&link)
                        {
                            ret_vec.push(link)
                        }
                    }
                }
                None => continue,
            };
        }

        let link_hashset: HashSet<String> = ret_vec.iter().cloned().collect();

        if link_hashset.is_empty() {
            None
        } else {
            Some(link_hashset)
        }
    }

    ///public function:
    /// inputs: &self and regex &str.
    /// returns a hashset<string>,
    /// used for when trying to crawl things like online threads.
    pub fn anchors_curate(&self, regex: &str) -> Option<HashSet<String>> {
        let mut ret_vec: Vec<String> = vec![];
        if let Ok(re) = Regex::new(regex) {
            let selector = Selector::parse("a").unwrap();
            for element in self.html.select(&selector) {
                match element.value().attr("href") {
                    Some(link) => {
                        if let Ok(parsed_link) = HtmlRecord::check_link(&self.origin, link) {
                            if re.is_match(&parsed_link) && !HtmlRecord::has_extension(&parsed_link)
                            {
                                ret_vec.push(parsed_link);
                            }
                        }
                    }
                    None => continue,
                };
            }
            let link_hashset: HashSet<String> = ret_vec.iter().cloned().collect();

            if link_hashset.is_empty() {
                None
            } else {
                Some(link_hashset)
            }
        } else {
            None
        }
    }

    ///public method,
    /// gets the text of a tag.
    pub fn tag_text(&self, tag: &str) -> Option<Vec<String>> {
        let mut ret_vec: Vec<String> = vec![];
        let re = Regex::new(r"\n|\t").unwrap();
        let selector = Selector::parse(tag).unwrap();
        for element in self.html.select(&selector) {
            let text_raw = element.text().collect::<String>(); //Vec<_>>();
            let text_parsed = re.replace_all(&text_raw, "").to_string();
            if !text_parsed.is_empty() {
                ret_vec.push(String::from(text_parsed.trim()));
            }
        }
        if ret_vec.is_empty() {
            None
        } else {
            Some(ret_vec)
        }
    }

    ///public method,
    /// gets the html of a tag.
    pub fn tag_html(&self, tag: &str) -> Option<Vec<String>> {
        let mut ret_vec: Vec<String> = vec![];
        let selector = Selector::parse(tag).unwrap();
        for element in self.html.select(&selector) {
            ret_vec.push(element.html());
            println!("{:?}", element.html())
        }
        if !ret_vec.is_empty() {
            Some(ret_vec)
        } else {
            None
        }
    }

    /// public function:
    /// returns HashMap<String, Vec<String>>.
    /// takes in reference to self,
    /// use to map out the meta information in html string
    pub fn html_meta(&self) -> Option<HashMap<String, Vec<String>>> {
        let mut meta_hash: HashMap<String, Vec<String>> = HashMap::new();
        let meta_selector = Selector::parse("meta").unwrap();
        for meta_element in self.html.select(&meta_selector) {
            let name = meta_element.value().attr("name").unwrap_or("Content-Type");
            let content = match meta_element.value().attr("content") {
                Some(content) => Vec::from_iter(content.split(',').map(String::from)),
                None => vec!["none".to_string()],
            };
            meta_hash.insert(String::from(name), content);
        }

        if meta_hash.is_empty() {
            None
        } else {
            Some(meta_hash)
        }
    }

    ///get_emails()->Option<Vec<String>>
    ///returns all emails withing a body.
    pub fn get_emails(&self) -> Option<HashSet<String>> {
        lazy_static! {
            static ref RE: Regex =
               Regex::new(r"([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})").unwrap();
        }
        // iterate over all matches
        let emails: HashSet<String> = RE
            .find_iter(&self.body)
            // try to parse the string matches as i64 (inferred from fn type signature)
            // and filter out the matches that can't be parsed (e.g. if there are too many digits to store in an i64).
            .map(|email| String::from(email.as_str()))
            // collect the results in to a Vec<i64> (inferred from fn type signature)
            .collect();

        if emails.is_empty() {
            None
        } else {
            Some(emails)
        }
    }

    ///get_phone_numbers()->Option<Vec<String>>
    ///returns all phone numbers within a body.
    /// only works with us numbers
    pub fn get_phone_numbers(&self) -> Option<HashSet<String>> {
        lazy_static! {
            static ref RE2: Regex =
               Regex::new(r"^(?:(?:\+?1\s*(?:[.-]\s*)?)?(?:\(\s*([2-9]1[02-9]|[2-9][02-8]1|[2-9][02-8][02-9])\s*\)|([2-9]1[02-9]|[2-9][02-8]1|[2-9][02-8][02-9]))\s*(?:[.-]\s*)?)?([2-9]1[02-9]|[2-9][02-9]1|[2-9][02-9]{2})\s*(?:[.-]\s*)?([0-9]{4})(?:\s*(?:#|x\.?|ext\.?|extension)\s*(\d+))?$").unwrap();
        }

        // iterate over all matches
        let numbers: HashSet<String> = RE2
            .find_iter(&self.body)
            // try to parse the string matches as i64 (inferred from fn type signature)
            // and filter out the matches that can't be parsed (e.g. if there are too many digits to store in an i64).
            .map(|number| String::from(number.as_str()))
            // collect the results in to a Vec<i64> (inferred from fn type signature)
            .collect();

        if numbers.is_empty() {
            None
        } else {
            Some(numbers)
        }
    }

    //the tuple returns the unparsed string in the 0's spot
    //returns the parsed link in the 1's spot
    pub fn get_image_links(&self) -> Option<HashSet<(String, String)>> {
        lazy_static! {
            static ref RE3: Regex = Regex::new(r";base64,").unwrap();
        }
        let mut ret_vec: Vec<(String, String)> = vec![];
        let selector = Selector::parse("img").unwrap();
        for element in self.html.select(&selector) {
            match element.value().attr("src") {
                Some(link) => {
                    if Url::parse(link) == Err(ParseError::RelativeUrlWithoutBase) {
                        let base = Url::parse(&self.origin)
                            .expect("get css links, origin could not be parsed");
                        let plink = base
                            .join(link)
                            .expect("css links, could not join")
                            .to_string();
                        ret_vec.push((link.to_string(), plink.to_string()))
                    } else if RE3.is_match(link) {
                        continue;
                    } else if let Ok(parsed_link) = Url::parse(link) {
                        ret_vec.push((link.to_string(), parsed_link.to_string()));
                    }
                }
                None => continue,
            };
        }

        let link_hashset: HashSet<(String, String)> = ret_vec.iter().cloned().collect();

        if link_hashset.is_empty() {
            None
        } else {
            Some(link_hashset)
        }
    }

    pub fn get_css_links(&self) -> Option<HashSet<(String, String)>> {
        let mut ret_vec: Vec<(String, String)> = vec![];
        let selector = Selector::parse("link").unwrap();
        for element in self.html.select(&selector) {
            if element.value().attr("rel").unwrap() == "stylesheet" {
                match element.value().attr("href") {
                    Some(link) => {
                        //take care of relative links here
                        if Url::parse(link) == Err(ParseError::RelativeUrlWithoutBase) {
                            let base = Url::parse(&self.origin)
                                .expect("get css links, origin could not be parsed");
                            let plink = base
                                .join(link)
                                .expect("css links, could not join")
                                .to_string();
                            ret_vec.push((link.to_string(), plink.to_string()))
                        } else if let Ok(parsed_link) = Url::parse(link) {
                            ret_vec.push((link.to_string(), parsed_link.to_string()));
                        }
                    }
                    None => continue,
                };
            }
        }

        let link_hashset: HashSet<(String, String)> = ret_vec.iter().cloned().collect();

        if link_hashset.is_empty() {
            None
        } else {
            Some(link_hashset)
        }
    }

    pub fn get_js_links(&self) -> Option<HashSet<(String, String)>> {
        let mut ret_vec: Vec<(String, String)> = vec![];
        let selector = Selector::parse("script").unwrap();
        for element in self.html.select(&selector) {
            match element.value().attr("src") {
                Some(link) => {
                    if Url::parse(link) == Err(ParseError::RelativeUrlWithoutBase) {
                        let base = Url::parse(&self.origin)
                            .expect("get js links, origin could not be parsed ");
                        let plink = base
                            .join(link)
                            .expect("js links, could not join")
                            .to_string();
                        ret_vec.push((link.to_string(), plink.to_string()))
                    } else if let Ok(parsed_link) = Url::parse(link) {
                        ret_vec.push((link.to_string(), parsed_link.to_string()));
                    }
                }
                None => continue,
            };
        }

        let link_hashset: HashSet<(String, String)> = ret_vec.iter().cloned().collect();

        if link_hashset.is_empty() {
            None
        } else {
            Some(link_hashset)
        }
    }

    /*

    PRIVATES

    */

    /// private function
    /// check_string -> string of domain that you want to check against
    /// link -> string of link you want to check.
    /// returns a bool.
    /// use case: used as a conditional check to whether an HtmlDocument anchor href.
    /// is associated with the domain.
    /// needs refactoring.
    fn is_host_related(check_string: &str, link: &str) -> bool {
        let origin_url = Url::parse(check_string).expect("self is not a url");
        let url_to_check = Url::parse(link).expect("link is not a string");

        origin_url.host() == url_to_check.host()
    }

    ///private method: check_link
    /// input "origin", type &str, stands for an HtmlDocument anchor
    /// input "in_link", type &str, stands for an HtmlDocument anchor that may need parsing
    /// this is a cursory check to see if a parse is needed
    /// returns string
    fn check_link(origin: &str, in_link: &str) -> Result<String, std::string::ParseError> {
        match Url::parse(in_link) {
            Ok(link) => Ok(link.to_string()),
            Err(_) => HtmlRecord::parse_link(origin, in_link),
        }
    }

    /// cdprivate method
    /// input is the origin of the HtmlDocument, and the unparsed link
    /// the output is the parsed string
    /// use case
    /// get an anchor tag href where it is unparsed, as in "/"
    fn parse_link(origin: &str, unparsed_link: &str) -> Result<String, std::string::ParseError> {
        let host = Url::parse(origin).expect("origin not a url");
        let host_string = host.host_str().expect("host is not doable");

        let parsed_link = if unparsed_link.substring(0, 1) == "/" {
            format!("{}://{}{}", host.scheme(), host_string, unparsed_link)
        } else {
            format!("{}://{}/{}", host.scheme(), host_string, unparsed_link)
        };

        Ok(parsed_link)
    }

    /// private method
    /// input is link to run http(s) check
    /// use case:
    /// this is used as conditional method to whether its an actual http url
    fn is_http(link: &str) -> bool {
        let url = Url::parse(link).expect("is http failed");

        url.scheme() == "http" || url.scheme() == "https"
    }

    fn has_extension(link: &str) -> bool {
        let url = Url::parse(link).expect("is http failed");
        let extention_vec = Vec::from_iter(url.path().split('.'));

        EXTENTIONS.contains(&extention_vec[extention_vec.len() - 1])
    }
}
pub static EXTENTIONS: [&str; 7] = ["jpeg", "jpg", "css", "js", "webm", "webp", "png"];
