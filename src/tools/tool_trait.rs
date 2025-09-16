use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait Tool {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> &str; // JSON Schema string
    async fn execute(&self, input: &Value) -> Result<Value>; // Input and output as JSON Value
}
