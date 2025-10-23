use std::fmt;

use async_openai::types::responses::{
    ContentType, InputContent, InputImage, InputItem, InputMessageArgs, InputMessageType,
    Role as AsyncOpenaiRole,
};
use async_openai::types::{
    ChatCompletionRequestAssistantMessage, ChatCompletionRequestAssistantMessageArgs,
    ChatCompletionRequestMessage, ChatCompletionRequestMessageContentPartImage,
    ChatCompletionRequestSystemMessage, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestToolMessage, ChatCompletionRequestToolMessageArgs,
    ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageArgs,
    ChatCompletionRequestUserMessageContentPart,
};
use serde_json::json;

use super::{Role, ToolCall};
use crate::schemas::ImageContent;
use crate::utils::helper::capitalize_first;

/// A single message of an LLM interaction.
///
/// Corresponds to
/// [`ChatCompletionRequestMessage`](async_openai::types::ChatCompletionRequestMessage) for the chat
/// completions api and [`InputItem`](async_openai::types::responses::InputItem) for the responses
/// api.
///
/// # Fields
/// - `id`: The unique identifier for the message. May be missing depending on the LLM provider.
/// - `role`: The message role. e.g. system, ai, human, tool
/// - `content`: The content of the message.
/// - `tool_calls`: Tool calls associated with the message.
/// - `images`: Images associated with the message.
///
/// # Usage
/// ```
/// use langchain_rust::schemas::Message;
///
/// let human_message = Message::new_human_message("Hello");
/// let system_message = Message::new_system_message("System Alert");
/// let ai_message = Message::new_ai_message("AI Response");
/// ```
#[derive(Debug, Clone)]
pub struct Message {
    /// The unique identifier for the message. May be missing depending on the LLM provider.
    pub id: Option<String>,
    /// The message role. e.g. system, ai, human, tool
    pub role: Role,
    /// The content of the message.
    pub content: String,
    /// Tool calls associated with the message.
    pub tool_calls: Option<Vec<ToolCall>>,
    /// Images associated with the message.
    pub images: Option<Vec<ImageContent>>,
}

impl Message {
    /// Construct a new [`Message`].
    pub fn new(role: Role, content: impl Into<String>) -> Self {
        Message {
            content: content.into(),
            role,
            id: None,
            tool_calls: None,
            images: None,
        }
    }

    /// Constructs a new system message.
    pub fn new_system_message(content: impl Into<String>) -> Self {
        Message {
            content: content.into(),
            role: Role::System,
            id: None,
            tool_calls: None,
            images: None,
        }
    }

    /// Constructs a new user message.
    pub fn new_human_message(content: impl Into<String>) -> Self {
        Message {
            content: content.into(),
            role: Role::Human,
            id: None,
            tool_calls: None,
            images: None,
        }
    }

    /// Constructs a new AI message.
    pub fn new_ai_message(content: impl Into<String>) -> Self {
        Message {
            content: content.into(),
            role: Role::Ai,
            id: None,
            tool_calls: None,
            images: None,
        }
    }

    /// Constructs a new tool call message.
    pub fn new_tool_call_message(
        thought: Option<String>,
        tool_calls: impl IntoIterator<Item = ToolCall>,
    ) -> Self {
        Message {
            content: thought.unwrap_or_default(),
            role: Role::Ai,
            id: None,
            tool_calls: Some(tool_calls.into_iter().collect()),
            images: None,
        }
    }

    // Constructs a new tool output message.
    pub fn new_tool_message(id: Option<String>, content: impl Into<String>) -> Self {
        Message {
            content: content.into(),
            role: Role::Tool,
            id,
            tool_calls: None,
            images: None,
        }
    }

    /// Sets a list of images for the message.
    pub fn with_images<T: Into<ImageContent>>(mut self, images: Vec<T>) -> Self {
        self.images = Some(images.into_iter().map(|i| i.into()).collect());
        self
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let role = capitalize_first(&self.role.to_string());
        write!(f, "{role} message:")?;

        let mut is_empty = true;

        if !self.content.is_empty() {
            is_empty = false;
            write!(f, "\n{}", self.content)?;
        }

        if let Some(calls) = &self.tool_calls
            && !calls.is_empty()
        {
            is_empty = false;
            writeln!(f, "tool calls:",)?;
            for call in calls {
                write!(f, "\n{call}")?;
            }
        }

        if let Some(images) = &self.images {
            is_empty = false;
            write!(f, "\nimages: {images:?}")?
        }

        if is_empty {
            write!(f, "\n(Empty message)")?;
        }

        Ok(())
    }
}

impl From<Message> for ChatCompletionRequestMessage {
    fn from(value: Message) -> Self {
        fn system(content: String) -> ChatCompletionRequestSystemMessage {
            ChatCompletionRequestSystemMessageArgs::default()
                .content(content)
                .build()
                .expect("All required fields are set.")
        }
        fn user(content: String) -> ChatCompletionRequestUserMessage {
            ChatCompletionRequestUserMessageArgs::default()
                .content(content)
                .build()
                .expect("All required fields are set.")
        }
        fn user_images(images: Vec<ImageContent>) -> ChatCompletionRequestUserMessage {
            let parts = images
                .into_iter()
                .map(ChatCompletionRequestMessageContentPartImage::from)
                .map(ChatCompletionRequestUserMessageContentPart::ImageUrl)
                .collect::<Vec<_>>();
            ChatCompletionRequestUserMessageArgs::default()
                .content(parts)
                .build()
                .expect("All required fields are set.")
        }
        fn assistant(content: String) -> ChatCompletionRequestAssistantMessage {
            ChatCompletionRequestAssistantMessageArgs::default()
                .content(content)
                .build()
                .expect("All required fields are set.")
        }
        fn tool_calls(
            content: String,
            tool_calls: Vec<ToolCall>,
        ) -> ChatCompletionRequestAssistantMessage {
            let calls = tool_calls.into_iter().map(Into::into).collect::<Vec<_>>();
            ChatCompletionRequestAssistantMessageArgs::default()
                .content(content)
                .tool_calls(calls)
                .build()
                .expect("All required fields are set.")
        }
        fn tool(tool_call_id: String, content: String) -> ChatCompletionRequestToolMessage {
            ChatCompletionRequestToolMessageArgs::default()
                .content(content)
                .tool_call_id(tool_call_id)
                .build()
                .expect("All required fields are set.")
        }

        match value.role {
            Role::Ai => match value.tool_calls {
                Some(calls) => tool_calls(value.content, calls).into(),
                None => assistant(value.content).into(),
            },
            Role::Human => match value.images {
                Some(images) => user_images(images).into(),
                None => user(value.content).into(),
            },
            Role::System => system(value.content).into(),
            Role::Tool => tool(value.id.unwrap_or_default(), value.content).into(),
        }
    }
}

impl From<Message> for Vec<InputItem> {
    fn from(value: Message) -> Self {
        fn system(content: String) -> InputItem {
            let msg = InputMessageArgs::default()
                .kind(InputMessageType::Message)
                .role(AsyncOpenaiRole::System)
                .content(content)
                .build()
                .expect("All required fields are set.");
            InputItem::Message(msg)
        }
        fn user(content: String) -> InputItem {
            let msg = InputMessageArgs::default()
                .kind(InputMessageType::Message)
                .role(AsyncOpenaiRole::User)
                .content(content)
                .build()
                .expect("All required fields are set.");
            InputItem::Message(msg)
        }
        fn user_images(images: Vec<ImageContent>) -> InputItem {
            let parts = images
                .into_iter()
                .map(InputImage::from)
                .map(ContentType::InputImage)
                .collect::<Vec<_>>();
            let msg = InputMessageArgs::default()
                .kind(InputMessageType::Message)
                .role(AsyncOpenaiRole::User)
                .content(InputContent::InputItemContentList(parts))
                .build()
                .expect("All required fields are set.");
            InputItem::Message(msg)
        }
        fn assistant(content: String) -> InputItem {
            let msg = InputMessageArgs::default()
                .kind(InputMessageType::Message)
                .role(AsyncOpenaiRole::Assistant)
                .content(content)
                .build()
                .expect("All required fields are set.");
            InputItem::Message(msg)
        }
        fn tool_calls(tool_calls: Vec<ToolCall>) -> Vec<InputItem> {
            // The "function_call" / "function_call_output" is not yet supported in
            // async-openai. So we manually construct the message here.
            tool_calls
                .into_iter()
                .map(|call| {
                    InputItem::Custom(json!({
                        "type": "function_call",
                        "name": call.name,
                        "arguments": call.arguments,
                    }))
                })
                .collect::<Vec<_>>()
        }
        fn tool(tool_call_id: String, content: String) -> InputItem {
            // The "function_call" / "function_call_output" is not yet supported in
            // async-openai. So we manually construct the message here.
            let json = json!({
                "type": "function_call_output",
                "call_id": tool_call_id,
                "output": content
            });
            InputItem::Custom(json)
        }

        match value.role {
            Role::Ai => match value.tool_calls {
                Some(calls) => tool_calls(calls),
                None => vec![assistant(value.content)],
            },
            Role::Human => match value.images {
                Some(images) => vec![user_images(images)],
                None => vec![user(value.content)],
            },
            Role::System => vec![system(value.content)],
            Role::Tool => vec![tool(value.id.unwrap_or_default(), value.content)],
        }
    }
}
