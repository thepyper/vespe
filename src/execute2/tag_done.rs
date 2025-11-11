//! This module implements the `DonePolicy` for the `@done` tag. The `@done`
//! ... TODO doc
use super::Result;

use super::tags::{StaticPolicy, StaticPolicyMonoInput, StaticPolicyMonoResult};

/// Implements the static policy for the `@done` tag.
///
// TODO doc
pub struct DonePolicy;

impl StaticPolicy for DonePolicy {
    // TODO doc
    fn mono(inputs: StaticPolicyMonoInput) -> Result<StaticPolicyMonoResult> {
        // TODO
        unimplemented!()
    }
}
