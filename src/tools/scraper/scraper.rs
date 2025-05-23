use async_trait::async_trait;
use regex::Regex;
use schemars::JsonSchema;
use scraper::{ElementRef, Html, Selector};
use serde::Deserialize;
use std::error::Error;

use crate::tools::ToolFunction;

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(description = "The URL to scrape, MUST be a working URL")]
pub struct WebScrapperInput(pub String);

#[derive(Default)]
pub struct WebScrapper {}

#[async_trait]
impl ToolFunction for WebScrapper {
    type Input = WebScrapperInput;
    type Result = String;

    fn name(&self) -> String {
        "Web Scraper".into()
    }

    fn description(&self) -> String {
        "Scan a url and return the content of the web page.".into()
    }

    fn strict(&self) -> bool {
        true
    }

    async fn run(&self, input: Self::Input) -> Result<String, Box<dyn Error + Send + Sync>> {
        let url = input.0;
        match scrape_url(&url).await {
            Ok(content) => Ok(content),
            Err(e) => Ok(format!("Error scraping {url}: {e}")),
        }
    }
}

async fn scrape_url(url: &str) -> Result<String, Box<dyn Error>> {
    let res = reqwest::get(url).await?.text().await?;

    let document = Html::parse_document(&res);
    let body_selector = Selector::parse("body").unwrap();

    let mut text = Vec::new();
    for element in document.select(&body_selector) {
        collect_text_not_in_script(&element, &mut text);
    }

    let joined_text = text.join(" ");
    let cleaned_text = joined_text.replace(['\n', '\t'], " ");
    let re = Regex::new(r"\s+").unwrap();
    let final_text = re.replace_all(&cleaned_text, " ");
    Ok(final_text.to_string())
}

fn collect_text_not_in_script(element: &ElementRef, text: &mut Vec<String>) {
    for node in element.children() {
        if node.value().is_element() {
            let tag_name = node.value().as_element().unwrap().name();
            if tag_name == "script" {
                continue;
            }
            collect_text_not_in_script(&ElementRef::wrap(node).unwrap(), text);
        } else if node.value().is_text() {
            text.push(node.value().as_text().unwrap().text.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};
    use tokio;

    use super::*;

    #[tokio::test]
    async fn test_scrape_url() {
        // Request a new server from the pool
        let mut server = mockito::Server::new_async().await;

        // Create a mock on the server
        let mock = server
            .mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body("<html><body>Hello World</body></html>")
            .create();

        // Instantiate your WebScrapper
        let scraper = WebScrapper::default();

        // Use the server URL for scraping
        let url = server.url();

        // Call the WebScrapper with the mocked URL
        let result = scraper.run(WebScrapperInput(url)).await;

        // Assert that the result is Ok and contains "Hello World"
        assert!(result.is_ok());
        let content = result.unwrap();
        assert_eq!(content.trim(), "Hello World");

        // Verify that the mock was called as expected
        mock.assert();
    }

    #[test]
    fn test_web_scrapper_input_deserialize() {
        let input = Value::String("https://example.com".to_string());

        let input = serde_json::from_value::<WebScrapperInput>(input).unwrap();

        assert_eq!(input.0, "https://example.com");
    }

    #[test]
    fn test_web_scrapper_input_schema() {
        let schema = WebScrapper::default().parameters();
        let schema = serde_json::to_value(schema).unwrap();

        let expected = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "WebScrapperInput",
            "type": "string",
            "description": "The URL to scrape, MUST be a working URL"
        });

        assert_eq!(schema, expected);
    }
}
