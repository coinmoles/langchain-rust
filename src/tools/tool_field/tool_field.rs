use serde_json::Value;

pub trait ToolField: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> Option<&str>;
    fn required(&self) -> bool;
    fn to_openai_field(&self) -> Value;
    fn to_plain_description(&self) -> String;
    fn clone_box(&self) -> Box<dyn ToolField>;
}
