use std::{collections::HashMap, sync::Arc};

use crate::{
    agent::create_prompt,
    chain::{DefaultChainInputCtor, InputCtor, LLMChain, OutputCtor, StringCtor},
    instructor::{BoxInstructorExt, DefaultInstructor, Instructor},
    llm::LLM,
    tools::{ListTools, ToolDyn, Toolbox},
    utils::helper::normalize_tool_name,
};

use super::{
    prompt::{DEFAULT_INITIAL_PROMPT, DEFAULT_SYSTEM_PROMPT},
    ConversationalAgent,
};

pub struct ConversationalAgentBuilder<
    'a,
    'b,
    I: InputCtor = DefaultChainInputCtor,
    O: OutputCtor = StringCtor,
> {
    /// The tools to be used by the agent.
    tools: Option<Vec<Box<dyn ToolDyn>>>,
    /// The toolboxes containing additional tools for the agent.
    toolboxes: Option<Vec<Box<dyn Toolbox>>>,
    /// The system prompt to be used by the agent.
    system_prompt: Option<&'a str>,
    /// The initial user prompt to be used by the agent.
    initial_prompt: Option<&'b str>,
    /// The instructor to customize tool call format and parsing logic.
    instructor: Option<Box<dyn Instructor>>,
    _phantom: std::marker::PhantomData<(I, O)>,
}

/// A builder for constructing an [`OpenAiToolAgent`].
///
/// # Type Parameters
/// - `I`: A [constructor](crate::chain::Ctor) for the agent’s input type.
/// - `O`: A [constructor](crate::chain::Ctor) for the agent’s output type.
impl<'a, 'b, I: InputCtor, O: OutputCtor> ConversationalAgentBuilder<'a, 'b, I, O> {
    /// Constructs a new [`ConversationalAgentBuilder`].
    ///
    /// This is the same as calling [`ConversationalAgent::builder()`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            tools: None,
            toolboxes: None,
            system_prompt: None,
            initial_prompt: None,
            instructor: None,
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

    /// Sets the instructor.
    ///
    /// Instructor is used to customize the tool call format and parsing logic.
    pub fn instructor(mut self, instructor: impl Instructor + 'static) -> Self {
        self.instructor = Some(Box::new(instructor));
        self
    }

    /// Returns a [`ConversationalAgent`] that uses this [`ConversationalAgentBuilder`] configuration.
    pub fn build<L: Into<Box<dyn LLM>>>(self, llm: L) -> ConversationalAgent<I, O> {
        let toolboxes = self
            .toolboxes
            .unwrap_or_default()
            .into_iter()
            .map(Arc::from)
            .collect::<Vec<_>>();

        let tools = {
            let toolbox_list_tools = toolboxes
                .iter()
                .map(|toolbox| Box::new(ListTools::new(toolbox)) as Box<dyn ToolDyn>);
            self.tools
                .unwrap_or_default()
                .into_iter()
                .chain(toolbox_list_tools)
                .map(|tool| (normalize_tool_name(&tool.name()), tool))
                .collect::<HashMap<_, _>>()
        };

        let instructor = self
            .instructor
            .unwrap_or_else(|| Box::new(DefaultInstructor));

        let system_prompt = {
            let body = self.system_prompt.unwrap_or(DEFAULT_SYSTEM_PROMPT);
            let suffix = if tools.is_empty() {
                String::new()
            } else {
                instructor.create_suffix(&tools.values().map(|t| t.as_ref()).collect::<Vec<_>>())
            };
            format!("{body}{suffix}")
        };
        let initial_prompt = self.initial_prompt.unwrap_or(DEFAULT_INITIAL_PROMPT);

        let prompt = create_prompt(system_prompt, initial_prompt);
        let llm_chain = LLMChain::builder()
            .prompt(prompt)
            .llm(llm)
            .output_parser(instructor.into_parser())
            .build()
            .unwrap_or_else(|_| unreachable!("All necessary fields are provided"));

        ConversationalAgent {
            llm_chain,
            tools,
            toolboxes,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'a, 'b, I: InputCtor, O: OutputCtor> Default for ConversationalAgentBuilder<'a, 'b, I, O> {
    fn default() -> Self {
        Self::new()
    }
}
