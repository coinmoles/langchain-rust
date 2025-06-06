use std::{io, string::FromUtf8Error};

use thiserror::Error;

use crate::text_splitter::TextSplitterError;

#[derive(Error, Debug)]
pub enum LoaderError {
    #[error("Error loading document: {0}")]
    LoadDocumentError(String),

    #[error("{0}")]
    TextSplitterError(#[from] TextSplitterError),

    #[error(transparent)]
    IOError(#[from] io::Error),

    #[error(transparent)]
    FromUtf8Error(#[from] FromUtf8Error),

    #[error(transparent)]
    CSVError(#[from] csv::Error),

    #[cfg(feature = "lopdf")]
    #[cfg(not(feature = "pdf-extract"))]
    #[error(transparent)]
    LoPdfError(#[from] lopdf::Error),

    #[cfg(feature = "pdf-extract")]
    #[error(transparent)]
    PdfExtractError(#[from] pdf_extract::Error),

    #[cfg(feature = "pdf-extract")]
    #[error(transparent)]
    PdfExtractOutputError(#[from] pdf_extract::OutputError),

    #[error(transparent)]
    ReadabilityError(#[from] readability::error::Error),

    #[error(transparent)]
    JoinError(#[from] tokio::task::JoinError),

    #[cfg(feature = "git")]
    #[error(transparent)]
    DiscoveryError(Box<gix::discover::Error>),

    #[error("Error: {0}")]
    OtherError(String),
}

#[cfg(feature = "git")]
impl From<gix::discover::Error> for LoaderError {
    fn from(err: gix::discover::Error) -> Self {
        LoaderError::DiscoveryError(Box::new(err))
    }
}
