use std::fmt;

use async_openai::error::OpenAIError;
use async_openai::types::ChatCompletionRequestAssistantMessageArgs;
use async_openai::types::ChatCompletionRequestMessage;
use async_openai::types::ChatCompletionRequestMessageContentPartImageArgs;
use async_openai::types::ChatCompletionRequestSystemMessageArgs;
use async_openai::types::ChatCompletionRequestToolMessageArgs;
use async_openai::types::ChatCompletionRequestUserMessageArgs;
use async_openai::types::ChatCompletionRequestUserMessageContent;
use serde::Deserialize;
use serde::Serialize;

use super::MessageType;
use super::ToolCall;

/// Struct `ImageContent` represents an image provided to an LLM.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ImageContent {
    pub image_url: String,
    pub detail: Option<String>,
}

impl<S: AsRef<str>> From<S> for ImageContent {
    fn from(image_url: S) -> Self {
        ImageContent {
            image_url: image_url.as_ref().into(),
            detail: None,
        }
    }
}

/// Struct `Message` represents a message with its content and type.
///
/// # Usage
/// ```rust,ignore
/// let human_message = Message::new_human_message("Hello");
/// let system_message = Message::new_system_message("System Alert");
/// let ai_message = Message::new_ai_message("AI Response");
/// ```
#[derive(Debug, Default, Clone)]
pub struct Message {
    pub content: String,
    pub message_type: MessageType,
    pub id: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub images: Option<Vec<ImageContent>>,
}

impl Message {
    pub fn new<T: std::fmt::Display>(message_type: MessageType, content: T) -> Self {
        Message {
            content: content.to_string(),
            message_type,
            id: None,
            tool_calls: None,
            images: None,
        }
    }

    pub fn new_system_message<T: std::fmt::Display>(content: T) -> Self {
        Message {
            content: content.to_string(),
            message_type: MessageType::System,
            id: None,
            tool_calls: None,
            images: None,
        }
    }

    pub fn new_human_message<T: std::fmt::Display>(content: T) -> Self {
        Message {
            content: content.to_string(),
            message_type: MessageType::Human,
            id: None,
            tool_calls: None,
            images: None,
        }
    }

    pub fn new_ai_message<T: std::fmt::Display>(content: T) -> Self {
        Message {
            content: content.to_string(),
            message_type: MessageType::Ai,
            id: None,
            tool_calls: None,
            images: None,
        }
    }

    pub fn new_tool_call_message(tool_calls: impl IntoIterator<Item = ToolCall>) -> Self {
        Message::new_ai_message("").with_tool_calls(tool_calls)
    }

    // Function to create a new Tool message with a generic type that implements Display
    pub fn new_tool_message<T: std::fmt::Display, S: Into<String>>(
        id: Option<S>,
        content: T,
    ) -> Self {
        Message {
            content: content.to_string(),
            message_type: MessageType::Tool,
            id: id.map(|id| id.into()),
            tool_calls: None,
            images: None,
        }
    }

    /// Sets the tool calls for the OpenAI-like API call.
    ///
    /// Use this method when you need to specify tool calls in the configuration.
    /// This is particularly useful in scenarios where interactions with specific
    /// tools are required for operation.
    ///
    /// # Arguments
    ///
    /// * `tool_calls` - A `serde_json::Value` representing the tool call configurations.
    pub fn with_tool_calls(mut self, tool_calls: impl IntoIterator<Item = ToolCall>) -> Self {
        self.tool_calls = Some(tool_calls.into_iter().collect());
        self
    }

    pub fn with_images<T: Into<ImageContent>>(mut self, images: Vec<T>) -> Self {
        self.images = Some(images.into_iter().map(|i| i.into()).collect());
        self
    }

    pub fn messages_to_string(messages: &[Message]) -> String {
        messages
            .iter()
            .map(|m| m.to_string())
            .collect::<Vec<String>>()
            .join("\n")
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(tool_calls) = &self.tool_calls {
            write!(
                f,
                "Tool call:\n{}",
                serde_json::to_string_pretty(&tool_calls)
                    .unwrap_or("Tool call details unknown".into())
            )
        } else if let Some(images) = &self.images {
            write!(
                f,
                "{}: {}\nImages: {:?}",
                self.message_type, self.content, images
            )
        } else if !self.content.is_empty() {
            write!(f, "{}: {}", self.message_type, self.content)
        } else {
            log::warn!("Message without content nor tool calls found, possibly an error");
            Ok(())
        }
    }
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
    }
}

impl TryFrom<Message> for ChatCompletionRequestMessage {
    type Error = OpenAIError;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        match value.message_type {
            MessageType::Ai => Ok(match value.tool_calls {
                Some(tool_calls) => ChatCompletionRequestAssistantMessageArgs::default()
                    .tool_calls(
                        tool_calls
                            .into_iter()
                            .map(TryInto::try_into)
                            .collect::<Result<Vec<_>, _>>()
                            .map_err(OpenAIError::JSONDeserialize)?,
                    )
                    .content(value.content)
                    .build()?
                    .into(),
                None => ChatCompletionRequestAssistantMessageArgs::default()
                    .content(value.content)
                    .build()?
                    .into(),
            }),
            MessageType::Human => {
                let content: ChatCompletionRequestUserMessageContent = match value.images {
                    Some(images) => images
                        .into_iter()
                        .map(|image| {
                            ChatCompletionRequestMessageContentPartImageArgs::default()
                                .image_url(image.image_url)
                                .build()
                                .map(Into::into)
                        })
                        .collect::<Result<Vec<_>, _>>()?
                        .into(),
                    None => value.content.into(),
                };

                Ok(ChatCompletionRequestUserMessageArgs::default()
                    .content(content)
                    .build()?
                    .into())
            }
            MessageType::System => Ok(ChatCompletionRequestSystemMessageArgs::default()
                .content(value.content)
                .build()?
                .into()),
            MessageType::Tool => Ok(ChatCompletionRequestToolMessageArgs::default()
                .content(value.content)
                .tool_call_id(value.id.unwrap_or_default())
                .build()?
                .into()),
        }
    }
}
