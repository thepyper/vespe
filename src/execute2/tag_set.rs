use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::variables::Variables;
use super::execute::{Collector, Worker};
use super::tags::StaticPolicy;
use crate::ast2::Tag;

pub struct SetPolicy;

impl StaticPolicy for SetPolicy {
    fn collect_static_tag(worker: &Worker, collector: Collector, local_variables: &Variables, tag: &Tag) -> Result<Collector> {
        tracing::debug!("tag_set::SetPolicy::collect_static_tag\nTag = {:?}\n", tag);
        Ok(collector.update_variables(&local_variables))
    }
}
