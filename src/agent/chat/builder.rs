use std::{collections::HashMap, sync::Arc};

use crate::{
    agent::AgentError,
    chain::llm_chain::LLMChainBuilder,
    language_models::llm::LLM,
    tools::{ListTools, Tool, Toolbox},
    utils::helper::normalize_tool_name,
};

use super::{
    prompt::{DEFAULT_INITIAL_PROMPT, DEFAULT_SYSTEM_PROMPT, SUFFIX},
    ConversationalAgent,
};

pub struct ConversationalAgentBuilder<'a, 'b, 'c> {
    tools: Option<Vec<Box<dyn Tool>>>,
    toolboxes: Option<Vec<Box<dyn Toolbox>>>,
    system_prompt: Option<&'a str>,
    initial_prompt: Option<&'b str>,
    custom_tool_prompt: Option<Box<dyn Fn(&HashMap<String, Box<dyn Tool>>) -> String + 'c>>,
}

impl<'a, 'b, 'c> ConversationalAgentBuilder<'a, 'b, 'c> {
    pub fn new() -> Self {
        Self {
            tools: None,
            toolboxes: None,
            system_prompt: None,
            initial_prompt: None,
            custom_tool_prompt: None,
        }
    }

    pub fn tools(mut self, tools: impl IntoIterator<Item = impl Into<Box<dyn Tool>>>) -> Self {
        self.tools = Some(tools.into_iter().map(Into::into).collect());
        self
    }

    pub fn toolboxes(mut self, toolboxes: Vec<Box<dyn Toolbox>>) -> Self {
        self.toolboxes = Some(toolboxes);
        self
    }

    pub fn system_prompt(mut self, system_prompt: &'a str) -> Self {
        self.system_prompt = Some(system_prompt);
        self
    }

    pub fn initial_prompt(mut self, initial_prompt: &'b str) -> Self {
        self.initial_prompt = Some(initial_prompt);
        self
    }

    pub fn custom_tool_prompt(
        mut self,
        custom_tool_prompt: impl Fn(&HashMap<String, Box<dyn Tool>>) -> String + 'c,
    ) -> Self {
        self.custom_tool_prompt = Some(Box::new(custom_tool_prompt));
        self
    }

    pub async fn build<L: Into<Box<dyn LLM>>>(
        self,
        llm: L,
    ) -> Result<ConversationalAgent, AgentError> {
        let toolboxes = self
            .toolboxes
            .unwrap_or_default()
            .into_iter()
            .map(|tool| Arc::from(tool))
            .collect::<Vec<_>>();

        let tools = {
            let toolbox_list_tools = toolboxes
                .iter()
                .map(|toolbox| Box::new(ListTools::new(toolbox)) as Box<dyn Tool>);
            self.tools
                .unwrap_or_default()
                .into_iter()
                .chain(toolbox_list_tools)
                .map(|tool| (normalize_tool_name(&tool.name()), tool))
                .collect::<HashMap<_, _>>()
        };

        let system_prompt = self.system_prompt.unwrap_or(DEFAULT_SYSTEM_PROMPT);
        let initial_prompt = self.initial_prompt.unwrap_or(DEFAULT_INITIAL_PROMPT);
        let suffix = match self.custom_tool_prompt {
            Some(custom_tool_prompt) => custom_tool_prompt(&tools),
            None => {
                let tool_names = tools.keys().cloned().collect::<Vec<_>>().join(", ");
                let tool_string = tools
                    .values()
                    .map(|tool| tool.to_plain_description())
                    .collect::<Vec<_>>()
                    .join("\n");
                SUFFIX
                    .replace("{{tool_names}}", &tool_names)
                    .replace("{{tools}}", &tool_string)
            }
        };

        let prompt = ConversationalAgent::create_prompt(
            &format!("{}{}", system_prompt, suffix),
            initial_prompt,
        );
        let chain = Box::new(LLMChainBuilder::new().prompt(prompt).llm(llm).build()?);

        Ok(ConversationalAgent {
            chain,
            tools,
            toolboxes,
        })
    }
}

impl Default for ConversationalAgentBuilder<'_, '_, '_> {
    fn default() -> Self {
        Self::new()
    }
}
