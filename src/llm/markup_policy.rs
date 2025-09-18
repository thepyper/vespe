
pub trait MarkupPolicy: Send + Sync {
    fn get_markup_instructions(&self) -> String;
    fn name(&self) -> &str;
}
