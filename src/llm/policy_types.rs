// src/llm/policy_types.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyType {
    Json,
    Xml,
    Sections,
    // Aggiungere altri tipi di policy qui se necessario
}
