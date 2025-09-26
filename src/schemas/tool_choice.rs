use async_openai::types::responses::{HostedToolType, ToolChoiceMode};
use async_openai::types::{
    ChatCompletionNamedToolChoice, ChatCompletionToolChoiceOption, FunctionName,
};

/// A parameter to control which (if any) tool is called by the model.
///
/// Corresponds to [`ChatCompletionToolChoiceOption`] for the chat completions api and
/// [`ToolChoice`](async_openai::types::responses::ToolChoice) for the responses api.
#[derive(Clone, Debug)]
pub struct ToolChoice(async_openai::types::responses::ToolChoice);

impl ToolChoice {
    /// The model will not call any tool and instead generate a message.
    pub const NONE: Self = ToolChoice(async_openai::types::responses::ToolChoice::Mode(
        ToolChoiceMode::None,
    ));

    /// The model can pick between generating a message or calling one or more tools.
    pub const AUTO: Self = ToolChoice(async_openai::types::responses::ToolChoice::Mode(
        ToolChoiceMode::Auto,
    ));

    /// The model must call one or more tools.
    pub const REQUIRED: Self = ToolChoice(async_openai::types::responses::ToolChoice::Mode(
        ToolChoiceMode::Required,
    ));

    /// The model must use the file search tool.
    ///
    /// Not compatible with chat completions api.
    pub const FILE_SEARCH: Self = ToolChoice(async_openai::types::responses::ToolChoice::Hosted {
        kind: HostedToolType::FileSearch,
    });

    /// The model must use the web search preview tool.
    ///
    /// Not compatible with chat completions api.
    pub const WEB_SEARCH_PREVIEW: Self =
        ToolChoice(async_openai::types::responses::ToolChoice::Hosted {
            kind: HostedToolType::WebSearchPreview,
        });

    /// The model must use the computer use preview tool.
    ///
    /// Not compatible with chat completions api.
    pub const COMPUTER_USE_PREVIEW: Self =
        ToolChoice(async_openai::types::responses::ToolChoice::Hosted {
            kind: HostedToolType::ComputerUsePreview,
        });

    /// The model will not call any tool and instead generate a message.
    #[must_use]
    #[inline]
    pub const fn none() -> Self {
        Self::NONE
    }

    /// The model can pick between generating a message or calling one or more tools.
    #[must_use]
    #[inline]
    pub const fn auto() -> Self {
        Self::AUTO
    }

    /// The model must call one or more tools.
    #[must_use]
    #[inline]
    pub const fn required() -> Self {
        Self::REQUIRED
    }

    /// The model must call the specified tool.
    #[must_use]
    #[inline]
    pub fn function(name: impl Into<String>) -> Self {
        ToolChoice(async_openai::types::responses::ToolChoice::Function { name: name.into() })
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
        match value.0 {
            async_openai::types::responses::ToolChoice::Mode(ToolChoiceMode::Auto) => {
                ChatCompletionToolChoiceOption::Auto
            }
            async_openai::types::responses::ToolChoice::Mode(ToolChoiceMode::None) => {
                ChatCompletionToolChoiceOption::None
            }
            async_openai::types::responses::ToolChoice::Mode(ToolChoiceMode::Required) => {
                ChatCompletionToolChoiceOption::Required
            }
            async_openai::types::responses::ToolChoice::Function { name } => {
                ChatCompletionToolChoiceOption::Named(ChatCompletionNamedToolChoice {
                    r#type: async_openai::types::ChatCompletionToolType::Function,
                    function: FunctionName { name },
                })
            }
            _ => ChatCompletionToolChoiceOption::Auto,
        }
    }
}

impl From<ChatCompletionToolChoiceOption> for ToolChoice {
    fn from(value: ChatCompletionToolChoiceOption) -> Self {
        match value {
            ChatCompletionToolChoiceOption::Auto => ToolChoice(
                async_openai::types::responses::ToolChoice::Mode(ToolChoiceMode::Auto),
            ),
            ChatCompletionToolChoiceOption::None => ToolChoice(
                async_openai::types::responses::ToolChoice::Mode(ToolChoiceMode::None),
            ),
            ChatCompletionToolChoiceOption::Required => ToolChoice(
                async_openai::types::responses::ToolChoice::Mode(ToolChoiceMode::Required),
            ),
            ChatCompletionToolChoiceOption::Named(named) => {
                ToolChoice(async_openai::types::responses::ToolChoice::Function {
                    name: named.function.name,
                })
            }
        }
    }
}

impl From<ToolChoice> for async_openai::types::responses::ToolChoice {
    fn from(value: ToolChoice) -> Self {
        value.0
    }
}

impl From<async_openai::types::responses::ToolChoice> for ToolChoice {
    fn from(value: async_openai::types::responses::ToolChoice) -> Self {
        ToolChoice(value)
    }
}
