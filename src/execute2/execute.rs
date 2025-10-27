use crate::ast2::{Anchor, AnchorKind, Arguments, CommandKind, Content, Document, Parameters, Range, Tag, Text,
    Position, Uuid as AstUuid};
use crate::file::FileAccessor;
use crate::path::PathResolver;
use super::*;
use crate::execute2::tags::TagBehaviorDispatch;

use crate::execute2::content::{ModelContent, ModelContentItem};
use crate::execute2::variables::Variables;

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
    file_access: Arc<dyn file::FileAccessor>,
    path_res: Arc<dyn path::PathResolver>,
    context_name: &str,
) -> Result<ModelContent> {
    tracing::debug!("Executing context: {}", context_name);

    let exe = Worker::new(file_access, path_res);
    let collector = exe.execute(Collector::new(true), context_name)?;

    tracing::debug!("Finished executing context: {}", context_name);
    Ok(collector.context)
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
    file_access: Arc<dyn file::FileAccessor>,
    path_res: Arc<dyn path::PathResolver>,
    context_name: &str,
) -> Result<ModelContent> {
    tracing::debug!("Collecting context: {}", context_name);

    let exe = Worker::new(file_access, path_res);
    let collector = exe.collect(Collector::new(false), context_name)?;

    tracing::debug!("Finished collecting context: {}", context_name);
    Ok(collector.context)
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
    /// A flag indicating whether the engine is in `execute` (true) or `collect` (false) mode.
    can_execute: bool,
}

impl Collector {
    /// Creates a new, empty `Collector`.
    ///
    /// # Arguments
    /// * `can_execute` - Sets the execution mode. `true` for full execution, `false` for collect-only.
    fn new(can_execute: bool) -> Self {
        Collector {
            visit_stack: Vec::new(),
            anchor_stack: Vec::new(),
            context: ModelContent::new(),
            variables: Variables::new(),
            can_execute,
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
            can_execute: self.can_execute,
        })
    }

    /// Creates a new `Collector` with updated variables values, taken from Parameters
    ///
    /// # Arguments
    /// * `parameters` - Parameters to take variable values from
    fn update(&self, parameters: &Parameters) -> Self {
        let mut collector = self.clone();
        collector.variables = collector.variables.update(parameters);
        collector
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
}

/// The stateless engine that drives the context execution.
///
/// The `Worker` holds thread-safe handles to the tools needed for execution,
/// such as the file accessor and path resolver. It contains the core logic
/// for the multi-pass execution strategy.
pub(crate) struct Worker {
    file_access: Arc<dyn file::FileAccessor>,
    path_res: Arc<dyn path::PathResolver>,
}

impl Worker {
    /// Creates a new `Worker` with the necessary tools.
    fn new(
        file_access: Arc<dyn file::FileAccessor>,
        path_res: Arc<dyn path::PathResolver>,
    ) -> Self {
        Worker {
            file_access,
            path_res,
        }
    }

    /// Collects context from a file, resolving includes but not executing stateful anchors.
    /// This represents the entry point for a read-only collection pass.
    fn collect(&self, collector: Collector, context_name: &str) -> Result<Collector> {
        /* TODO
        tracing::debug!("Worker::collect for context: {}", context_name);
        let context_path = self.path_res.resolve_context(context_name)?;

        match collector.descent(&context_path) {
            None => {
                tracing::debug!(
                    "Context {} already in visit stack, skipping collection.",
                    context_name
                );
                return Ok(collector);
            }
            Some(collector) => {
                // Read file, parse it, collect without allowing execution
                match self.pass_1(collector, &context_path)? {
                    Some(collector) => {
                        // Successfully collected everything without needing further processing
                        tracing::debug!(
                            "Worker::execute_step collected from pass_1 for path: {:?}",
                            context_path
                        );
                        return Ok(collector);
                    }
                    None => {
                        return Err(anyhow::anyhow!(
                            "Collection incomplete, execution needed for context: {}",
                            context_name
                        ));
                    }
                };
            }
        }
        */
        unimplemented!()
    }

    /// Executes a context fully, running a loop of `execute_step` until the state converges.
    ///
    /// This is the main entry point for a full execution, which can modify files.
    /// It orchestrates the two-pass strategy until all anchors are `Completed`.
    fn execute(&self, collector: Collector, context_name: &str) -> Result<Collector> {
        /* TODO
        tracing::debug!("Worker::execute for context: {}", context_name);
        let context_path = self.path_res.resolve_context(context_name)?;

        match collector.descent(&context_path) {
            None => {
                tracing::debug!(
                    "Context {} already in visit stack, skipping execution.",
                    context_name
                );
                return Ok(collector);
            }
            Some(collector) => {
                let mut collector = collector;
                loop {
                    if let Some(new_collector) =
                        self.execute_step(collector.clone(), &context_path)?
                    {
                        tracing::debug!(
                            "Worker::execute returning collected for context: {}",
                            context_name
                        );
                        return Ok(new_collector);
                    }
                    tracing::debug!("Worker::execute continuing for context: {}", context_name);
                }
            }
        }*/
        unimplemented!()
    }

    /// Executes a single step of the two-pass strategy.
    ///
    /// 1.  **Pass 2**: Scans the document for anchors that are ready for injection (`NeedInjection`)
    ///     and patches the file on disk. This is a fast, synchronous operation.
    /// 2.  **Pass 1**: Re-reads the document and collects content. If it encounters anchors
    ///     that need processing (`JustCreated` or `NeedProcessing`), it triggers the slow
    ///     tasks (like model calls) and returns `Ok(None)` to signal that the `execute`
    ///     loop needs to run again.
    ///
    /// If `pass_1` completes without starting any new tasks, it returns `Ok(Some(Collector))`,
    /// signaling that the execution has converged and is complete.
    fn execute_step(&self, collector: Collector, context_path: &Path) -> Result<Option<Collector>> {
        /* TODO
        tracing::debug!("Worker::execute_step for path: {:?}", context_path);

        // Lock file, read it (could be edited outside), parse it, execute fast things that may modify context and save it
        match self.pass_2(context_path)? {
            true => {
                tracing::debug!(
                    "Worker::execute_step pass_2 needs another pass for path: {:?}",
                    context_path
                );
                return Ok(None);
            }
            false => {
                tracing::debug!(
                    "Worker::execute_step no changes in pass_2 for path: {:?}",
                    context_path
                );
            }
        }

        // Re-read file, parse it, execute slow things that do not modify context, collect data
        match self.pass_1(collector, context_path)? {
            Some(collector) => {
                // Successfully collected everything without needing further processing
                tracing::debug!(
                    "Worker::execute_step collected from pass_1 for path: {:?}",
                    context_path
                );
                return Ok(Some(collector));
            }
            None => {
                // Could not collect everything in pass_1, need to trigger another step
                tracing::debug!(
                    "Worker::execute_step could not collect from pass_1 for path: {:?}",
                    context_path
                );
                return Ok(None);
            }
        };
        */
        unimplemented!()
    }

    fn call_model(&self, collector: &Collector, contents: Vec<ModelContent>) -> Result<String> {
        let query = contents
            .into_iter()
            .flat_map(|mc| mc.0)
            .map(|item| item.to_string())
            .collect::<Vec<String>>()
            .join("\n");
        agent::shell::shell_call(&collector.variables.provider, &query)
    }

    fn collect_pass(&self, collector: Collector, context_path: &Path) -> Result<Option<Collector>> {
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

    fn execute_pass(&self, collector: Collector, context_path: &Path) -> Result<Option<Collector>> {
        let _lock = crate::file::FileLock::new(self.file_access.clone(), context_path)?;
        self._pass_internal(collector, context_path, false)
    }

    fn _pass_internal(
        &self,
        mut collector: Collector,
        context_path: &Path,
        is_collect: bool,
    ) -> Result<Option<Collector>> {
        let context_content = self.file_access.read_file(context_path)?;
        let ast = crate::ast2::parse_document(&context_content)?;
        let anchor_index = utils::AnchorIndex::new(&ast.content);

        for item in &ast.content {
            let (maybe_new_collector, patches) = match item {
                Content::Text(text) => {
                    collector
                        .context
                        .push(ModelContentItem::user(&text.content));
                    (collector, vec![])
                }
                Content::Tag(tag) => {
                    if is_collect {
                        TagBehaviorDispatch::collect_tag(self, collector, tag)?
                    } else {
                        TagBehaviorDispatch::execute_tag(self, collector, tag)?
                    }
                }
                Content::Anchor(anchor) => match anchor.kind {
                    AnchorKind::Begin => {
                        let anchor_end = anchor_index.get_end(anchor.uuid)?;
                        if is_collect {
                            TagBehaviorDispatch::collect_anchor(
                                self, collector, anchor, anchor_end,
                            )?
                        } else {
                            TagBehaviorDispatch::execute_anchor(
                                self, collector, anchor, anchor_end,
                            )?
                        }
                        collector.enter(anchor);
                    }
                    AnchorKind::End => {
                        collector.exit()?;
                    }
                },
            };
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
                return Ok(None);
            }
            // Check if collector has been discarded, then exit and trigger another pass
            match maybe_new_collector {
                None => {
                    return Ok(None);
                }
                Some(new_collector) => {
                    collector = new_collector;
                }
            }
        }
        // No patches applied nor new pass triggered, then return definitive collector
        Ok(Some(collector))
    }
}
