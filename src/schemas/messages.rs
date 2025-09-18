use std::fmt;

use async_openai::error::OpenAIError;
use async_openai::types::responses::{
    ContentType, InputContent, InputImageArgs, InputItem, InputMessageArgs, InputMessageType, Role,
};
use async_openai::types::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
    ChatCompletionRequestMessageContentPartImageArgs, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestToolMessageArgs, ChatCompletionRequestUserMessageArgs,
    ChatCompletionRequestUserMessageContent,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{MessageType, ToolCall};

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
            writeln!(f, "Tool call:",)?;
            for (i, tool_call) in tool_calls.iter().enumerate() {
                if i > 0 {
                    writeln!(f)?;
                }
                write!(f, "{tool_call}")?;
            }
            Ok(())
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

impl TryFrom<Message> for ChatCompletionRequestMessage {
    type Error = OpenAIError;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        fn tool_calls(
            tool_calls: Vec<ToolCall>,
        ) -> Result<ChatCompletionRequestMessage, OpenAIError> {
            let calls = tool_calls
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()
                .map_err(OpenAIError::JSONDeserialize)?;
            let msg = ChatCompletionRequestAssistantMessageArgs::default()
                .tool_calls(calls)
                .build()?
                .into();
            Ok(msg)
        }

        fn assistant(content: String) -> Result<ChatCompletionRequestMessage, OpenAIError> {
            let msg = ChatCompletionRequestAssistantMessageArgs::default()
                .content(content)
                .build()?
                .into();
            Ok(msg)
        }

        fn user(
            text: String,
            images: Option<Vec<ImageContent>>,
        ) -> Result<ChatCompletionRequestMessage, OpenAIError> {
            let content: ChatCompletionRequestUserMessageContent = match images {
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
                None => text.into(),
            };
            let msg = ChatCompletionRequestUserMessageArgs::default()
                .content(content)
                .build()?
                .into();
            Ok(msg)
        }

        fn system(content: String) -> Result<ChatCompletionRequestMessage, OpenAIError> {
            let msg = ChatCompletionRequestSystemMessageArgs::default()
                .content(content)
                .build()?
                .into();
            Ok(msg)
        }

        fn tool(
            tool_call_id: String,
            content: String,
        ) -> Result<ChatCompletionRequestMessage, OpenAIError> {
            let msg = ChatCompletionRequestToolMessageArgs::default()
                .content(content)
                .tool_call_id(tool_call_id)
                .build()?
                .into();
            Ok(msg)
        }

        match value.message_type {
            MessageType::Ai => match value.tool_calls {
                Some(calls) => tool_calls(calls),
                None => assistant(value.content),
            },
            MessageType::Human => user(value.content, value.images),
            MessageType::System => system(value.content),
            MessageType::Tool => tool(value.id.unwrap_or_default(), value.content),
        }
    }
}

impl TryFrom<Message> for Vec<InputItem> {
    type Error = OpenAIError;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        fn tool_calls(tool_calls: Vec<ToolCall>) -> Result<Vec<InputItem>, OpenAIError> {
            // The "function_call" / "function_call_output" is not yet supported in
            // async-openai. So we manually construct the message here.
            let calls = tool_calls
                .into_iter()
                .map(|call| {
                    InputItem::Custom(json!({
                        "type": "function_call",
                        "name": call.name,
                        "arguments": call.arguments,
                    }))
                })
                .collect::<Vec<_>>();
            Ok(calls)
        }

        fn assistant(content: String) -> Result<InputItem, OpenAIError> {
            let msg = InputMessageArgs::default()
                .kind(InputMessageType::Message)
                .role(Role::Assistant)
                .content(content)
                .build()?;
            Ok(InputItem::Message(msg))
        }

        fn user(text: String, images: Option<Vec<ImageContent>>) -> Result<InputItem, OpenAIError> {
            let content = match images {
                Some(images) => {
                    let images = images
                        .into_iter()
                        .map(|image| {
                            // TODO: reimplement detail
                            InputImageArgs::default()
                                .image_url(image.image_url)
                                .build()
                                .map(ContentType::InputImage)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    InputContent::InputItemContentList(images)
                }
                None => InputContent::TextInput(text),
            };
            let msg = InputMessageArgs::default()
                .kind(InputMessageType::Message)
                .role(Role::User)
                .content(content)
                .build()?;
            Ok(InputItem::Message(msg))
        }

        fn system(content: String) -> Result<InputItem, OpenAIError> {
            let msg = InputMessageArgs::default()
                .kind(InputMessageType::Message)
                .role(Role::System)
                .content(content)
                .build()?;
            Ok(InputItem::Message(msg))
        }

        fn tool(tool_call_id: String, content: String) -> Result<InputItem, OpenAIError> {
            // The "function_call" / "function_call_output" is not yet supported in
            // async-openai. So we manually construct the message here.
            let json = json!({
                "type": "function_call_output",
                "call_id": tool_call_id,
                "output": content
            });
            Ok(InputItem::Custom(json))
        }

        let msgs = match value.message_type {
            MessageType::Ai => match value.tool_calls {
                Some(calls) => tool_calls(calls)?,
                None => vec![assistant(value.content)?],
            },
            MessageType::Human => vec![user(value.content, value.images)?],
            MessageType::System => vec![system(value.content)?],
            MessageType::Tool => vec![tool(value.id.unwrap_or_default(), value.content)?],
        };
        Ok(msgs)
    }
}
