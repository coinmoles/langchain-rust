use async_openai::types::{ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage};
use serde::Serialize;

/// A helper struct to serialize chat history without cloning and while preserving easy access to a
/// system message for later alteration.
pub struct ChatHistory<'b> {
    /// The system message.
    system: SystemMessage<'b>,
    /// Other messages.
    messages: &'b [ChatCompletionRequestMessage],
}

impl<'b> ChatHistory<'b> {
    /// Constructs a new [`ChatHistory`].
    pub fn new(
        system: &'b ChatCompletionRequestSystemMessage,
        messages: &'b [ChatCompletionRequestMessage],
    ) -> Self {
        ChatHistory {
            system: SystemMessage::new(system),
            messages,
        }
    }

    /// Constructs a new [`ChatHistory`] with system message serialized as an assistant message.
    pub fn as_assistant(
        system: &'b ChatCompletionRequestSystemMessage,
        messages: &'b [ChatCompletionRequestMessage],
    ) -> Self {
        ChatHistory {
            system: SystemMessage::as_assistant(system),
            messages,
        }
    }
}

impl Serialize for ChatHistory<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;

        let mut seq = serializer.serialize_seq(Some(self.messages.len() + 1))?;
        seq.serialize_element(&self.system)?;
        for message in self.messages {
            seq.serialize_element(message)?;
        }
        seq.end()
    }
}

/// A helper struct to serialize system message with a "system" role.
#[derive(Serialize)]
pub struct SystemMessage<'b> {
    /// The role of the message. Should be "system", or "assistant" for models that do not support
    /// system message.
    role: &'static str,
    /// The message itself.
    #[serde(flatten)]
    message: &'b ChatCompletionRequestSystemMessage,
}

impl<'b> SystemMessage<'b> {
    /// Constructs a new system message.
    pub fn new(message: &'b ChatCompletionRequestSystemMessage) -> Self {
        SystemMessage {
            role: "system",
            message,
        }
    }

    /// Constructs a new system message that is serialized as an assistant message.
    pub fn as_assistant(message: &'b ChatCompletionRequestSystemMessage) -> Self {
        SystemMessage {
            role: "assistant",
            message,
        }
    }
}

#[cfg(test)]
mod tests {
    use async_openai::types::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestUserMessageArgs,
    };
    use serde_json::json;

    use super::*;

    #[test]
    fn test_serialize_system_message() {
        let system_message = ChatCompletionRequestSystemMessage {
            content: "You are a helpful assistant.".into(),
            name: None,
        };

        let system = SystemMessage::new(&system_message);
        let serialized = serde_json::to_value(&system).unwrap();
        let expected = json!({
            "role": "system",
            "content": "You are a helpful assistant."
        });

        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_serialize_chat_history() {
        let system_message = ChatCompletionRequestSystemMessage {
            content: "You are a helpful assistant.".into(),
            name: None,
        };
        let user_message = ChatCompletionRequestUserMessageArgs::default()
            .content("Hello!")
            .build()
            .unwrap();
        let assistant_message = ChatCompletionRequestAssistantMessageArgs::default()
            .content("Hi there! How can I assist you today?")
            .build()
            .unwrap();

        let messages = vec![
            ChatCompletionRequestMessage::User(user_message),
            ChatCompletionRequestMessage::Assistant(assistant_message),
        ];

        let chat_history = ChatHistory::new(&system_message, &messages);
        let serialized = serde_json::to_value(&chat_history).unwrap();
        let expected = json!([
            {
                "role": "system",
                "content": "You are a helpful assistant."
            },
            {
                "role": "user",
                "content": "Hello!"
            },
            {
                "role": "assistant",
                "content": "Hi there! How can I assist you today?"
            }
        ]);

        assert_eq!(serialized, expected);
    }
}
