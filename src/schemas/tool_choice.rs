use async_openai::types::responses::{
    HostedToolType, ToolChoice as ResponsesToolChoice, ToolChoiceMode,
};
use async_openai::types::{
    ChatCompletionNamedToolChoice, ChatCompletionToolChoiceOption, FunctionName,
};

/// A parameter to control which (if any) tool is called by the model.
///
/// Note that these options only apply to models that support function call natively.
///
/// Corresponds to [`ChatCompletionToolChoiceOption`] for the chat completions api and
/// [`ToolChoice`](ResponsesToolChoice) for the responses api.
#[derive(Clone, Debug)]
pub enum ToolChoice {
    /// The model will not call any tool and instead generates a message.
    None,
    /// The model can pick between generating a message or calling one or more tools.
    Auto,
    /// The model must call one or more tools.
    Required,
    /// The model must call a specific function.
    Function(String),
    /// The model must call a hosted function. Not compatible with the chat completions api.
    Hosted(&'static str),
}

impl ToolChoice {
    /// The model must use the file search tool.
    ///
    /// Not compatible with chat completions api.
    pub const FILE_SEARCH: Self = ToolChoice::Hosted("file_search");

    /// The model must use the web search preview tool.
    ///
    /// Not compatible with chat completions api.
    pub const WEB_SEARCH_PREVIEW: Self = ToolChoice::Hosted("web_search_preview");

    /// The model must use the computer use preview tool.
    ///
    /// Not compatible with chat completions api.
    pub const COMPUTER_USE_PREVIEW: Self = ToolChoice::Hosted("computer_use_preview");

    /// The model must call exactly one specific function.
    #[must_use]
    #[inline]
    pub fn function(name: impl Into<String>) -> Self {
        ToolChoice::Function(name.into())
    }

    /// The model must use the file search tool.
    ///
    /// Not compatible with chat completions api.
    #[must_use]
    #[inline]
    pub const fn file_search() -> Self {
        Self::FILE_SEARCH
    }

    /// The model must use the web search preview tool.
    ///
    /// Not compatible with chat completions api.
    #[must_use]
    #[inline]
    pub const fn web_search_preview() -> Self {
        Self::WEB_SEARCH_PREVIEW
    }

    /// The model must use the computer use preview tool.
    ///
    /// Not compatible with chat completions api.
    #[must_use]
    #[inline]
    pub const fn computer_use_preview() -> Self {
        Self::COMPUTER_USE_PREVIEW
    }
}

impl From<ToolChoice> for ChatCompletionToolChoiceOption {
    fn from(value: ToolChoice) -> Self {
        match value {
            ToolChoice::None => ChatCompletionToolChoiceOption::None,
            ToolChoice::Auto => ChatCompletionToolChoiceOption::Auto,
            ToolChoice::Required => ChatCompletionToolChoiceOption::Required,
            ToolChoice::Function(name) => {
                ChatCompletionToolChoiceOption::Named(ChatCompletionNamedToolChoice {
                    r#type: async_openai::types::ChatCompletionToolType::Function,
                    function: FunctionName { name },
                })
            }
            ToolChoice::Hosted(hosted) => {
                log::warn!("Hosted function {hosted} is not supported for the api.");
                ChatCompletionToolChoiceOption::Auto
            }
        }
    }
}

impl From<ChatCompletionToolChoiceOption> for ToolChoice {
    fn from(value: ChatCompletionToolChoiceOption) -> Self {
        match value {
            ChatCompletionToolChoiceOption::None => ToolChoice::None,
            ChatCompletionToolChoiceOption::Auto => ToolChoice::Auto,
            ChatCompletionToolChoiceOption::Required => ToolChoice::Required,
            ChatCompletionToolChoiceOption::Named(choice) => {
                ToolChoice::Function(choice.function.name)
            }
        }
    }
}

impl From<ToolChoice> for ResponsesToolChoice {
    fn from(value: ToolChoice) -> Self {
        match value {
            ToolChoice::None => ResponsesToolChoice::Mode(ToolChoiceMode::None),
            ToolChoice::Auto => ResponsesToolChoice::Mode(ToolChoiceMode::Auto),
            ToolChoice::Required => ResponsesToolChoice::Mode(ToolChoiceMode::Required),
            ToolChoice::Function(name) => ResponsesToolChoice::Function { name },
            ToolChoice::Hosted("file_search") => ResponsesToolChoice::Hosted {
                kind: HostedToolType::FileSearch,
            },
            ToolChoice::Hosted("web_search_preview") => ResponsesToolChoice::Hosted {
                kind: HostedToolType::WebSearchPreview,
            },
            ToolChoice::Hosted("computer_use_preview") => ResponsesToolChoice::Hosted {
                kind: HostedToolType::ComputerUsePreview,
            },
            ToolChoice::Hosted(hosted) => {
                log::warn!("Hosted function {hosted} is not supported for the api.");
                ResponsesToolChoice::Mode(ToolChoiceMode::Auto)
            }
        }
    }
}

impl From<ResponsesToolChoice> for ToolChoice {
    fn from(value: ResponsesToolChoice) -> Self {
        match value {
            ResponsesToolChoice::Mode(ToolChoiceMode::None) => ToolChoice::None,
            ResponsesToolChoice::Mode(ToolChoiceMode::Auto) => ToolChoice::Auto,
            ResponsesToolChoice::Mode(ToolChoiceMode::Required) => ToolChoice::Required,
            ResponsesToolChoice::Function { name } => ToolChoice::Function(name),
            ResponsesToolChoice::Hosted {
                kind: HostedToolType::FileSearch,
            } => ToolChoice::Hosted("file_search"),
            ResponsesToolChoice::Hosted {
                kind: HostedToolType::WebSearchPreview,
            } => ToolChoice::Hosted("web_search_preview"),
            ResponsesToolChoice::Hosted {
                kind: HostedToolType::ComputerUsePreview,
            } => ToolChoice::Hosted("computer_use_preview"),
        }
    }
}
