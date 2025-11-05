use crate::ast2::{
    Anchor, AnchorKind, CommandKind, Content, JsonPlusEntity, Parameters, Position, Range, Tag,
};
use crate::execute2::tags::TagBehaviorDispatch;
use crate::file::FileAccessor;
use crate::path::PathResolver;
use uuid::Uuid;

use crate::execute2::content::{ModelContent, ModelContentItem};
//use crate::execute2::variables::{self, Variables};

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;

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
    exe.execute(context_name)
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

    exe.collect(context_name)
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
    ///variables: Variables,
    /// Execution-time default parameters for tags
    default_parameters: Parameters,
    /// Latest processed range
    latest_range: Range,
}

impl Collector {
    pub fn context(&self) -> &ModelContent {
        &self.context
    }

    pub fn anchor_stack(&self) -> &Vec<Anchor> {
        &self.anchor_stack
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
            default_parameters: Parameters::new(),
            latest_range: Range::null(),
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
            context: self.context.clone(),
            default_parameters: self.default_parameters.clone(),
            latest_range: Range::null(),
        })
    }

    pub fn forget(mut self) -> Self {
        self.context = ModelContent::new();
        self 
    }

    fn enter(mut self, anchor: &Anchor) -> Self {
        self.anchor_stack.push(anchor.clone());
        self 
    }

    fn exit(mut self) -> Result<Self> {
        self 
            .anchor_stack
            .pop()
            .ok_or_else(|| anyhow::anyhow!("Pop on empty stack at {:?}", self.latest_range))?;
        Ok(self)
    }

    pub fn push_item(mut self, item: ModelContentItem) -> Self {
        self.context.push(item);
        self
    }

    pub fn set_latest_range(&mut self, range: &Range) {
        self.latest_range = range.clone();
    }

    pub fn set_default_parameters(mut self, new_parameters: &Parameters) -> Self {
        self.default_parameters = new_parameters.clone();
        self
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

    pub fn execute(&self, context_name: &str) -> Result<ModelContent> {
        match self._execute(Collector::new(), context_name, 100)? {
            Some(collector) => {
                return Ok(collector.context().clone());
            }
            None => {
                return Err(anyhow::anyhow!(
                    "Could not execute context {}",
                    context_name
                ));
            }
        }
    }

    pub fn collect(&self, context_name: &str) -> Result<ModelContent> {
        match self._execute(Collector::new(), context_name, 0)? {
            Some(collector) => {
                return Ok(collector.context().clone());
            }
            None => {
                return Err(anyhow::anyhow!(
                    "Could not collect context {}",
                    context_name
                ));
            }
        }
    }

    /// Executes a context fully, running a loop of `execute_step` until the state converges.
    ///
    /// This is the main entry point for a full execution, which can modify files.
    /// It orchestrates the two-pass strategy until all anchors are `Completed`.
    pub fn _execute(
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
                            // Ready for final collect pass
                            break;
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
        parameters: &Parameters,
        content: &ModelContent,
    ) -> Result<String> {
        let mut prompt = ModelContent::new();
        match parameters.parameters.properties.get("prefix") {
            Some(JsonPlusEntity::NudeString(x)) => {
                prompt.push(ModelContentItem::system(&self.execute(x)?.to_string()));
            }
            Some(x) => {
                return Err(anyhow::anyhow!("Bad prefix"));
            }
            None => {}
        }
        prompt.extend(content.clone());
        match parameters.parameters.properties.get("postfix") {
            Some(JsonPlusEntity::NudeString(x)) => {
                prompt.push(ModelContentItem::system(&self.execute(x)?.to_string()));
            }
            Some(x) => {
                return Err(anyhow::anyhow!("Bad postfix"));
            }
            None => {}
        }
        let provider = match parameters.parameters.properties.get("provider") {
            Some(
                JsonPlusEntity::NudeString(x)
                | JsonPlusEntity::SingleQuotedString(x)
                | JsonPlusEntity::DoubleQuotedString(x),
            ) => x,
            Some(x) => {
                return Err(anyhow::anyhow!("Bad provider"));
            }
            None => {
                return Err(anyhow::anyhow!("No provider"));
            }
        };
        crate::agent::shell::shell_call(&provider, &prompt.to_prompt()) // to_string())
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
                    collector.set_latest_range(&text.range);
                    let content = if collector.anchor_stack.is_empty() {
                        // User writes outside anchors
                        ModelContentItem::user(&text.content)
                    } else {
                        // Agents write inside anchors
                        ModelContentItem::agent(&text.content)
                    };
                    collector.context.push(content);
                    (false, collector, vec![])
                }
                Content::Tag(tag) => {
                    collector.set_latest_range(&tag.range);
                    let integrated_tag = tag.clone().integrate(&collector.default_parameters);
                    if is_collect {
                        let (do_next_pass, collector) =
                            TagBehaviorDispatch::collect_tag(self, collector, &integrated_tag)?;
                        (do_next_pass, collector, vec![])
                    } else {
                        TagBehaviorDispatch::execute_tag(self, collector, &integrated_tag)?
                    }
                }
                Content::Anchor(anchor) => match anchor.kind {
                    AnchorKind::Begin => {
                        collector.set_latest_range(&anchor.range);
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
                                anchor,
                                anchor_end.range.begin,
                            )?;
                            (do_next_pass, collector, vec![])
                        } else {
                            TagBehaviorDispatch::execute_anchor(
                                self,
                                collector,
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
        tag: &Tag,
        output: &str,
    ) -> Result<(Uuid, Vec<(Range, String)>)> {
        let (a0, a1) = Anchor::new_couple(tag.command, &tag.parameters, &tag.arguments);
        match self.redirect_output(&tag.parameters, output)? {
            true => {
                // Output redirected, just convert tag into anchor
                Ok((
                    a0.uuid,
                    vec![(
                        tag.range,
                        format!("{}\n{}\n", a0.to_string(), a1.to_string()),
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
        anchor: &Anchor,
        anchor_end: &Position,
        output: &str,
    ) -> Result<Vec<(Range, String)>> {
        match self.redirect_output(&anchor.parameters, output)? {
            true => {
                // Output redirected, delete anchor contents
                Ok(vec![(
                    Range {
                        begin: anchor.range.end,
                        end: *anchor_end,
                    },
                    String::new(),
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

    pub fn is_output_redirected(&self, parameters: &Parameters) -> Result<Option<PathBuf>> {
        match &parameters.parameters.properties.get("output") {
            Some(JsonPlusEntity::NudeString(x)) => {
                let output_path = self.path_res.resolve_context(&x)?;
                return Ok(Some(output_path));
            }
            Some(x) => {
                return Err(anyhow::anyhow!("Unsupported output {:?}", x));
            }
            None => {
                return Ok(None);
            }
        }
    }

    fn redirect_output(&self, parameters: &Parameters, output: &str) -> Result<bool> {
        match self.is_output_redirected(parameters)? {
            Some(output_path) => {
                self.file_access.write_file(&output_path, output, None)?;
                return Ok(true);
            }
            None => {
                return Ok(false);
            }
        }
    }

    pub fn redirect_input(
        &self,
        parameters: &Parameters,
        input: ModelContent,
    ) -> Result<ModelContent> {
        match &parameters.parameters.properties.get("input") {
            Some(JsonPlusEntity::NudeString(x)) => {
                let output_path = self.path_res.resolve_context(&x)?;
                self.execute(&x)
            }
            Some(x) => {
                return Err(anyhow::anyhow!("Unsupported input {:?}", x));
            }
            None => {
                return Ok(input);
            }
        }
    }

    pub fn mutate_anchor(&self, anchor: &Anchor) -> Result<Vec<(Range, String)>> {
        Ok(vec![(anchor.range, format!("{}\n", anchor.to_string()))])
    }
}
