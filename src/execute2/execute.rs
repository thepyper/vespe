//! This module contains the core logic for the execution engine, orchestrating the
//! processing of documents containing directives (tags and anchors). It defines
//! the `Worker` responsible for driving the execution and the `Collector` for
//! managing the execution state and accumulating results. The module implements
//! a multi-pass execution strategy to handle dynamic content generation and
//! modification.
use crate::ast2::{
    Anchor, AnchorKind, CommandKind, Content, JsonPlusEntity, Parameters, Position, Range, Tag,
};
use crate::execute2::tags::TagBehaviorDispatch;
use crate::file::FileAccessor;
use crate::path::PathResolver;
use uuid::Uuid;

use crate::execute2::content::{ModelContent, ModelContentItem};

use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::{ExecuteError, Result};

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
    /// Execution-time default parameters for tags
    default_parameters: Parameters,
    /// Latest processed range
    latest_range: Range,
}

impl Collector {
    /// Returns a reference to the accumulated `ModelContent`.
    ///
    /// This content represents the current state of the prompt being built
    /// for the external model.
    ///
    /// # Returns
    ///
    /// A reference to the [`ModelContent`].
    pub fn context(&self) -> &ModelContent {
        &self.context
    }

    /// Returns a reference to the current anchor stack.
    ///
    /// The anchor stack keeps track of nested anchors, which is crucial for
    /// correctly processing content within dynamic tags.
    ///
    /// # Returns
    ///
    /// A reference to a `Vec<Anchor>` representing the current anchor stack.
    pub fn anchor_stack(&self) -> &Vec<Anchor> {
        &self.anchor_stack
    }

    /// Creates a new, empty `Collector` instance.
    ///
    /// Initializes all internal stacks and the `ModelContent` as empty.
    ///
    /// # Returns
    ///
    /// A new `Collector`.
    pub fn new() -> Self {
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
    ///
    /// # Arguments
    ///
    /// * `context_path` - The path to the new context file to descend into.
    ///
    /// # Returns
    ///
    /// An `Option<Self>`: `Some(Collector)` if the descent is valid and a new
    /// collector is created, `None` if a circular dependency is detected.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use std::path::{Path, PathBuf};
    /// # use crate::execute2::Collector;
    /// let collector = Collector::new();
    /// let path = PathBuf::from("test_context.md");
    /// let new_collector = collector.descent(&path).unwrap();
    /// assert_eq!(new_collector.visit_stack.len(), 1);
    /// ```
    fn descent(&self, context_path: &Path) -> Option<Self> {
        // Check if context has already been visited
        if self.visit_stack.contains(&context_path.to_path_buf()) {
            return None;
        }
        // Create descending collector
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

    /// Ascends from a nested context, merging the state from the descended collector.
    ///
    /// This method takes the `Collector` that was used for a sub-context (the `descent_collector`)
    /// and merges its accumulated `context` and `default_parameters` back into the
    /// current collector. The `visit_stack` is implicitly handled by the `descent` method.
    ///
    /// # Arguments
    ///
    /// * `descent_collector` - The `Collector` instance that was used for the nested context.
    ///
    /// # Returns
    ///
    /// The updated `Collector` with merged state.
    fn ascend(mut self, descent_collector: Collector) -> Self {
        // Merge descending collector context
        self.context = descent_collector.context;
        self.default_parameters = descent_collector.default_parameters;
        self
    }

    /// Resets the accumulated `ModelContent` of the collector.
    ///
    /// This is typically used by tags like `@forget` to clear the current prompt
    /// content, effectively starting a new conversation segment for the model.
    ///
    /// # Returns
    ///
    /// The `Collector` with its `context` reset.
    pub fn forget(mut self) -> Self {
        self.context = ModelContent::new();
        self
    }

    /// Enters a new anchor, pushing it onto the anchor stack.
    ///
    /// This is used to track the current nesting level of anchors within a document,
    /// which is important for determining how text content should be interpreted
    /// (e.g., user-written vs. agent-generated).
    ///
    /// # Arguments
    ///
    /// * `anchor` - The [`Anchor`] to enter.
    ///
    /// # Returns
    ///
    /// The `Collector` with the new anchor added to its stack.
    fn enter(mut self, anchor: &Anchor) -> Self {
        self.anchor_stack.push(anchor.clone());
        self
    }

    /// Exits the current anchor, popping it from the anchor stack.
    ///
    /// # Returns
    ///
    /// A `Result<Self>`: `Ok(Collector)` if an anchor was successfully popped,
    /// or an `Err(ExecuteError::EmptyAnchorStack)` if the stack was already empty.
    ///
    /// # Errors
    ///
    /// Returns [`ExecuteError::EmptyAnchorStack`] if there are no anchors to exit.
    fn exit(mut self) -> Result<Self> {
        self.anchor_stack
            .pop()
            .ok_or_else(|| ExecuteError::EmptyAnchorStack(self.latest_range.clone()))?;
        Ok(self)
    }

    /// Pushes a single `ModelContentItem` onto the accumulated `ModelContent`.
    ///
    /// This is the primary way to add text segments (system, user, or agent messages)
    /// to the prompt being constructed.
    ///
    /// # Arguments
    ///
    /// * `item` - The [`ModelContentItem`] to add.
    ///
    /// # Returns
    ///
    /// The `Collector` with the new item added to its context.
    pub fn push_item(mut self, item: ModelContentItem) -> Self {
        self.context.push(item);
        self
    }

    /// Sets the latest processed [`Range`] in the document.
    ///
    /// This is used for error reporting and to provide context for certain operations.
    ///
    /// # Arguments
    ///
    /// * `range` - The new [`Range`] to set.
    ///
    /// # Returns
    ///
    /// The `Collector` with the updated latest range.
    pub fn set_latest_range(mut self, range: &Range) -> Self {
        self.latest_range = range.clone();
        self
    }

    /// Sets the default parameters for subsequent tags.
    ///
    /// Tags can inherit or override these default parameters. This is useful for
    /// applying global settings or context-specific configurations.
    ///
    /// # Arguments
    ///
    /// * `new_parameters` - The new [`Parameters`] to use as defaults.
    ///
    /// # Returns
    ///
    /// The `Collector` with the updated default parameters.
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
    ///
    /// # Arguments
    ///
    /// * `file_access` - A shared reference to an object implementing `FileAccessor`,
    ///                   used for all file system operations.
    /// * `path_res` - A shared reference to an object implementing `PathResolver`,
    ///                used for resolving context and metadata paths.
    ///
    /// # Returns
    ///
    /// A new `Worker` instance.
    fn new(file_access: Arc<dyn FileAccessor>, path_res: Arc<dyn PathResolver>) -> Self {
        Worker {
            file_access,
            path_res,
        }
    }

    /// Executes a context fully, running a loop of passes until the state converges.
    ///
    /// This is the main entry point for a full execution, which can modify files.
    /// It orchestrates the multi-pass strategy until all dynamic anchors are `Completed`
    /// or a maximum number of rewrite steps is reached.
    ///
    /// # Arguments
    ///
    /// * `context_name` - The name of the root context to execute.
    ///
    /// # Returns
    ///
    /// A `Result` containing the final, collected [`ModelContent`] after full execution,
    /// or an [`ExecuteError`] if the context cannot be found or an error occurs during execution.
    ///
    /// # Errors
    ///
    /// Returns [`ExecuteError::ContextNotFound`] if the specified context cannot be resolved.
    /// Returns other [`ExecuteError`] variants for various execution failures.
    pub fn execute(&self, context_name: &str) -> Result<ModelContent> {
        match self._execute(Collector::new(), context_name, 100)? {
            Some(collector) => {
                return Ok(collector.context().clone());
            }
            None => {
                return Err(ExecuteError::ContextNotFound(context_name.to_string()));
            }
        }
    }

    /// Collects a context's content without executing any commands that modify state or call models.
    ///
    /// This function performs a read-only execution pass to gather all text content,
    /// resolving `@include` tags and processing existing anchors, but it will not
    /// process `@answer`, `@derive`, or other state-changing operations that would
    /// modify the source files or call external models.
    ///
    /// # Arguments
    ///
    /// * `context_name` - The name of the root context to collect.
    ///
    /// # Returns
    ///
    /// A `Result` containing the collected [`ModelContent`], or an [`ExecuteError`]
    /// if the context cannot be found or an error occurs during collection.
    ///
    /// # Errors
    ///
    /// Returns [`ExecuteError::ContextNotFound`] if the specified context cannot be resolved.
    /// Returns other [`ExecuteError`] variants for various collection failures.
    pub fn collect(&self, context_name: &str) -> Result<ModelContent> {
        match self._execute(Collector::new(), context_name, 0)? {
            Some(collector) => {
                return Ok(collector.context().clone());
            }
            None => {
                return Err(ExecuteError::ContextNotFound(context_name.to_string()));
            }
        }
    }

    /// Internal execution loop for processing a context, potentially over multiple passes.
    ///
    /// This function drives the multi-pass execution. It repeatedly calls `execute_pass`
    /// (for modifying operations) and `collect_pass` (for read-only collection) until
    /// the document state converges (no more patches are generated) or `max_rewrite_steps`
    /// is reached. The `readonly` flag within `_pass_internal` determines whether
    /// file modifications and model calls are allowed.
    ///
    /// # Arguments
    ///
    /// * `collector` - The initial [`Collector`] state.
    /// * `context_name` - The name of the context to execute.
    /// * `max_rewrite_steps` - The maximum number of rewrite passes to attempt.
    ///                         A value of `0` implies a single read-only pass (collection).
    ///
    /// # Returns
    ///
    /// A `Result` containing `Some(Collector)` if the execution completes successfully
    /// and a final collector state is available, or `None` if the context is already
    /// in the visit stack (circular dependency) or if `max_rewrite_steps` is exceeded
    /// without convergence.
    ///
    /// # Errors
    ///
    /// Returns [`ExecuteError::ContextNotFound`] if the context path cannot be resolved.
    /// Returns other [`ExecuteError`] variants for various execution failures.
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
            Some(descent_collector) => {
                for i in 1..=max_rewrite_steps {
                    // Lock file, read it (could be edited outside), parse it, execute fast things that may modify context and save it
                    let (do_next_pass, _) =
                        self.execute_pass(descent_collector.clone(), &context_path)?;
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
                    let (do_next_pass, _) =
                        self.collect_pass(descent_collector.clone(), &context_path)?;
                    match do_next_pass {
                        true => {
                            tracing::debug!(
                                "execute::Worker::execute: After {} readonly pass, needs another pass for context: {:?}",
                                i, context_path
                            );
                        }
                        false => {
                            // Ready for final collet pass
                            break;
                        }
                    };
                }
                // Last re-read file, parse it, collect data
                let (do_next_pass, descent_collector) =
                    self.collect_pass(descent_collector, &context_path)?;
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
                            "execute::Worker::execute: Successfully collected context: {:?} got {}",
                            context_path,
                            descent_collector.context().to_string()
                        );
                        return Ok(Some(collector.ascend(descent_collector)));
                    }
                };
            }
        }
    }

    /// Calls an external model (LLM) with the provided content and parameters.
    ///
    /// This function constructs a prompt using the given `content` and any `prefix`
    /// or `postfix` specified in the `parameters`. It then uses the `provider`
    /// parameter to determine which external model to call via the shell.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The [`Parameters`] containing model-specific settings like `provider`,
    ///                  `prefix`, and `postfix`.
    /// * `content` - The [`ModelContent`] to be sent to the model as the main prompt body.
    ///
    /// # Returns
    ///
    /// A `Result` containing the model's response as a `String`, or an [`ExecuteError`]
    /// if any parameter is invalid, missing, or the shell call fails.
    ///
    /// # Errors
    ///
    /// Returns [`ExecuteError::UnsupportedParameterValue`] if `prefix`, `postfix`, or `provider`
    /// parameters have invalid values.
    /// Returns [`ExecuteError::MissingParameter`] if the `provider` parameter is not found.
    /// Returns [`ExecuteError::ShellError`] if the external shell command fails.
    pub(crate) fn call_model(
        &self,
        parameters: &Parameters,
        content: &ModelContent,
    ) -> Result<String> {
        let mut prompt = ModelContent::new();
        match parameters.get("prefix") {
            Some(JsonPlusEntity::NudeString(x)) => {
                prompt.push(ModelContentItem::system(&self.execute(x)?.to_string()));
            }
            Some(x) => {
                return Err(ExecuteError::UnsupportedParameterValue(format!(
                    "bad prefix: {:?}",
                    x
                )));
            }
            None => {}
        }
        prompt.extend(content.clone());
        match parameters.get("postfix") {
            Some(JsonPlusEntity::NudeString(x)) => {
                prompt.push(ModelContentItem::system(&self.execute(x)?.to_string()));
            }
            Some(x) => {
                return Err(ExecuteError::UnsupportedParameterValue(format!(
                    "bad postfix: {:?}",
                    x
                )));
            }
            None => {}
        }
        let provider = match parameters.get("provider") {
            Some(
                JsonPlusEntity::NudeString(x)
                | JsonPlusEntity::SingleQuotedString(x)
                | JsonPlusEntity::DoubleQuotedString(x),
            ) => x,
            Some(x) => {
                return Err(ExecuteError::UnsupportedParameterValue(format!(
                    "bad provider: {:?}",
                    x
                )));
            }
            None => {
                return Err(ExecuteError::MissingParameter("provider".to_string()));
            }
        };
        crate::agent::shell::shell_call(&provider, &prompt.to_prompt())
            .map_err(|e| ExecuteError::ShellError(e.to_string()))
    }

    /// Executes a single read-only pass over the context file.
    ///
    /// This pass collects content and processes tags/anchors without modifying
    /// the source file or calling external models. It's used for initial collection
    /// and for subsequent passes after modifications.
    ///
    /// # Arguments
    ///
    /// * `collector` - The current [`Collector`] state.
    /// * `context_path` - The path to the context file being processed.
    ///
    /// # Returns
    ///
    /// A `Result` containing a tuple: `(do_next_pass, updated_collector)`.
    /// `do_next_pass` is `true` if another pass is required (e.g., due to state evolution),
    /// `false` otherwise. `updated_collector` is the [`Collector`] after this pass.
    ///
    /// # Errors
    ///
    /// Returns various [`ExecuteError`] variants if file operations, AST parsing,
    /// or tag/anchor processing fail.
    fn collect_pass(&self, collector: Collector, context_path: &Path) -> Result<(bool, Collector)> {
        self._pass_internal(collector, context_path, true)
    }

    /// Executes a single modifying pass over the context file.
    ///
    /// This pass processes tags/anchors that can lead to modifications of the
    /// source file (e.g., injecting model responses). It acquires a file lock
    /// to ensure exclusive access during modification.
    ///
    /// # Arguments
    ///
    /// * `collector` - The current [`Collector`] state.
    /// * `context_path` - The path to the context file being processed.
    ///
    /// # Returns
    ///
    /// A `Result` containing a tuple: `(do_next_pass, updated_collector)`.
    /// `do_next_pass` is `true` if another pass is required (e.g., due to modifications),
    /// `false` otherwise. `updated_collector` is the [`Collector`] after this pass.
    ///
    /// # Errors
    ///
    /// Returns various [`ExecuteError`] variants if file operations, AST parsing,
    /// or tag/anchor processing fail.
    fn execute_pass(&self, collector: Collector, context_path: &Path) -> Result<(bool, Collector)> {
        let _lock = crate::file::FileLock::new(self.file_access.clone(), context_path)?;
        self._pass_internal(collector, context_path, false)
    }

    /// Internal function to perform a single pass (either read-only or modifying) over a context file.
    ///
    /// This function reads the file, parses its AST, and iterates through its content
    /// (text, tags, anchors). It dispatches to appropriate tag/anchor behaviors based
    /// on the `readonly` flag. If patches are generated in a non-readonly pass, they
    /// are applied, and a new pass is triggered.
    ///
    /// # Arguments
    ///
    /// * `collector` - The current [`Collector`] state.
    /// * `context_path` - The path to the context file being processed.
    /// * `readonly` - A boolean flag: `true` for a read-only pass (collection), `false` for a modifying pass.
    ///
    /// # Returns
    ///
    /// A `Result` containing a tuple: `(do_next_pass, updated_collector)`.
    /// `do_next_pass` is `true` if another pass is required (e.g., due to modifications or state evolution),
    /// `false` otherwise. `updated_collector` is the [`Collector`] after this pass.
    ///
    /// # Panics
    ///
    /// Panics if patches are produced during a `readonly` pass, as this indicates a logic error.
    ///
    /// # Errors
    ///
    /// Returns various [`ExecuteError`] variants for issues like file access failures,
    /// AST parsing errors, missing end anchors, or failures in tag/anchor behavior execution.
    fn _pass_internal(
        &self,
        mut collector: Collector,
        context_path: &Path,
        readonly: bool,
    ) -> Result<(bool, Collector)> {
        let context_content = self.file_access.read_file(context_path)?;
        let ast = crate::ast2::parse_document(&context_content)?;
        let anchor_index = super::AnchorIndex::new(&ast.content);

        for item in &ast.content {
            let (do_next_pass, next_collector, patches) = match item {
                Content::Text(text) => {
                    collector = collector.set_latest_range(&text.range);
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
                    collector = collector.set_latest_range(&tag.range);
                    let integrated_tag = tag.clone().integrate(&collector.default_parameters);
                    if readonly {
                        let (do_next_pass, collector) =
                            TagBehaviorDispatch::collect_tag(self, collector, &integrated_tag)?;
                        (do_next_pass, collector, vec![])
                    } else {
                        TagBehaviorDispatch::execute_tag(self, collector, &integrated_tag)?
                    }
                }
                Content::Anchor(anchor) => match anchor.kind {
                    AnchorKind::Begin => {
                        collector = collector.set_latest_range(&anchor.range);
                        let anchor_end = anchor_index
                            .get_end(&anchor.uuid)
                            .ok_or(ExecuteError::EndAnchorNotFound(anchor.uuid))?;
                        let anchor_end = ast
                            .content
                            .get(anchor_end)
                            .and_then(|c| match c {
                                Content::Anchor(a) => Some(a),
                                _ => None,
                            })
                            .ok_or(ExecuteError::EndAnchorNotFound(anchor.uuid))?;
                        let (do_next_pass, new_collector, patches) = if readonly {
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
            } else if readonly {
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

    /// Applies a series of text patches to the given content.
    ///
    /// Patches are applied in reverse order to ensure that `Range`s remain valid
    /// even if previous patches change the length of the content.
    ///
    /// # Arguments
    ///
    /// * `context_content` - The original string content to which patches will be applied.
    /// * `patches` - A vector of tuples, where each tuple contains a [`Range`] and the
    ///               `String` to replace the content within that range.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `String` with all patches applied.
    ///
    /// # Errors
    ///
    /// This function does not directly return errors, but panics if byte offsets
    /// derived from character offsets are out of bounds, which should not happen
    /// with valid `Range` objects.
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

    /// Constructs the file system path for storing the state of a dynamic command.
    ///
    /// Dynamic commands (like `@answer` or `@repeat`) persist their state in JSON files
    /// within a metadata directory associated with their UUID.
    ///
    /// # Arguments
    ///
    /// * `command` - The [`CommandKind`] of the dynamic command.
    /// * `uuid` - The unique identifier of the command instance.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `PathBuf` to the state file.
    ///
    /// # Errors
    ///
    /// Returns [`ExecuteError::PathResolutionError`] if the metadata path cannot be resolved.
    fn get_state_path(&self, command: CommandKind, uuid: &Uuid) -> Result<PathBuf> {
        let meta_path = self
            .path_res
            .resolve_metadata(&command.to_string(), &uuid)?;
        let state_path = meta_path.join("state.json");
        Ok(state_path)
    }

    /// Loads the state of a dynamic command from its associated JSON file.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type into which the JSON state should be deserialized. Must implement `serde::de::DeserializeOwned`.
    ///
    /// # Arguments
    ///
    /// * `command` - The [`CommandKind`] of the dynamic command.
    /// * `uuid` - The unique identifier of the command instance.
    ///
    /// # Returns
    ///
    /// A `Result` containing the deserialized state object of type `T`.
    ///
    /// # Errors
    ///
    /// Returns [`ExecuteError::PathResolutionError`] if the state path cannot be resolved.
    /// Returns [`ExecuteError::IoError`] if the file cannot be read.
    /// Returns [`ExecuteError::JsonError`] if the file content is not valid JSON or cannot be deserialized into `T`.
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

    /// Saves the state of a dynamic command to its associated JSON file.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of the state object to serialize. Must implement `serde::Serialize`.
    ///
    /// # Arguments
    ///
    /// * `command` - The [`CommandKind`] of the dynamic command.
    /// * `uuid` - The unique identifier of the command instance.
    /// * `state` - A reference to the state object to be serialized and saved.
    /// * `comment` - An optional comment to associate with the file write operation.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an [`ExecuteError`] if the state cannot be saved.
    ///
    /// # Errors
    ///
    /// Returns [`ExecuteError::PathResolutionError`] if the state path cannot be resolved.
    /// Returns [`ExecuteError::JsonError`] if the state object cannot be serialized to JSON.
    /// Returns [`ExecuteError::IoError`] if the file cannot be written.
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

    /// Converts a dynamic tag into a pair of anchor tags and potentially injects output.
    ///
    /// When a dynamic tag (e.g., `@answer`) is first encountered, it is transformed
    /// into a pair of `<!-- @@begin_anchor... -->` and `<!-- @@end_anchor... -->`
    /// markers in the source document. If the output is not redirected, the `output`
    /// content is placed between these new anchors.
    ///
    /// # Arguments
    ///
    /// * `collector` - The current [`Collector`] state (used for context, though not directly modified here).
    /// * `tag` - The [`Tag`] to be converted.
    /// * `output` - The content to potentially inject between the new anchors.
    ///
    /// # Returns
    ///
    /// A `Result` containing a tuple: `(uuid, patches)`.
    /// `uuid` is the UUID of the newly created anchor. `patches` is a vector of
    /// `(Range, String)` tuples representing the modifications to be applied to the source file.
    ///
    /// # Errors
    ///
    /// Returns [`ExecuteError::UnsupportedParameterValue`] if the `output` parameter is invalid.
    /// Returns [`ExecuteError::PathResolutionError`] if an output redirection path cannot be resolved.
    /// Returns [`ExecuteError::IoError`] if an output redirection file cannot be written.
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

    /// Injects content into an existing anchor or clears its content if output is redirected.
    ///
    /// This function is used when a dynamic anchor (e.g., `<!-- @@answer...@@ -->`)
    /// has produced new content (e.g., a model response). If the output is redirected,
    /// the content between the anchor tags is cleared. Otherwise, the `output` content
    /// replaces the existing content between the anchor tags.
    ///
    /// # Arguments
    ///
    /// * `collector` - The current [`Collector`] state (used for context, though not directly modified here).
    /// * `anchor` - The beginning [`Anchor`] tag.
    /// * `anchor_end` - The [`Position`] of the end of the anchor's content (i.e., the start of the closing anchor tag).
    /// * `output` - The content to inject or the content that was redirected.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `(Range, String)` tuples representing the
    /// modifications to be applied to the source file.
    ///
    /// # Errors
    ///
    /// Returns [`ExecuteError::UnsupportedParameterValue`] if the `output` parameter is invalid.
    /// Returns [`ExecuteError::PathResolutionError`] if an output redirection path cannot be resolved.
    /// Returns [`ExecuteError::IoError`] if an output redirection file cannot be written.
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

    /// Checks if the output of a command or anchor is redirected to a file.
    ///
    /// This function inspects the `output` parameter within the provided `Parameters`.
    /// If an `output` path is specified, it attempts to resolve it.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The [`Parameters`] to check for an `output` redirection.
    ///
    /// # Returns
    ///
    /// A `Result` containing `Some(PathBuf)` if output is redirected to a valid path,
    /// `None` if no `output` parameter is present, or an [`ExecuteError`] if the
    /// `output` parameter has an invalid value or path cannot be resolved.
    ///
    /// # Errors
    ///
    /// Returns [`ExecuteError::UnsupportedParameterValue`] if the `output` parameter
    /// is not a valid string.
    /// Returns [`ExecuteError::PathResolutionError`] if the specified output path cannot be resolved.
    pub fn is_output_redirected(&self, parameters: &Parameters) -> Result<Option<PathBuf>> {
        match &parameters.get("output") {
            Some(JsonPlusEntity::NudeString(x)) => {
                let output_path = self.path_res.resolve_context(&x)?;
                return Ok(Some(output_path));
            }
            Some(x) => {
                return Err(ExecuteError::UnsupportedParameterValue(format!(
                    "output: {:?}",
                    x
                )));
            }
            None => {
                return Ok(None);
            }
        }
    }

    /// Redirects the given `output` string to a file specified in the `parameters`.
    ///
    /// If the `output` parameter is present and valid, the `output` string is written
    /// to the resolved file path. If no `output` parameter is found, no action is taken.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The [`Parameters`] which may contain an `output` path.
    /// * `output` - The `String` content to be written to the file if redirection occurs.
    ///
    /// # Returns
    ///
    /// A `Result` containing `true` if the output was redirected and written to a file,
    /// `false` otherwise. Returns an [`ExecuteError`] if redirection fails.
    ///
    /// # Errors
    ///
    /// Returns [`ExecuteError::UnsupportedParameterValue`] if the `output` parameter
    /// is not a valid string.
    /// Returns [`ExecuteError::PathResolutionError`] if the specified output path cannot be resolved.
    /// Returns [`ExecuteError::IoError`] if the file cannot be written.
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

    /// Handles input redirection, potentially replacing the current input content.
    ///
    /// If an `input` parameter is present in the `parameters`, this function attempts
    /// to resolve the input path and execute that context, effectively replacing
    /// the `input` `ModelContent` with the result of the redirected execution.
    /// If no `input` parameter is found, the original `input` `ModelContent` is returned.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The [`Parameters`] which may contain an `input` path.
    /// * `input` - The original [`ModelContent`] that might be replaced.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `ModelContent` (either the original or the redirected/executed content).
    ///
    /// # Errors
    ///
    /// Returns [`ExecuteError::UnsupportedParameterValue`] if the `input` parameter
    /// is not a valid string.
    /// Returns [`ExecuteError::PathResolutionError`] if the specified input path cannot be resolved.
    /// Returns other [`ExecuteError`] variants if the redirected context execution fails.
    pub fn redirect_input(
        &self,
        parameters: &Parameters,
        input: ModelContent,
    ) -> Result<ModelContent> {
        match &parameters.get("input") {
            Some(JsonPlusEntity::NudeString(x)) => {
                let output_path = self.path_res.resolve_context(&x)?;
                self.execute(&x)
            }
            Some(x) => {
                return Err(ExecuteError::UnsupportedParameterValue(format!(
                    "input: {:?}",
                    x
                )));
            }
            None => {
                return Ok(input);
            }
        }
    }

    /// Generates patches to mutate an existing anchor in the document.
    ///
    /// This function creates a patch that replaces the content of an anchor
    /// with its string representation, effectively ensuring the anchor is correctly
    /// formatted in the source document.
    ///
    /// # Arguments
    ///
    /// * `anchor` - The [`Anchor`] to be mutated.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `(Range, String)` tuples representing the
    /// modification to be applied to the source file.
    pub fn mutate_anchor(&self, anchor: &Anchor) -> Result<Vec<(Range, String)>> {
        Ok(vec![(anchor.range, format!("{}\n", anchor.to_string()))])
    }
}
