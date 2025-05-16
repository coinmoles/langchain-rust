use std::error::Error;

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::tools::ToolFunction;

pub struct CommandExecutor {
    platform: String,
}

impl CommandExecutor {
    /// Create a new CommandExecutor instance
    /// # Example
    /// ```rust,ignore
    /// let tool = CommandExecutor::new("linux");
    /// ```
    pub fn new<S: Into<String>>(platform: S) -> Self {
        Self {
            platform: platform.into(),
        }
    }
}

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new("linux")
    }
}

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(description = "Object representing a command and its optional arguments")]
pub struct Command {
    #[schemars(description = "The command to execute")]
    pub cmd: String,
    #[serde(default)]
    #[schemars(description = "List of arguments for the command")]
    pub args: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(description = "An array of commands to be executed")]
pub struct CommandExecutorInput(pub Vec<Command>);

#[async_trait]
impl ToolFunction for CommandExecutor {
    type Input = CommandExecutorInput;
    type Result = String;

    fn name(&self) -> String {
        "Command Executor".into()
    }

    fn description(&self) -> String {
        format!(
            r#""This tool let you run command on the terminal"
            "The input should be an array with commands for the following platform: {}"
            "examle of input: [{{ "cmd": "ls", "args": [] }},{{"cmd":"mkdir","args":["test"]}}]"
            "Should be a comma separated commands"
            "#,
            self.platform
        )
    }

    fn inline_subschema(&self) -> bool {
        true
    }

    async fn run(&self, input: Self::Input) -> Result<Self::Result, Box<dyn Error + Send + Sync>> {
        let commands = input.0;
        let mut result = String::new();

        for command in commands {
            let mut command_to_execute = std::process::Command::new(&command.cmd);
            command_to_execute.args(&command.args);

            let output = command_to_execute.output()?;

            result.push_str(&format!(
                "Command: {}\nOutput: {}",
                command.cmd,
                String::from_utf8_lossy(&output.stdout),
            ));

            if !output.status.success() {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "Command {} failed with status: {}",
                        command.cmd, output.status
                    ),
                )));
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::*;

    #[tokio::test]
    async fn test_with_string_executor() {
        let tool = CommandExecutor::new("linux");
        let input = CommandExecutorInput(vec![Command {
            cmd: "ls".into(),
            args: vec![],
        }]);
        let result = tool.run(input).await.unwrap();
        println!("Res: {}", result);
    }

    #[test]
    fn test_command_executor_input_schema() {
        let schema = CommandExecutor::default().parameters();
        let schema = serde_json::to_value(schema).unwrap();

        let expected = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "CommandExecutorInput",
            "type": "array",
            "description": "An array of commands to be executed",
            "items": {
                "type": "object",
                "description": "Object representing a command and its optional arguments",
                "properties": {
                    "cmd": {
                        "type": "string",
                        "description": "The command to execute",
                    },
                    "args": {
                        "type": "array",
                        "description": "List of arguments for the command",
                        "items": { "type": "string" },
                        "default": [],
                    },
                },
                "required": ["cmd"],
                "additionalProperties": false
            }
        });

        assert_eq!(schema, expected)
    }
}
