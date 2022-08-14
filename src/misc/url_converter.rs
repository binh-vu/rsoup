use anyhow::Result;
use url::Url;

use crate::models::rich_text::RichText;

pub struct URLConverter {
    url: Url,
}

impl URLConverter {
    pub fn new(raw_url: String) -> Result<Self> {
        let url = Url::parse(&raw_url)?;
        Ok(URLConverter { url })
    }

    #[inline]
    pub fn is_absolute(&self, url: &str) -> bool {
        // url[..url.len().max(10)].find("://").is_some()
        url.starts_with("http://") || url.starts_with("https://")
    }

    #[inline]
    pub fn to_absolute(&self, relative_url: &str) -> Result<String> {
        if relative_url.starts_with("//") {
            Ok(format!("{}:{}", self.url.scheme(), relative_url))
        } else if relative_url.starts_with("/") {
            Ok(format!(
                "{}://{}{}",
                self.url.scheme(),
                self.url.host_str().unwrap(),
                relative_url
            ))
        } else if relative_url.starts_with(".") {
            Ok(self.url.join(relative_url)?.as_str().to_owned())
        } else {
            Ok(relative_url.to_owned())
        }
    }

    #[inline]
    pub fn normalize_rich_text(&self, rich_text: &mut RichText) {
        for element in rich_text.element.iter_mut() {
            if element.tag == "a" {
                if let Some(href) = element.attrs.get("href") {
                    if !self.is_absolute(href) {
                        if let Ok(href) = self.to_absolute(href) {
                            element.attrs.insert("href".to_owned(), href);
                        }
                    }
                }
            }
        }
    }
}
