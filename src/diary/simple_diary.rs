use crate::schemas::ToolCall;

use super::{Diary, DiaryStep};

pub struct SimpleDiary {
    steps: Vec<DiaryStep>,
}

impl SimpleDiary {
    pub fn new(steps: impl IntoIterator<Item = DiaryStep>) -> Self {
        Self {
            steps: steps.into_iter().collect(),
        }
    }

    pub fn push_step(&mut self, tool_call: ToolCall, result: impl Into<String>) {
        let step = DiaryStep::new(tool_call, result);
        self.steps.push(step);
    }
}

impl Diary for SimpleDiary {
    fn push_step(&mut self, tool_call: ToolCall, result: String) {
        self.push_step(tool_call, result);
    }

    fn get_steps(&self) -> Vec<DiaryStep> {
        self.steps.clone()
    }
}

impl Default for SimpleDiary {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn test_simple_diary() {
        let mut diary = SimpleDiary::default();

        diary.push_step(ToolCall::new("", "tool", Value::Null), "result");

        let steps = diary.get_steps();
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].tool_call.name, "tool");
        assert_eq!(steps[0].result, "result");
    }
}
