use crate::{
    prompt_template,
    schemas::MessageType,
    template::{MessageOrTemplate, MessageTemplate, PromptTemplate},
};

pub fn create_prompt(
    system_prompt: impl Into<String>,
    initial_prompt: impl Into<String>,
) -> PromptTemplate {
    prompt_template![
        MessageTemplate::from_jinja2(MessageType::System, system_prompt),
        MessageOrTemplate::Placeholder("chat_history".into()),
        MessageTemplate::from_jinja2(MessageType::Human, initial_prompt),
        MessageOrTemplate::Placeholder("agent_scratchpad".into()),
        MessageOrTemplate::Placeholder("ultimatum".into())
    ]
}
