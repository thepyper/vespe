use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::llm::parsing::match_source::ParserSource;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ParserStats {
    pub usage: u64,
    pub fading_usage: f64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ModelStats {
    pub invocations: u64,
    pub fading_invocations: f64,
    pub parser_stats: HashMap<ParserSource, ParserStats>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UsageStatistics {
    pub model_stats: HashMap<String, ModelStats>, // "{Provider}::{Model}" -> ModelStats
}