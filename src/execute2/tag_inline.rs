//! Implements the behavior for the dynamic `@inline` tag.
//!
//! The `@inline` tag is used to dynamically include content from another context file.
//! Unlike the static `@include` tag, `@inline` is stateful, allowing its content
//! to be re-evaluated or "repeated" during the execution flow. This is useful when
//! the inlined content needs to be refreshed after other operations have modified
//! the execution state.

use serde::{Deserialize, Serialize};

use super::error::ExecuteError;
use super::tags::{
    Container, DynamicPolicy, DynamicPolicyMonoInput, DynamicPolicyMonoResult, DynamicState,
};
use super::Result;
use crate::ast2::JsonPlusEntity;
use std::str::FromStr;

/// Represents the execution status of an `@inline` tag.
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub enum InlineStatus {
    /// The `@inline` tag has just been converted into an anchor and its content
    /// needs to be loaded for the first time.
    #[default]
    JustCreated,
    /// The `@inline` tag is in a state where it needs to be re-executed, forcing
    /// a reload of its content from the source file.
    Repeat,
    /// The `@inline` tag has successfully loaded its content, and no further
    /// action is needed unless its state is changed to `Repeat`.
    Completed,
}

impl ToString for InlineStatus {
    fn to_string(&self) -> String {
        match self {
            InlineStatus::JustCreated => "created".to_string(),
            InlineStatus::Repeat => "repeat".to_string(),
            InlineStatus::Completed => "completed".to_string(),
        }
    }
}

impl FromStr for InlineStatus {
    type Err = ExecuteError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "created" => Ok(InlineStatus::JustCreated),
            "repeat" => Ok(InlineStatus::Repeat),
            "completed" => Ok(InlineStatus::Completed),
            _ => Err(ExecuteError::UnsupportedStatus(s.to_string())),
        }
    }
}

/// Holds the persistent state for an `@inline` anchor.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct InlineState {
    /// The current status of the `@inline` anchor.
    pub status: InlineStatus,
}

/// Implements the dynamic policy for the `@inline` tag.
///
/// This policy defines how an `@inline` tag and its corresponding anchor behave
/// during the execution process, managing state transitions to allow for dynamic

/// content loading and refreshing.
pub struct InlinePolicy;

impl DynamicPolicy for InlinePolicy {
    /// The state object associated with this policy.
    type State = InlineState;

    /// Executes a single step of the `@inline` tag's lifecycle.
    ///
    /// This method handles the state transitions for an `@inline` anchor:
    /// - `JustCreated`: Reads the content from the file specified in the tag's
    ///   arguments, sets it as the new output, and transitions the state to `Completed`.
    ///   It triggers a new pass to process the injected content.
    /// - `Completed`: No action is taken, as the content is already present.
    /// - `Repeat`: Resets the state to `JustCreated` and clears the existing content,
    ///   triggering a new pass to force a reload of the content from the source file.
    ///
    /// # Arguments
    ///
    /// * `worker` - A reference to the [`Worker`] instance.
    /// * `collector` - The current [`Collector`] state.
    /// * `_input` - The [`ModelContent`] collected so far (unused in this policy).
    /// * `_parameters` - The [`Parameters`] associated with the tag (unused).
    /// * `arguments` - The [`Arguments`] containing the path to the context file to inline.
    /// * `state` - The current [`InlineState`] of the anchor.
    /// * `_readonly` - A boolean indicating if the current pass is read-only (unused).
    ///
    /// # Returns
    ///
    /// A `Result` containing a [`DynamicPolicyMonoResult`] describing the outcome
    /// of this execution step, including any new state or output.
    ///
    /// # Errors
    ///
    /// Returns an [`ExecuteError`] if the context name is missing from the arguments
    /// or if the file cannot be read.
    fn mono(
        inputs: DynamicPolicyMonoInput<Self::State>,
        /*
        worker: &Worker,
        collector: Collector,
        _input: &ModelContent,
        _input_hash: String,
        parameters: &Parameters,
        arguments: &Arguments,
        mut state: Self::State,
        _readonly: bool,
        */
    ) -> Result<DynamicPolicyMonoResult<Self::State>> {
        tracing::debug!("tag_inline::InlinePolicy::mono\nState = {:?}", inputs.state);
        let (mut result, mut residual) =
            DynamicPolicyMonoResult::<Self::State>::from_inputs(inputs);
        match (residual.container, residual.state.status) {
            (Container::Tag(_) | Container::BeginAnchor(_, _), InlineStatus::JustCreated) => {
                if !residual.readonly {
                    let context_name = residual
                        .arguments
                        .arguments
                        .get(0)
                        .ok_or_else(|| ExecuteError::MissingInlineArgument {
                            range: residual.arguments.range,
                        })?
                        .value
                        .clone();
                    // Load content from the specified context
                    residual.state.status = InlineStatus::Completed;
                    result.new_state = Some(residual.state);
                    let context = residual.worker.read_context(&context_name)?;
                    let context = Self::slice_with_markers(
                        &context,
                        residual.parameters.get_as_integer_only("begin_line"),
                        residual.parameters.get_as_string_only("begin_marker").as_deref(),
                        residual.parameters.get_as_integer_only("end_line"),
                        residual.parameters.get_as_string_only("end_marker").as_deref(),
                    );
                    let context = match residual.parameters.get("data") {
                        Some(JsonPlusEntity::Object(data)) => {
                            residual.worker.process_context_with_data(context, data)?
                        }
                        Some(_) => {
                            return Err(ExecuteError::UnsupportedDataParameter {
                                range: residual.parameters.range,
                            });
                        }
                        None => context,
                    };
                    result.new_output = Some(context);
                }
                result.do_next_pass = true;
            }
            (Container::BeginAnchor(_, _), InlineStatus::Completed) => {
                // Nothing to do
            }
            (Container::BeginAnchor(_, _), InlineStatus::Repeat) => {
                // Reset state to force a reload in the next pass
                if !residual.readonly {
                    residual.state.status = InlineStatus::JustCreated;
                    result.new_state = Some(residual.state);
                    result.new_output = Some(String::new());
                }
                result.do_next_pass = true;
            }
            _ => {}
        }
        Ok(result)
    }
}

impl InlinePolicy {
    fn slice_with_markers(
        text: &str,
        begin_line: Option<i64>,
        begin_marker: Option<&str>,
        end_line: Option<i64>,
        end_marker: Option<&str>,
    ) -> String {
        let lines: Vec<&str> = text.lines().collect();

        let begin_marker_line = if let Some(marker) = begin_marker {
            lines
                .iter()
                .position(|l| l.contains(marker))
                .map(|idx| idx as i64)
                .unwrap_or(0) // default se marker non trovato
        } else {
            0
        };

        let end_marker_line = if let Some(marker) = end_marker {
            lines
                .iter()
                .rposition(|l| l.contains(marker))
                .map(|idx| idx as i64)
                .unwrap_or(lines.len() as i64) // default se non trovato
        } else {
            lines.len() as i64
        };

        let begin = begin_marker_line + begin_line.unwrap_or(0);
        let end = end_marker_line + end_line.unwrap_or(0);

        let begin = begin.clamp(0, lines.len() as i64) as usize;
        let end = end.clamp(0, lines.len() as i64) as usize;

        if begin >= end {
            return String::new();
        }

        lines[begin..end].join("\n")
    }
}

impl DynamicState for InlineState {
    fn status_indicator(&self) -> String {
        self.status.to_string()
    }
}
