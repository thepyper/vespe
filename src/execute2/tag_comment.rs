
use super::{ExecuteError, Result};

use super::execute::{Collector, Worker};
use super::tags::StaticPolicy;
use crate::ast2::Tag;

pub struct CommentPolicy;

impl StaticPolicy for CommentPolicy {
    
    fn collect_static_tag(worker: &Worker, collector: Collector, tag: &Tag) -> Result<Collector> {        
        Ok(collector)
    }
}
