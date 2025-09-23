
pub trait MarkupPolicy: Send + Sync {
    fn get_markup_instructions(&self) -> String;
    fn get_tool_invocation_preamble(&self) -> String;
    fn name(&self) -> &str;
}
