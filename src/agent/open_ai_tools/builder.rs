use crate::{
    agent::create_prompt,
    chain::{InputCtor, LLMChain, OutputCtor},
    llm::{options::CallOptions, LLM},
    tools::{ToolDyn, Toolbox},
    utils::helper::normalize_tool_name,
};

use super::{
    prompt::{DEFAULT_INITIAL_PROMPT, DEFAULT_SYSTEM_PROMPT},
    OpenAiToolAgent,
};

/// A builder for constructing an [`OpenAiToolAgent`].
///
/// # Type Parameters
/// - `I`: A [constructor](crate::chain::Ctor) for the agent’s input type.
/// - `O`: A [constructor](crate::chain::Ctor) for the agent’s output type.
pub struct OpenAiToolAgentBuilder<'a, 'b, I: InputCtor, O: OutputCtor> {
    /// The tools to be used by the agent.
    tools: Option<Vec<Box<dyn ToolDyn>>>,
    /// The toolboxes containing additional tools for the agent.
    toolboxes: Option<Vec<Box<dyn Toolbox>>>,
    /// The system prompt to be used by the agent.
    system_prompt: Option<&'a str>,
    /// The initial user prompt to be used by the agent.
    initial_prompt: Option<&'b str>,
    _phantom: std::marker::PhantomData<(I, O)>,
}

impl<'a, 'b, I: InputCtor, O: OutputCtor> OpenAiToolAgentBuilder<'a, 'b, I, O> {
    /// Constructs a new [`OpenAiToolAgentBuilder`].
    ///
    /// This is the same as calling [`OpenAiToolAgent::builder()`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            tools: None,
            toolboxes: None,
            system_prompt: None,
            initial_prompt: None,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Adds tools.
    pub fn tools(mut self, tools: impl IntoIterator<Item = impl Into<Box<dyn ToolDyn>>>) -> Self {
        self.tools = Some(tools.into_iter().map(Into::into).collect());
        self
    }

    /// Adds toolboxes.
    pub fn toolboxes(mut self, toolboxes: Vec<Box<dyn Toolbox>>) -> Self {
        self.toolboxes = Some(toolboxes);
        self
    }

    /// Sets the system prompt.
    pub fn system_prompt(mut self, system_prompt: &'a str) -> Self {
        self.system_prompt = Some(system_prompt);
        self
    }

    /// Sets the initial prompt.
    pub fn initial_prompt(mut self, initial_prompt: &'b str) -> Self {
        self.initial_prompt = Some(initial_prompt);
        self
    }

    /// Returns a [`OpenAiToolAgent`] that uses this [`OpenAiToolAgentBuilder`] configuration.
    pub fn build<L: LLM + 'static>(self, llm: L) -> OpenAiToolAgent<I, O> {
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
                .map(|toolbox| toolbox.get_tools().into_iter())
                .flat_map(|tools| tools.map(|(_, tool)| tool.as_openai_tool()))
                .collect::<Vec<_>>();

            local_tools
                .into_iter()
                .chain(toolbox_tools)
                .collect::<Vec<_>>()
        };

        let prompt = create_prompt(system_prompt, initial_prompt);
        let mut llm = llm;
        llm.add_call_options(CallOptions::new().with_tools(tools_openai));
        let llm_chain = LLMChain::builder()
            .prompt(prompt)
            .llm(llm)
            .build()
            .unwrap_or_else(|_| unreachable!("All necessary fields are provided"));

        let tools_map = tools
            .into_iter()
            .map(|tool| (normalize_tool_name(&tool.name()), tool))
            .collect();

        OpenAiToolAgent {
            llm_chain,
            tools: tools_map,
            toolboxes,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'a, 'b, I: InputCtor, O: OutputCtor> Default for OpenAiToolAgentBuilder<'a, 'b, I, O> {
    fn default() -> Self {
        Self::new()
    }
}
