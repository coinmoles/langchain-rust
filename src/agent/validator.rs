use crate::schemas::ToolCall;

pub trait FinalAnswerValidator: Send + Sync {
    fn validate_final_answer(
        &self,
        final_answer: &str,
        intermediate_steps: &[(ToolCall, String)],
    ) -> bool;
}
