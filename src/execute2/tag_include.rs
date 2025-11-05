use super::{ExecuteError, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::execute::{Collector, Worker};
use super::tags::StaticPolicy;
//use super::variables::Variables;
use crate::ast2::Tag;
pub struct IncludePolicy;

impl StaticPolicy for IncludePolicy {
    fn collect_static_tag(worker: &Worker, collector: Collector, tag: &Tag) -> Result<Collector> {
        tracing::debug!(
            "tag_include::IncludePolicy::collect_static_tag\nTag = {:?}\n",
            tag
        );
        let included_context_name = tag
            .arguments
            .arguments
            .get(0)
            .ok_or_else(|| ExecuteError::MissingParameter("include tag argument".to_string()))?
            .value
            .clone();
        match worker._execute(collector, &included_context_name, 0)? {
            Some(collector) => Ok(collector),
            None => Err(ExecuteError::Generic(
                "Included context returned no collector".to_string(),
            )),
        }
    }
}
