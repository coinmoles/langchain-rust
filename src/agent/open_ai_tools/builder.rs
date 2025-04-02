use std::{collections::HashMap, sync::Arc};

use async_openai::types::ChatCompletionTool;

use crate::{
    agent::AgentError,
    chain::LLMChainBuilder,
    language_models::{llm::LLM, options::CallOptions, LLMError},
    tools::Tool,
};

use super::{
    prompt::{DEFAULT_INITIAL_PROMPT, DEFAULT_SYSTEM_PROMPT},
    OpenAiToolAgent,
};

pub struct OpenAiToolAgentBuilder<'a, 'b> {
    tools: Option<HashMap<String, Arc<dyn Tool>>>,
    system_prompt: Option<&'a str>,
    initial_prompt: Option<&'b str>,
}

impl<'a, 'b> OpenAiToolAgentBuilder<'a, 'b> {
    pub fn new() -> Self {
        Self {
            tools: None,
            system_prompt: None,
            initial_prompt: None,
        }
    }

    pub fn tools(mut self, tools: HashMap<String, Arc<dyn Tool>>) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn system_prompt<S: Into<String>>(mut self, system_prompt: &'a str) -> Self {
        self.system_prompt = Some(system_prompt);
        self
    }

    pub fn initial_prompt<S: Into<String>>(mut self, initial_prompt: &'b str) -> Self {
        self.initial_prompt = Some(initial_prompt);
        self
    }

    pub fn build<L: LLM + 'static>(self, llm: L) -> Result<OpenAiToolAgent, AgentError> {
        let tools = self.tools.unwrap_or_default();
        let system_prompt = self.system_prompt.unwrap_or(DEFAULT_SYSTEM_PROMPT);
        let initial_prompt = self.initial_prompt.unwrap_or(DEFAULT_INITIAL_PROMPT);

        let mut llm = llm;

        let prompt = OpenAiToolAgent::create_prompt(system_prompt, initial_prompt)?;

        let tools_openai = tools
            .values()
            .map(|tool| tool.try_into_opeai_tool())
            .collect::<Result<Vec<ChatCompletionTool>, _>>()
            .map_err(LLMError::from)?;
        llm.add_options(CallOptions::new().with_tools(tools_openai));
        let chain = Box::new(LLMChainBuilder::new().prompt(prompt).llm(llm).build()?);

        Ok(OpenAiToolAgent { chain, tools })
    }
}

impl<'a, 'b> Default for OpenAiToolAgentBuilder<'a, 'b> {
    fn default() -> Self {
        Self::new()
    }
}
