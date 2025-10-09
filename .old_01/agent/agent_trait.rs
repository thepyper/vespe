use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Agent {
    fn name(&self) -> &str;
    async fn execute(&self, input: &str) -> Result<String>;
}
