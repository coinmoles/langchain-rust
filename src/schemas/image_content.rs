use async_openai::types::responses::{InputImage, InputImageArgs};
use async_openai::types::{ChatCompletionRequestMessageContentPartImage, ImageDetail, ImageUrl};
use serde::{Deserialize, Serialize};

/// An image provided to an LLM.
///
/// Corresponds to [`ImageUrl`](async_openai::types::ImageUrl) for the chat completions api
/// and [`InputImage`](async_openai::types::responses::InputImage) for the responses api.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ImageContent {
    /// Either a URL of the image or the base64 encoded image data.
    pub image_url: String,
    /// The level of detail for the image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<ImageDetail>,
}

impl ImageContent {
    /// Constructs a new `ImageContent` with the given url.
    pub fn new(image_url: impl Into<String>) -> Self {
        ImageContent {
            image_url: image_url.into(),
            detail: None,
        }
    }
}

impl From<ImageContent> for ImageUrl {
    fn from(value: ImageContent) -> Self {
        ImageUrl {
            url: value.image_url,
            detail: value.detail,
        }
    }
}

impl From<ImageContent> for ChatCompletionRequestMessageContentPartImage {
    fn from(value: ImageContent) -> Self {
        ChatCompletionRequestMessageContentPartImage {
            image_url: value.into(),
        }
    }
}

impl From<ImageContent> for InputImage {
    fn from(value: ImageContent) -> Self {
        InputImageArgs::default()
            .image_url(value.image_url)
            .detail(value.detail.unwrap_or_default())
            .build()
            .expect("All required fields are set.")
    }
}
