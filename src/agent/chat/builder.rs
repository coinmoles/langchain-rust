use std::{collections::HashMap, fmt::Display, sync::Arc};

use crate::{
    agent::{
        create_prompt,
        instructor::{BoxInstructorExt, DefaultInstructor, Instructor},
    },
    chain::LLMChain,
    language_models::llm::LLM,
    schemas::{
        BuilderError, ChainOutput, DefaultChainInputCtor, InputCtor, OutputCtor, StringCtor,
    },
    tools::{ListTools, Tool, Toolbox},
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
> where
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    tools: Option<Vec<Box<dyn Tool>>>,
    toolboxes: Option<Vec<Box<dyn Toolbox>>>,
    system_prompt: Option<&'a str>,
    initial_prompt: Option<&'b str>,
    instructor: Option<Box<dyn Instructor>>,
    _phantom: std::marker::PhantomData<(I, O)>,
}

impl<'a, 'b, I: InputCtor, O: OutputCtor> ConversationalAgentBuilder<'a, 'b, I, O>
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
            instructor: None,
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

    pub fn instructor(mut self, instructor: impl Instructor + 'static) -> Self {
        self.instructor = Some(Box::new(instructor));
        self
    }

    pub async fn build<L: Into<Box<dyn LLM>>>(
        self,
        llm: L,
    ) -> Result<ConversationalAgent<I, O>, BuilderError> {
        let toolboxes = self
            .toolboxes
            .unwrap_or_default()
            .into_iter()
            .map(Arc::from)
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
            .map_err(|e| BuilderError::Inner("llm_chain", Box::new(e)))?;

        Ok(ConversationalAgent {
            llm_chain,
            tools,
            toolboxes,
            _phantom: std::marker::PhantomData,
        })
    }
}
