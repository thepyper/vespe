use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::execute::{Collector, Worker};
use super::tags::StaticPolicy;
//use super::variables::Variables;
use crate::ast2::Tag;
pub struct ForgetPolicy;

impl StaticPolicy for ForgetPolicy {
    fn collect_static_tag(worker: &Worker, collector: Collector, tag: &Tag) -> Result<Collector> {
        tracing::debug!(
            "tag_forget::ForgetPolicy::collect_static_tag\nTag = {:?}\n",
            tag
        );
        Ok(collector.forget())
    }
}
