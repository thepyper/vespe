//! This module implements the `DonePolicy` for the `@done` tag. The `@done`
//! ... TODO doc
use super::Result;

use super::tags::{TagOrAnchor, StaticPolicy, StaticPolicyMonoInput, StaticPolicyMonoResult};
use super::tag_task::{TaskStatus, TaskState};
use crate::ast2::{CommandKind};

/// Implements the static policy for the `@done` tag.
///
// TODO doc
pub struct DonePolicy;

impl StaticPolicy for DonePolicy {
    // TODO doc
    fn mono(inputs: StaticPolicyMonoInput) -> Result<StaticPolicyMonoResult> {
        let (mut result, residual) = StaticPolicyMonoResult::from_inputs(inputs);
        let tag = match residual.tag_or_anchor {
            TagOrAnchor::Tag(tag) => tag,
            _ => {
                panic!("!?!?!? cannot be anchor in static tag !?!?!?"); // better error TODO
            }
        };
        match result.collector.latest_task() {
            Some(anchor) => {
                match anchor.command {
                            CommandKind::Task => {
                                let mut task_state = residual
                                    .worker
                                    .load_state::<TaskState>(anchor.command, &anchor.uuid)?;
                                 task_state.status = TaskStatus::Eating;
                                task_state.eating_end = tag.range.begin;
                                residual.worker.save_state::<TaskState>(
                                    anchor.command,
                                    &anchor.uuid,
                                    &task_state,
                                    None,
                                )?;           
                                result.new_patches = vec![
                                    (tag.range, String::new())
                                ];                    
                            }
                            _ => {
                                panic!("not a task anchor!=!=!="); // TODO better error
                            }
                        }
            }
            None => {
                                panic!("no previous task anchor!=!=!="); // TODO better error
            }
        }
        Ok(result)
    }
}
