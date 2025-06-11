use thiserror::Error;

#[derive(Error, Debug)]
pub enum BuilderError {
    #[error("Missing required field: {0}")]
    MissingField(&'static str),
    #[error("Missing required field in {0}: {1}")]
    Inner(&'static str, Box<BuilderError>),
    #[error("Other error: {0}")]
    Other(String),
}
