use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::execute::{Collector, Worker};
use super::tags::StaticPolicy;
use crate::ast2::Tag;

pub struct SetPolicy;

impl StaticPolicy for SetPolicy {
    fn collect_static_tag(_worker: &Worker, collector: Collector, tag: &Tag) -> Result<Collector> {
        tracing::debug!("tag_set::SetPolicy::collect_static_tag\nTag = {:?}\n", tag);
        Ok(collector.update(&tag.parameters))
    }
}
