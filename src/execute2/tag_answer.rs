//! This module implements the `AnswerPolicy` for the `@answer` tag, a dynamic tag
//! that orchestrates interaction with an external model (LLM). It defines the
//! state machine for the `@answer` tag's lifecycle, from initial creation to
//! processing the model's response and injecting it into the document.
use super::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt;
use std::str::FromStr;

use super::content::{ModelContent, ModelContentItem};
use super::error::ExecuteError;
use super::execute::Worker;
use super::tags::{Container, DynamicPolicy, DynamicPolicyMonoInput, DynamicPolicyMonoResult};
use crate::ast2::{JsonPlusEntity, Parameters, Range};

use handlebars::Handlebars;

/// Represents the current status of an `@answer` tag's execution.
///
/// This enum defines the different stages a dynamic `@answer` tag goes through
/// during the multi-pass execution, from being initially created to having its
/// response completed and injected.
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub enum AnswerStatus {
    /// The `@answer` tag has just been created and needs to be processed.
    #[default]
    JustCreated,
    /// The `@answer` tag is in a state where it needs to re-execute the model call.
    Repeat,
    /// The `@answer` tag needs to call the external model to get a response.
    NeedProcessing,
    /// The model has responded, and its reply needs to be injected into the document.
    NeedInjection,
    /// The `@answer` tag has completed its execution, and its response is in the document.
    Completed,
    /// The `@answer` tag content has been edited by user, then it must be seen as user conten by llm.
    Edited,
}

impl fmt::Display for AnswerStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AnswerStatus::JustCreated => write!(f, "JustCreated"),
            AnswerStatus::Repeat => write!(f, "Repeat"),
            AnswerStatus::NeedProcessing => write!(f, "NeedProcessing"),
            AnswerStatus::NeedInjection => write!(f, "NeedInjection"),
            AnswerStatus::Completed => write!(f, "Completed"),
            AnswerStatus::Edited => write!(f, "Edited"),
        }
    }
}

impl FromStr for AnswerStatus {
    type Err = ExecuteError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "JustCreated" => Ok(AnswerStatus::JustCreated),
            "Repeat" => Ok(AnswerStatus::Repeat),
            "NeedProcessing" => Ok(AnswerStatus::NeedProcessing),
            "NeedInjection" => Ok(AnswerStatus::NeedInjection),
            "Completed" => Ok(AnswerStatus::Completed),
            "Edited" => Ok(AnswerStatus::Edited),
            _ => Err(ExecuteError::UnsupportedParameterValue(format!(
                "Unknown AnswerStatus: {}",
                s
            ))),
        }
    }
}

/// Represents the persistent state of an `@answer` tag.
///
/// This struct holds the current status of the answer process and the model's
/// reply, allowing the execution engine to resume processing across multiple passes.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AnswerState {
    /// The current status of the `@answer` tag.
    pub status: AnswerStatus,
    /// The query sent to the external model
    pub query: String,
    /// The reply received from the external model.
    pub reply: String,
    /// The context hash
    pub context_hash: String,
    /// The reply hash
    pub reply_hash: String,
}

/// Implements the dynamic policy for the `@answer` tag.
///
/// This policy defines how the `@answer` tag behaves during the execution
/// process, managing its state transitions and interactions with the external model.
pub struct AnswerPolicy;

impl DynamicPolicy for AnswerPolicy {
    /// The associated state for the `AnswerPolicy` is [`AnswerState`].
    type State = AnswerState;

    /// Executes a single step of the `@answer` tag's lifecycle.
    ///
    /// This method handles the state transitions of an `@answer` tag:
    /// - `JustCreated`: Transitions to `NeedProcessing` and triggers a new pass.
    /// - `NeedProcessing`: Calls the external model, stores the response, transitions
    ///   to `NeedInjection`, and triggers a new pass.
    /// - `NeedInjection`: Prepares the model's reply for injection into the document,
    ///   transitions to `Completed`, and triggers a new pass.
    /// - `Completed`: No action, the tag is resolved.
    /// - `Repeat`: Resets the state to `NeedProcessing` and triggers a new pass to re-execute.
    ///
    /// # Arguments
    ///
    /// * `worker` - A reference to the [`Worker`] instance.
    /// * `collector` - The current [`Collector`] state.
    /// * `input` - The [`ModelContent`] collected so far, serving as input to the model.
    /// * `parameters` - The [`Parameters`] associated with the `@answer` tag.
    /// * `arguments` - The [`Arguments`] associated with the `@answer` tag.
    /// * `state` - The current [`AnswerState`] of the tag.
    /// * `readonly` - A boolean indicating if the current pass is read-only.
    ///
    /// # Returns
    ///
    /// A `Result` containing a [`DynamicPolicyMonoResult`] describing the outcome
    /// of this execution step.
    ///
    /// # Errors
    ///
    /// Returns an [`ExecuteError`] if the model call fails or if there are issues
    /// with parameters.
    fn mono(
        inputs: DynamicPolicyMonoInput<Self::State>,
    ) -> Result<DynamicPolicyMonoResult<Self::State>> {
        tracing::debug!(
            "tag_answer::AnswerPolicy::mono\nState = {:?}\nreadonly = {}\n",
            inputs.state,
            inputs.readonly,
        );
        let (mut result, mut residual) =
            DynamicPolicyMonoResult::<Self::State>::from_inputs(inputs);
        match (residual.container, residual.state.status) {
            (Container::Tag(_) | Container::BeginAnchor(_, _), AnswerStatus::JustCreated) => {
                // Prepare the query
                residual.state.status = AnswerStatus::NeedProcessing;
                residual.state.reply = String::new();
                result.new_state = Some(residual.state);
                result.do_next_pass = true;
            }
            (Container::BeginAnchor(_, _), AnswerStatus::NeedProcessing) => {
                // Execute the model query
                let prompt = residual
                    .worker
                    .prefix_content_from_parameters(residual.input, residual.parameters)?;
                let prompt = residual
                    .worker
                    .postfix_content_from_parameters(prompt, residual.parameters)?;
                let prompt = Self::postfix_content_with_choice(
                    residual.worker,
                    prompt,
                    residual.parameters,
                )?;
                let (prompt, response) =
                    residual.worker.call_model(residual.parameters, &prompt)?;
                let response = Self::process_response_with_choice(response, residual.parameters)?;
                residual.state.query = prompt;
                residual.state.reply_hash = result.collector.normalized_hash(&response);
                residual.state.reply = response;
                residual.state.status = AnswerStatus::NeedInjection;
                residual.state.context_hash = residual.input_hash;
                result.new_state = Some(residual.state);
                result.do_next_pass = true;
            }
            (Container::BeginAnchor(_, _), AnswerStatus::NeedInjection) => {
                // Inject the reply into the document
                if !residual.readonly {
                    let output = residual.state.reply.clone();
                    residual.state.status = AnswerStatus::Completed;
                    result.new_state = Some(residual.state);
                    result.new_output = Some(output);
                }
                result.do_next_pass = true;
            }
            (Container::BeginAnchor(_, _), AnswerStatus::Completed) => {
                // Nothing to do
                let is_dynamic = residual
                    .parameters
                    .get("dynamic")
                    .map(|x| x.as_bool().unwrap_or(false))
                    .unwrap_or(false);
                if !is_dynamic {
                    // Do nothing
                } else if residual.state.context_hash != residual.input_hash {
                    // Repeat
                    residual.state.status = AnswerStatus::Repeat;
                    result.new_state = Some(residual.state);
                    result.do_next_pass = true;
                }
            }
            (Container::EndAnchor(a0, a1), AnswerStatus::Completed) => {
                if let Some(_) = residual.worker.is_output_redirected(&a0.parameters)? {
                    // No evaporation for redirected output anchors
                } else {
                    // Evaporate anchor
                    let content = Worker::get_range(
                        residual.document,
                        &Range {
                            begin: a0.range.end,
                            end: a1.range.begin,
                        },
                    )?;
                    let content_hash = result.collector.normalized_hash(&content);
                    if residual.state.reply_hash != content_hash {
                        // Content has been modified, evaporate anchor and let content become user content
                        if !residual.readonly {
                            residual.state.status = AnswerStatus::Edited;
                            result.new_state = Some(residual.state);
                            let mut a0 = a0.clone();
                            a0.parameters.insert("edited".into(), JsonPlusEntity::Flag);
                            result.new_patches = vec![residual.worker.mutate_anchor(&a0)?];
                        }
                        result.do_next_pass = true;
                    }
                }
            }
            (Container::BeginAnchor(a0, _), AnswerStatus::Repeat) => {
                // Return to need processing
                if !residual.readonly {
                    residual.state.status = AnswerStatus::NeedProcessing;
                    residual.state.reply = String::new();
                    result.new_state = Some(residual.state);
                    result.new_output = Some(String::new());
                    let mut a0 = a0.clone();
                    a0.parameters.remove("edited".into());
                    result.new_patches = vec![residual.worker.mutate_anchor(&a0)?];
                }
                result.do_next_pass = true;
            }
            (Container::BeginAnchor(a0, _), AnswerStatus::Edited) => {
                // Return to need processing
                if let Some(JsonPlusEntity::Flag) = a0.parameters.get("edited") {
                    // Edited preserved
                } else {
                    if !residual.readonly {
                        let mut a0 = a0.clone();
                        a0.parameters.insert("edited".into(), JsonPlusEntity::Flag);
                        result.new_patches = vec![residual.worker.mutate_anchor(&a0)?];
                    }
                    result.do_next_pass = true;
                }
            }
            _ => {}
        }
        Ok(result)
    }
}

impl AnswerPolicy {
    fn postfix_content_with_choice(
        worker: &Worker,
        content: ModelContent,
        parameters: &Parameters,
    ) -> Result<ModelContent> {
        let choices = match parameters.get("choose") {
            Some(JsonPlusEntity::Array(choices_list)) => {
                let mut choices = Vec::new();
                for choice in choices_list {
                    match choice {
                        JsonPlusEntity::Object(x) => {
                            return Err(ExecuteError::UnsupportedParameterValue(format!(
                                "bad choice {:?}",
                                x
                            )));
                        }
                        JsonPlusEntity::Array(x) => {
                            return Err(ExecuteError::UnsupportedParameterValue(format!(
                                "bad choice {:?}",
                                x
                            )));
                        }
                        x => {
                            choices.push(x.to_string());
                        }
                    }
                }
                Some(choices)
            }
            Some(JsonPlusEntity::Object(x)) => {
                let choices = x
                    .properties
                    .keys()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>();
                Some(choices)
            }
            _ => None,
        };
        match choices {
            Some(ref x) => {
                let choice_tags = x
                    .iter()
                    .map(|x| Self::choice_tag_from_choice(x))
                    .collect::<Vec<String>>();
                let handlebars = Handlebars::new();
                let json_choices = match choices {
                    Some(c) => serde_json::Value::Array(
                        c.iter().cloned().map(serde_json::Value::String).collect(),
                    ),
                    None => {
                        return Err(ExecuteError::UnsupportedParameterValue(
                            "no choice given".to_string(),
                        ));
                    }
                };
                let postfix = handlebars.render_template(
                    super::CHOICE_TEMPLATE,
                    &json!({ "choices": json_choices, "choice_tags": choice_tags }),
                )?;
                let postfix = ModelContentItem::system(&postfix);
                let postfix = ModelContent::from_item(postfix);
                Ok(worker.postfix_content(content, postfix))
            }
            None => Ok(content),
        }
    }
    fn process_response_with_choice(response: String, parameters: &Parameters) -> Result<String> {
        match parameters.get("choose") {
            Some(JsonPlusEntity::Object(x)) => {
                let choice_tags = x
                    .properties
                    .iter()
                    .filter_map(|(key, value)| {
                        match response.contains(&Self::choice_tag_from_choice(key)) {
                            true => Some(value.to_prompt()),
                            false => None,
                        }
                    })
                    .collect::<Vec<String>>();
                let response = match choice_tags.len() {
                    1 => choice_tags.get(0).expect("There is one element!"),
                    0 => super::NO_CHOICE_MESSAGE,
                    _ => super::MANY_CHOICES_MESSAGE,
                };
                Ok(format!("{}\n", response))
            }
            _ => Ok(response),
        }
    }
    fn choice_tag_from_choice(choice: &str) -> String {
        format!("§{}§", choice)
    }
}
