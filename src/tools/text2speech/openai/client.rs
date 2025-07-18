use std::{error::Error, sync::Arc};

use async_openai::types::CreateSpeechRequestArgs;
use async_openai::Client;
pub use async_openai::{
    config::{Config, OpenAIConfig},
    types::{SpeechModel, SpeechResponseFormat, Voice},
};
use async_trait::async_trait;

use crate::tools::{input::DefaultToolInput, SpeechStorage, ToolFunction};

#[derive(Clone)]
pub struct Text2SpeechOpenAI<C: Config> {
    api_config: C,
    model: SpeechModel,
    voice: Voice,
    storage: Option<Arc<dyn SpeechStorage>>,
    response_format: SpeechResponseFormat,
    path: String,
}

impl<C: Config> Text2SpeechOpenAI<C> {
    pub fn new(api_config: C) -> Self {
        Self {
            api_config,
            model: SpeechModel::Tts1,
            voice: Voice::Alloy,
            storage: None,
            response_format: SpeechResponseFormat::Mp3,
            path: "./data/audio.mp3".to_string(),
        }
    }

    pub fn with_model(mut self, model: SpeechModel) -> Self {
        self.model = model;
        self
    }

    pub fn with_voice(mut self, voice: Voice) -> Self {
        self.voice = voice;
        self
    }

    pub fn with_storage<SS: SpeechStorage + 'static>(mut self, storage: SS) -> Self {
        self.storage = Some(Arc::new(storage));
        self
    }

    pub fn with_response_format(mut self, response_format: SpeechResponseFormat) -> Self {
        self.response_format = response_format;
        self
    }

    pub fn with_path<S: Into<String>>(mut self, path: S) -> Self {
        self.path = path.into();
        self
    }

    pub fn with_api_config(mut self, api_config: C) -> Self {
        self.api_config = api_config;
        self
    }
}

impl Default for Text2SpeechOpenAI<OpenAIConfig> {
    fn default() -> Self {
        Self::new(OpenAIConfig::default())
    }
}

#[async_trait]
impl<C: Config + Send + Sync> ToolFunction for Text2SpeechOpenAI<C> {
    type Input = DefaultToolInput;
    type Output = String;

    fn name(&self) -> String {
        "Text2SpeechOpenAI".into()
    }

    fn description(&self) -> String {
        r#"A wrapper around OpenAI Text2Speech. "
        "Useful for when you need to convert text to speech. "
        "It supports multiple languages, including English, German, Polish, "
        "Spanish, Italian, French, Portuguese""#
            .into()
    }

    async fn run(&self, input: Self::Input) -> Result<Self::Output, Box<dyn Error + Send + Sync>> {
        let client = Client::new();
        let response_format: SpeechResponseFormat = self.response_format;

        let request = CreateSpeechRequestArgs::default()
            .input(input.0)
            .voice(self.voice.clone())
            .response_format(response_format)
            .model(self.model.clone())
            .build()?;

        let response = client.audio().speech(request).await?;

        if self.storage.is_some() {
            let storage = self.storage.as_ref().unwrap(); //safe to unwrap
            let data = response.bytes;
            return storage.save(&self.path, &data).await;
        } else {
            response.save(&self.path).await?;
        }

        Ok(self.path.clone())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::tools::{Text2SpeechOpenAI, Tool};

    #[tokio::test]
    #[ignore]
    async fn openai_speech2text_tool() {
        let openai = Text2SpeechOpenAI::default();
        let s = openai
            .call(Value::String("Hola como estas".to_string()))
            .await
            .unwrap();
        println!("{s}");
    }
}
