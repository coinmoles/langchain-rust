use crate::schemas::AgentStep;

pub trait FinalAnswerValidator: Send + Sync {
    fn validate_final_answer(&self, final_answer: &str, intermediate_steps: &[AgentStep]) -> bool;
}
