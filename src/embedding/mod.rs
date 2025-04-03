mod error;

pub mod embedder_trait;
pub use embedder_trait::*;

pub mod openai;
pub use error::*;

#[cfg(feature = "fastembed")]
mod fastembed;
#[cfg(feature = "fastembed")]
pub use fastembed::*;

#[cfg(feature = "mistralai")]
pub mod mistralai;
#[cfg(feature = "mistralai")]
pub use mistralai::*;
