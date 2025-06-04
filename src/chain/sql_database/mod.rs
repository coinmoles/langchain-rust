mod builder;
mod chain;
mod input;
mod prompt;

pub use builder::*;
pub use chain::*;
pub use input::*;
pub use prompt::*;

const STOP_WORD: &str = "\nSQLResult:";
// const SQL_CHAIN_DEFAULT_OUTPUT_KEY: &str = "result";
const QUERY_PREFIX_WITH: &str = "\nSQLQuery:";
