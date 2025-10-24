//! Module for models that support OpenAI chat completions api.

mod generic_chat;
pub use generic_chat::*;

mod openai_chat;
pub use openai_chat::*;

mod chat_history;
mod helper;
mod request;
