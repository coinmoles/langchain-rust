use std::fmt::Display;

use crate::{
    agent::create_prompt,
    chain::{ChainOutput, InputCtor, LLMChain, OutputCtor},
    llm::{options::CallOptions, LLM},
    schemas::BuilderError,
    tools::{Tool, Toolbox},
    utils::helper::normalize_tool_name,
};

use super::{
    prompt::{DEFAULT_INITIAL_PROMPT, DEFAULT_SYSTEM_PROMPT},
    OpenAiToolAgent,
};

pub struct OpenAiToolAgentBuilder<'a, 'b, I: InputCtor, O: OutputCtor>
where
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    tools: Option<Vec<Box<dyn Tool>>>,
    toolboxes: Option<Vec<Box<dyn Toolbox>>>,
    system_prompt: Option<&'a str>,
    initial_prompt: Option<&'b str>,
    _phantom: std::marker::PhantomData<(I, O)>,
}

impl<'a, 'b, I: InputCtor, O: OutputCtor> OpenAiToolAgentBuilder<'a, 'b, I, O>
where
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    pub(super) fn new() -> Self {
        Self {
            tools: None,
            toolboxes: None,
            system_prompt: None,
            initial_prompt: None,
            _phantom: std::marker::PhantomData,
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

    pub async fn build<L: LLM + 'static>(
        self,
        llm: L,
    ) -> Result<OpenAiToolAgent<I, O>, BuilderError> {
        let system_prompt = self.system_prompt.unwrap_or(DEFAULT_SYSTEM_PROMPT);
        let initial_prompt = self.initial_prompt.unwrap_or(DEFAULT_INITIAL_PROMPT);

        let toolboxes = self.toolboxes.unwrap_or_default();
        let tools = self.tools.unwrap_or_default();

        let tools_openai = {
            let local_tools = tools
                .iter()
                .map(|tool| tool.as_openai_tool())
                .collect::<Vec<_>>();

            let toolbox_tools = toolboxes
                .iter()
                .map(|toolbox| toolbox.get_tools())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| {
                    BuilderError::Other(format!("Failed to fetch tool metadata from toolbox: {e}"))
                })?
                .iter()
                .flat_map(|tools| tools.values().map(|tool| tool.as_openai_tool()))
                .collect::<Vec<_>>();

            local_tools
                .into_iter()
                .chain(toolbox_tools.into_iter())
                .collect::<Vec<_>>()
        };

        let prompt = create_prompt(system_prompt, initial_prompt);
        let mut llm = llm;
        llm.add_call_options(CallOptions::new().with_tools(tools_openai));
        let llm_chain = LLMChain::builder()
            .prompt(prompt)
            .llm(llm)
            .build()
            .map_err(|e| BuilderError::Inner("llm_chain", Box::new(e)))?;

        let tools_map = tools
            .into_iter()
            .map(|tool| (normalize_tool_name(&tool.name()), tool))
            .collect();

        Ok(OpenAiToolAgent {
            llm_chain,
            tools: tools_map,
            toolboxes,
            _phantom: std::marker::PhantomData,
        })
    }
}
