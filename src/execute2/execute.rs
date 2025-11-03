use crate::ast2::{
    Anchor, AnchorKind, CommandKind, Content, JsonPlusEntity, Parameters, Position, Range, Tag,
};
use crate::execute2::tags::TagBehaviorDispatch;
use crate::file::FileAccessor;
use crate::path::PathResolver;
use uuid::Uuid;

use crate::execute2::content::{ModelContent, ModelContentItem};
use crate::execute2::variables::{self, Variables};

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;

const REDIRECTED_OUTPUT_PLACEHOLDER : &str = "Context here has been answered but output has been redirected.\nAnyway do not respond anymore to context above this sentence.\n";

/// Executes a context and all its dependencies, processing all commands.
///
/// This function orchestrates the full, multi-pass execution of a context file.
/// It will resolve includes, execute model calls (`@answer`, `@derive`), and inject
/// the results back into the document by modifying the source files.
///
/// # Arguments
/// * `file_access` - A thread-safe file accessor.
/// * `path_res` - A thread-safe path resolver.
/// * `context_name` - The name of the root context to execute.
///
/// # Returns
/// The final, collected `ModelContent` after full execution.
pub fn execute_context(
    file_access: Arc<dyn FileAccessor>,
    path_res: Arc<dyn PathResolver>,
    context_name: &str,
) -> Result<ModelContent> {
    tracing::debug!("Executing context: {}", context_name);

    let exe = Worker::new(file_access, path_res);
    match exe.execute(Collector::new(), context_name, 77)? {
        Some(collector) => Ok(collector.context().clone()),
        None => Err(anyhow::anyhow!("Execute context returned no collector")),
    }
}

/// Collects a context's content without executing any commands that modify state or call models.
///
/// This function performs a single pass to gather all text content, resolving `@include`
/// tags, but it will not process `@answer`, `@derive`, or other state-changing anchors.
/// It is a 'read-only' version of `execute_context`.
///
/// # Arguments
/// * `file_access` - A thread-safe file accessor.
/// * `path_res` - A thread-safe path resolver.
/// * `context_name` - The name of the root context to collect.
///
/// # Returns
/// The collected `ModelContent`.
pub fn collect_context(
    file_access: Arc<dyn FileAccessor>,
    path_res: Arc<dyn PathResolver>,
    context_name: &str,
) -> Result<ModelContent> {
    tracing::debug!("Collecting context: {}", context_name);

    let exe = Worker::new(file_access, path_res);

    match exe.execute(Collector::new(), context_name, 0)? {
        Some(collector) => Ok(collector.context().clone()),
        None => Err(anyhow::anyhow!("Collect context returned no collector")),
    }
}

/// Central state object for the execution engine.
///
/// The `Collector` accumulates the final `ModelContent` and tracks execution-time
/// variables and the call stack to prevent infinite recursion.
/// It is passed by value through the execution flow, ensuring a functional-style,
/// predictable state management.
#[derive(Clone)]
pub(crate) struct Collector {
    /// A stack of visited context file paths to detect and prevent circular includes.
    visit_stack: Vec<PathBuf>,
    /// A stack of entered anchors in current context file
    anchor_stack: Vec<Anchor>,
    /// The accumulated content that will be sent to the model.
    context: ModelContent,
    /// Execution-time variables and settings.
    variables: Variables,
}

impl Collector {
    pub fn context(&self) -> &ModelContent {
        &self.context
    }

    pub fn anchor_stack(&self) -> &Vec<Anchor> {
        &self.anchor_stack
    }

    pub fn variables(&self) -> &Variables {
        &self.variables
    }

    /// Creates a new, empty `Collector`.
    ///
    /// # Arguments
    /// * `can_execute` - Sets the execution mode. `true` for full execution, `false` for collect-only.
    fn new() -> Self {
        Collector {
            visit_stack: Vec::new(),
            anchor_stack: Vec::new(),
            context: ModelContent::new(),
            variables: Variables::new(),
        }
    }

    /// Prepares the collector for descending into a new context file.
    ///
    /// It checks for circular dependencies. If the path is already in the `visit_stack`,
    /// it returns `None`. Otherwise, it returns a new `Collector` with the given path
    /// added to its stack and a cleared context, ready for the new file.
    fn descent(&self, context_path: &Path) -> Option<Self> {
        if self.visit_stack.contains(&context_path.to_path_buf()) {
            return None;
        }

        let mut visit_stack = self.visit_stack.clone();
        visit_stack.push(context_path.to_path_buf());
        Some(Collector {
            visit_stack,
            anchor_stack: Vec::new(),
            context: ModelContent::new(),
            variables: self.variables.clone(),
        })
    }

    // TODO doc (entra in anchor)
    fn enter(&self, anchor: &Anchor) -> Self {
        let mut collector = self.clone();
        collector.anchor_stack.push(anchor.clone());
        collector
    }

    // TODO doc (esci da anchor)
    fn exit(&self) -> Result<Self> {
        let mut collector = self.clone();
        collector
            .anchor_stack
            .pop()
            .ok_or_else(|| anyhow::anyhow!("Pop on empty stack!?"))?;
        Ok(collector)
    }

    // TODO doc
    pub fn update_variables(&self, new_variables: &Variables) -> Self {
        let mut collector = self.clone();
        collector.variables = new_variables.clone();
        collector
    }
}

/// The stateless engine that drives the context execution.
///
/// The `Worker` holds thread-safe handles to the tools needed for execution,
/// such as the file accessor and path resolver. It contains the core logic
/// for the multi-pass execution strategy.
pub(crate) struct Worker {
    file_access: Arc<dyn FileAccessor>,
    path_res: Arc<dyn PathResolver>,
}

impl Worker {
    /// Creates a new `Worker` with the necessary tools.
    fn new(file_access: Arc<dyn FileAccessor>, path_res: Arc<dyn PathResolver>) -> Self {
        Worker {
            file_access,
            path_res,
        }
    }

    /// Executes a context fully, running a loop of `execute_step` until the state converges.
    ///
    /// This is the main entry point for a full execution, which can modify files.
    /// It orchestrates the two-pass strategy until all anchors are `Completed`.
    pub fn execute(
        &self,
        collector: Collector,
        context_name: &str,
        max_rewrite_steps: usize,
    ) -> Result<Option<Collector>> {
        tracing::debug!("Worker::execute for context: {}", context_name);
        let context_path = self.path_res.resolve_context(context_name)?;

        match collector.descent(&context_path) {
            None => {
                tracing::debug!(
                    "execute::Worker::execute: Context {} already in visit stack, skipping execution.",
                    context_name
                );
                return Ok(Some(collector));
            }
            Some(collector) => {
                for i in 1..=max_rewrite_steps {
                    // Lock file, read it (could be edited outside), parse it, execute fast things that may modify context and save it
                    let (do_next_pass, collector_1) =
                        self.execute_pass(collector.clone(), &context_path)?;
                    match do_next_pass {
                        true => {
                            tracing::debug!(
                                "execute::Worker::execute: After {} modifying pass, needs another pass for context: {:?}",
                                i, context_path
                            );
                        }
                        false => {
                            return Ok(Some(collector_1));
                        }
                    };
                    // Re-read file, parse it, execute slow things that do not modify context, collect data
                    let (do_next_pass, collector_2) =
                        self.collect_pass(collector.clone(), &context_path)?;
                    match do_next_pass {
                        true => {
                            tracing::debug!(
                                "execute::Worker::execute: After {} readonly pass, needs another pass for context: {:?}",
                                i, context_path
                            );
                        }
                        false => {
                            return Ok(Some(collector_2));
                        }
                    };
                }
                // Last re-read file, parse it, collect data
                let (do_next_pass, collector) =
                    self.collect_pass(collector.clone(), &context_path)?;
                match do_next_pass {
                    true => {
                        tracing::debug!(
                            "execute::Worker::execute: After last readonly pass, needs another pass for context: {:?}",
                            context_path
                        );
                        return Ok(None);
                    }
                    false => {
                        // Successfully collected everything without needing further processing
                        tracing::debug!(
                            "execute::Worker::execute: Successfully collected context: {:?}",
                            context_path
                        );
                        return Ok(Some(collector));
                    }
                };
            }
        }
    }

    pub(crate) fn call_model(
        &self,
        variables: &Variables,
        contents: Vec<ModelContent>,
    ) -> Result<String> {
        /*/ let query = contents
        .into_iter()
        .flat_map(|mc| mc.0)
        .map(|item| item.to_string())
        .collect::<Vec<String>>()
        .join("\n"); */
        let mut prompt = String::new();
        prompt.push_str(&variables.system.clone().unwrap_or(String::new()));
        prompt.push_str(&self.modelcontent_to_string(&contents)?);
        crate::agent::shell::shell_call(&variables.provider, &prompt)
    }

    fn collect_pass(&self, collector: Collector, context_path: &Path) -> Result<(bool, Collector)> {
        self._pass_internal(collector, context_path, true)
    }

    fn apply_patches(context_content: &str, patches: &Vec<(Range, String)>) -> Result<String> {
        let mut result = context_content.to_string();
        for (range, replace) in patches.iter().rev() {
            let start_byte = context_content
                .char_indices()
                .nth(range.begin.offset)
                .map(|(i, _)| i)
                .unwrap_or(context_content.len());
            let end_byte = context_content
                .char_indices()
                .nth(range.end.offset)
                .map(|(i, _)| i)
                .unwrap_or(context_content.len());
            result.replace_range(start_byte..end_byte, replace);
        }
        Ok(result)
    }

    fn execute_pass(&self, collector: Collector, context_path: &Path) -> Result<(bool, Collector)> {
        let _lock = crate::file::FileLock::new(self.file_access.clone(), context_path)?;
        self._pass_internal(collector, context_path, false)
    }

    fn _pass_internal(
        &self,
        mut collector: Collector,
        context_path: &Path,
        is_collect: bool,
    ) -> Result<(bool, Collector)> {
        let context_content = self.file_access.read_file(context_path)?;
        let ast = crate::ast2::parse_document(&context_content)?;
        let anchor_index = super::AnchorIndex::new(&ast.content);

        for item in &ast.content {
            let (do_next_pass, next_collector, patches) = match item {
                Content::Text(text) => {
                    collector
                        .context
                        .push(ModelContentItem::user(&text.content));
                    (false, collector, vec![])
                }
                Content::Tag(tag) => {
                    let local_variables = self.update_variables(&collector.variables(), &tag.parameters)?;
                    if is_collect {
                        let (do_next_pass, collector) =
                            TagBehaviorDispatch::collect_tag(self, collector, &local_variables, tag)?;
                        (do_next_pass, collector, vec![])
                    } else {
                        TagBehaviorDispatch::execute_tag(self, collector, &local_variables, tag)?
                    }
                }
                Content::Anchor(anchor) => match anchor.kind {
                    AnchorKind::Begin => {
                        let local_variables = self.update_variables(&collector.variables(), &anchor.parameters)?;
                        let anchor_end = anchor_index
                            .get_end(&anchor.uuid)
                            .ok_or(anyhow::anyhow!("end anchor not found"))?;
                        let anchor_end = ast
                            .content
                            .get(anchor_end)
                            .and_then(|c| match c {
                                Content::Anchor(a) => Some(a),
                                _ => None,
                            })
                            .ok_or(anyhow::anyhow!("end anchor not found"))?;
                        let (do_next_pass, new_collector, patches) = if is_collect {
                            let (do_next_pass, collector) = TagBehaviorDispatch::collect_anchor(
                                self,
                                collector,
                                &local_variables,
                                anchor,
                                anchor_end.range.begin,
                            )?;
                            (do_next_pass, collector, vec![])
                        } else {
                            TagBehaviorDispatch::execute_anchor(
                                self,
                                collector,
                                &local_variables,
                                anchor,
                                anchor_end.range.begin,
                            )?
                        };
                        (do_next_pass, new_collector.enter(anchor), patches)
                    }
                    AnchorKind::End => (false, collector.exit()?, vec![]),
                },
            };
            collector = next_collector;

            // Evaluate patches
            if patches.is_empty() {
                // No patches to apply
            } else if is_collect {
                // This is collect pass, cannot produce patches, it's a bug!
                panic!("Cannot produce patches during collect pass!");
            } else {
                // Apply patches and trigger new pass
                let new_content = Self::apply_patches(&context_content, &patches)?;
                self.file_access
                    .write_file(context_path, &new_content, None)?;
                return Ok((true, collector));
            }
            // Check if collector has been discarded, then exit and trigger another pass
            if do_next_pass {
                return Ok((true, collector));
            }
        }
        // No patches applied nor new pass triggered, then return definitive collector
        Ok((false, collector))
    }

    fn get_state_path(&self, command: CommandKind, uuid: &Uuid) -> Result<PathBuf> {
        let meta_path = self
            .path_res
            .resolve_metadata(&command.to_string(), &uuid)?;
        let state_path = meta_path.join("state.json");
        Ok(state_path)
    }
    pub fn load_state<T: serde::de::DeserializeOwned>(
        &self,
        command: CommandKind,
        uuid: &Uuid,
    ) -> Result<T> {
        let state_path = self.get_state_path(command, uuid)?;
        let state = self.file_access.read_file(&state_path)?;
        let state: T = serde_json::from_str(&state)?;
        Ok(state)
    }
    pub fn save_state<T: serde::Serialize>(
        &self,
        command: CommandKind,
        uuid: &Uuid,
        state: &T,
        comment: Option<&str>,
    ) -> Result<()> {
        let state_path = self.get_state_path(command, uuid)?;
        let state_str = serde_json::to_string_pretty(state)?;
        self.file_access
            .write_file(&state_path, &state_str, comment)?;
        Ok(())
    }

    pub fn tag_to_anchor(
        &self,
        collector: &Collector,
        local_variables: &Variables,
        tag: &Tag,
        output: &str,
    ) -> Result<(Uuid, Vec<(Range, String)>)> {
        let (a0, a1) = Anchor::new_couple(tag.command, &tag.parameters, &tag.arguments);
        match self.redirect_output(local_variables, output)? {
            true => {
                // Output redirected, just convert tag into anchor
                Ok((
                    a0.uuid,
                    vec![(
                        tag.range,
                        format!("{}\n{}{}\n", a0.to_string(), REDIRECTED_OUTPUT_PLACEHOLDER, a1.to_string()),
                    )],
                ))
            }
            false => {
                // Output not redirected, include in new anchors
                Ok((
                    a0.uuid,
                    vec![(
                        tag.range,
                        format!("{}\n{}{}\n", a0.to_string(), output, a1.to_string()),
                    )],
                ))
            }
        }
    }

    pub fn inject_into_anchor(
        &self,
        collector: &Collector,
        local_variables: &Variables,
        anchor: &Anchor,
        anchor_end: &Position,
        output: &str,
    ) -> Result<Vec<(Range, String)>> {
        match self.redirect_output(local_variables, output)? {
            true => {
                // Output redirected, delete anchor contents
                Ok(vec![(
                    Range {
                        begin: anchor.range.end,
                        end: *anchor_end,
                    },
                    REDIRECTED_OUTPUT_PLACEHOLDER.to_string(),
                )])
            }
            false => {
                // Output not redirected, patch anchor contents
                Ok(vec![(
                    Range {
                        begin: anchor.range.end,
                        end: *anchor_end,
                    },
                    output.to_string(),
                )])
            }
        }
    }

    fn redirect_output(&self, local_variables: &Variables, output: &str) -> Result<bool> {        
        match &local_variables.output {
            Some(x) => {
                tracing::debug!("Output redirection to {}\n", &x);
                let output_path = self.path_res.resolve_context(&x)?;
                self.file_access.write_file(&output_path, output, None)?;
                return Ok(true);
            }
            None => {
                return Ok(false);
            }
        }
    }

    pub fn mutate_anchor(&self, anchor: &Anchor) -> Result<Vec<(Range, String)>> {
        Ok(vec![(anchor.range, format!("{}\n", anchor.to_string()))])
    }

    pub fn update_variables(
        &self,
        variables: &Variables,
        parameters: &Parameters,
    ) -> Result<Variables> {
        let mut new_variables = variables.clone();
        match parameters.get("provider") {
            Some(
                JsonPlusEntity::DoubleQuotedString(x)
                | JsonPlusEntity::SingleQuotedString(x)
                | JsonPlusEntity::NudeString(x),
            ) => {
                new_variables.provider = x.clone();
            }
            _ => {}
        };
        match parameters.get("output") {
            Some(
                JsonPlusEntity::DoubleQuotedString(x)
                | JsonPlusEntity::SingleQuotedString(x)
                | JsonPlusEntity::NudeString(x),
            ) => {
                tracing::debug!("output = {}", &x);
                new_variables.output = Some(x.clone());
            }
            _ => {}
        }
        match parameters.get("system") {
            Some(JsonPlusEntity::NudeString(x)) => match self.execute(Collector::new(), x, 0)? {
                Some(x) => {
                    new_variables.system =
                        Some(self.modelcontent_to_string(&vec![x.context().clone()])?);
                }
                None => {
                    return Err(anyhow::anyhow!("Failed to collect system contest {}", x));
                }
            },
            _ => {}
        }
        Ok(new_variables)
    }

    fn modelcontent_to_string(&self, content: &Vec<ModelContent>) -> Result<String> {
        Ok(content
            .into_iter()
            .flat_map(|mc| mc.0.clone())
            .map(|item| item.to_string())
            .collect::<Vec<String>>()
            .join("\n"))
    }
}
