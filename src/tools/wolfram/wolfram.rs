use async_trait::async_trait;
use serde_json::Value;

use crate::tools::{Tool, ToolFunction, ToolWrapper};
use std::{error::Error, sync::Arc};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct WolframError {
    code: String,
    msg: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
enum WolframErrorStatus {
    Error(WolframError),
    NoError(bool),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct WolframResponse {
    queryresult: WolframResponseContent,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct WolframResponseContent {
    success: bool,
    error: WolframErrorStatus,
    pods: Option<Vec<Pod>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Pod {
    title: String,
    subpods: Vec<Subpod>,
}

impl From<Pod> for String {
    fn from(pod: Pod) -> String {
        let subpods_str: Vec<String> = pod
            .subpods
            .into_iter()
            .map(String::from)
            .filter(|s| !s.is_empty())
            .collect();

        if subpods_str.is_empty() {
            return String::from("");
        }

        format!(
            "{{\"title\": {},\"subpods\": [{}]}}",
            pod.title,
            subpods_str.join(",")
        )
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Subpod {
    title: String,
    plaintext: String,
}

impl From<Subpod> for String {
    fn from(subpod: Subpod) -> String {
        if subpod.plaintext.is_empty() {
            return String::from("");
        }

        format!(
            "{{\"title\": \"{}\",\"plaintext\": \"{}\"}}",
            subpod.title,
            subpod.plaintext.replace("\n", " // ")
        )
    }
}

/// When being used within agents GPT4 is recommended
pub struct Wolfram {
    app_id: String,
    exclude_pods: Vec<String>,
    client: reqwest::Client,
}

impl Wolfram {
    pub fn new(app_id: String) -> Self {
        Self {
            app_id,
            exclude_pods: Vec::new(),
            client: reqwest::Client::new(),
        }
    }

    pub fn with_excludes<S: AsRef<str>>(mut self, exclude_pods: &[S]) -> Self {
        self.exclude_pods = exclude_pods.iter().map(|s| s.as_ref().to_owned()).collect();
        self
    }

    pub fn with_app_id<S: AsRef<str>>(mut self, app_id: S) -> Self {
        self.app_id = app_id.as_ref().to_owned();
        self
    }
}

impl Default for Wolfram {
    fn default() -> Wolfram {
        Wolfram {
            app_id: std::env::var("WOLFRAM_APP_ID").unwrap_or_default(),
            exclude_pods: Vec::new(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl ToolFunction for Wolfram {
    type Input = String;
    type Result = String;

    fn name(&self) -> String {
        String::from("Wolfram")
    }

    fn description(&self) -> String {
        String::from(
            "Wolfram Solver leverages the Wolfram Alpha computational engine
            to solve complex queries. Input should be a valid mathematical 
            expression or query formulated in a way that Wolfram Alpha can 
            interpret.",
        )
    }

    async fn parse_input(&self, input: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        input
            .as_str()
            .map(|s| s.to_string())
            .ok_or("Invalid input".into())
    }

    async fn run(&self, input: String) -> Result<String, Box<dyn Error + Send + Sync>> {
        let mut url = format!(
            "https://api.wolframalpha.com/v2/query?appid={}&input={}&output=JSON&format=plaintext&podstate=Result__Step-by-step+solution",
            &self.app_id,
            urlencoding::encode(&input)
        );

        if !self.exclude_pods.is_empty() {
            url += &format!("&excludepodid={}", self.exclude_pods.join(","));
        }

        let response: WolframResponse = self.client.get(&url).send().await?.json().await?;

        if let WolframErrorStatus::Error(error) = response.queryresult.error {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Wolfram Error {}: {}", error.code, error.msg),
            )));
        } else if !response.queryresult.success {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Wolfram Error invalid query input: The query requested can not be processed by Wolfram".to_string(),
            )));
        }

        let pods_str: Vec<String> = response
            .queryresult
            .pods
            .unwrap_or_default()
            .into_iter()
            .map(String::from)
            .filter(|s| !s.is_empty())
            .collect();

        Ok(format!("{{\"pods\": [{}]}}", pods_str.join(",")))
    }
}

impl From<Wolfram> for Arc<dyn Tool> {
    fn from(wolfram: Wolfram) -> Arc<dyn Tool> {
        Arc::new(ToolWrapper::new(wolfram))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_wolfram() {
        let wolfram: Arc<dyn Tool> = Wolfram::default().with_excludes(&["Plot"]).into();
        let input = "Solve x^2 - 2x + 1 = 0";
        let result = wolfram.call(Value::String(input.to_string())).await;

        assert!(result.is_ok());
        println!("{}", result.unwrap());
    }
}
