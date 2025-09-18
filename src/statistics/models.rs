use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::llm::parsing::match_source::ParserSource;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UsageStatistics {
    pub parser_usage: HashMap<String, HashMap<ParserSource, u64>>, // "{Provider}::{Model}" -> ParserSource -> Count
    pub llm_invocations: HashMap<String, u64>, // "{Provider}::{Model}" -> Count
}
