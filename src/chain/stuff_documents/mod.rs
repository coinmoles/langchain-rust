mod chain;
pub use chain::*;

mod builder;
pub use builder::*;

mod prompt;

const COMBINE_DOCUMENTS_DEFAULT_INPUT_KEY: &str = "input_documents";
// const COMBINE_DOCUMENTS_DEFAULT_OUTPUT_KEY: &str = "text";
const COMBINE_DOCUMENTS_DEFAULT_DOCUMENT_VARIABLE_NAME: &str = "context";
const STUFF_DOCUMENTS_DEFAULT_SEPARATOR: &str = "\n\n";
