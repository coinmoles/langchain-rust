use std::sync::Arc;

use crate::agent::FinalAnswerValidator;

pub struct ExecutorOptions {
    pub max_iterations: Option<usize>,
    pub max_consecutive_fails: Option<usize>,
    pub break_if_tool_error: bool,
    pub final_answer_validator: Option<Arc<dyn FinalAnswerValidator>>,
}

impl ExecutorOptions {
    pub fn new(
        max_iterations: Option<usize>,
        max_consecutive_fails: Option<usize>,
        break_if_tool_error: bool,
        final_answer_validator: Option<Arc<dyn FinalAnswerValidator>>,
    ) -> Self {
        Self {
            max_iterations,
            max_consecutive_fails,
            break_if_tool_error,
            final_answer_validator,
        }
    }

    pub fn with_max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = Some(max_iterations);
        self
    }

    pub fn without_max_iterations(mut self) -> Self {
        self.max_iterations = None;
        self
    }

    pub fn with_max_consecutive_fails(mut self, max_consecutive_fails: usize) -> Self {
        self.max_consecutive_fails = Some(max_consecutive_fails);
        self
    }

    pub fn without_max_consecutive_fails(mut self) -> Self {
        self.max_consecutive_fails = None;
        self
    }

    pub fn with_break_if_tool_error(mut self, break_if_tool_error: bool) -> Self {
        self.break_if_tool_error = break_if_tool_error;
        self
    }

    pub fn with_final_answer_validator(
        mut self,
        final_answer_validator: Arc<dyn FinalAnswerValidator>,
    ) -> Self {
        self.final_answer_validator = Some(final_answer_validator);
        self
    }
}

impl Default for ExecutorOptions {
    fn default() -> Self {
        Self {
            max_iterations: Some(10),
            max_consecutive_fails: Some(3),
            break_if_tool_error: false,
            final_answer_validator: None,
        }
    }
}
