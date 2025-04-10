use futures::future::try_join_all;

use crate::{
    agent::AgentError,
    chain::LLMChainBuilder,
    language_models::{llm::LLM, options::CallOptions, LLMError},
    tools::{Tool, Toolbox},
    utils::helper::normalize_tool_name,
};

use super::{
    prompt::{DEFAULT_INITIAL_PROMPT, DEFAULT_SYSTEM_PROMPT},
    OpenAiToolAgent,
};

pub struct OpenAiToolAgentBuilder<'a, 'b> {
    tools: Option<Vec<Box<dyn Tool>>>,
    toolboxes: Option<Vec<Box<dyn Toolbox>>>,
    system_prompt: Option<&'a str>,
    initial_prompt: Option<&'b str>,
}

impl<'a, 'b> OpenAiToolAgentBuilder<'a, 'b> {
    pub fn new() -> Self {
        Self {
            tools: None,
            toolboxes: None,
            system_prompt: None,
            initial_prompt: None,
        }
    }

    pub fn tools(mut self, tools: Vec<Box<dyn Tool>>) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn toolboxes(mut self, toolboxes: Vec<Box<dyn Toolbox>>) -> Self {
        self.toolboxes = Some(toolboxes);
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

    pub async fn build<L: LLM + 'static>(self, llm: L) -> Result<OpenAiToolAgent, AgentError> {
        let system_prompt = self.system_prompt.unwrap_or(DEFAULT_SYSTEM_PROMPT);
        let initial_prompt = self.initial_prompt.unwrap_or(DEFAULT_INITIAL_PROMPT);

        let toolboxes = self.toolboxes.unwrap_or_default();
        let tools = self.tools.unwrap_or_default();

        let tools_openai = {
            let local_tools = tools
                .iter()
                .map(|tool| tool.into_openai_tool())
                .collect::<Vec<_>>();

            let get_toolbox_tools_futures = toolboxes
                .iter()
                .map(|toolbox| toolbox.get_tools())
                .collect::<Vec<_>>();
            let toolbox_tools = try_join_all(get_toolbox_tools_futures)
                .await
                .or_else(|e| {
                    Err(LLMError::OtherError(format!(
                        "Failed to fetch tool metadata from toolbox: {e}"
                    )))
                })?
                .into_iter()
                .flat_map(|tools| tools.values().map(|tool| tool.into_openai_tool()))
                .collect::<Vec<_>>();

            local_tools
                .into_iter()
                .chain(toolbox_tools.into_iter())
                .collect::<Vec<_>>()
        };

        let prompt = OpenAiToolAgent::create_prompt(system_prompt, initial_prompt)?;
        let mut llm = llm;
        llm.add_call_options(CallOptions::new().with_tools(tools_openai));
        let chain = Box::new(LLMChainBuilder::new().prompt(prompt).llm(llm).build()?);

        let tools_map = tools
            .into_iter()
            .map(|tool| (normalize_tool_name(&tool.name()), tool))
            .collect();

        Ok(OpenAiToolAgent {
            chain,
            tools: tools_map,
            toolboxes,
        })
    }
}

impl<'a, 'b> Default for OpenAiToolAgentBuilder<'a, 'b> {
    fn default() -> Self {
        Self::new()
    }
}
