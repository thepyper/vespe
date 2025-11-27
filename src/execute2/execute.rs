//! This module contains the core logic for the execution engine, orchestrating the
//! processing of documents containing directives (tags and anchors). It defines
//! the `Worker` responsible for driving the execution and the `Collector` for
//! managing the execution state and accumulating results. The module implements
//! a multi-pass execution strategy to handle dynamic content generation and
//! modification.
use super::{ExecuteError, Result};
use crate::ast2::{
    Anchor, AnchorKind, CommandKind, Content, JsonPlusEntity, JsonPlusObject, Parameters, Range,
    Tag,
};
use crate::execute2::content::{ModelContent, ModelContentItem, PromptConfig, PromptFormat};
use crate::execute2::tag_answer::AnswerStatus;
use crate::execute2::tags::TagBehaviorDispatch;
use crate::utils::file::FileAccessor;
use crate::utils::path::PathResolver;
use crate::utils::task::{TaskManager, TaskStatus};
use std::sync::mpsc;

use handlebars::Handlebars;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use uuid::Uuid;

const TAB_SIZE: usize = 4;

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
    data: Option<&JsonPlusObject>,
) -> Result<ModelContent> {
    tracing::debug!("Executing context: {}", context_name);

    let exe = Worker::new(file_access, path_res);
    exe.execute(context_name, data)
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
    data: Option<&JsonPlusObject>,
) -> Result<ModelContent> {
    tracing::debug!("Collecting context: {}", context_name);

    let exe = Worker::new(file_access, path_res);

    exe.collect(context_name, data)
}

/// Central state object for the execution engine.
///
/// The `Collector` accumulates the final `ModelContent` and tracks execution-time
/// variables and the call stack to prevent infinite recursion.
/// It is passed by value through the execution flow, ensuring a functional-style,
/// predictable state management.
#[derive(Clone, Debug)]
pub(crate) struct Collector {
    /// A stack of visited context file paths to detect and prevent circular includes.
    visit_stack: Vec<PathBuf>,
    /// A stack of entered anchors in current context file. This helps in tracking
    /// the nesting level of dynamic content.
    anchor_stack: Vec<Anchor>,
    /// The accumulated content that will be sent to the model. This is the primary
    /// output of the collection process.
    context: ModelContent,
    /// A SHA256 hasher used to compute a hash of the accumulated context,
    /// useful for detecting changes and caching.
    context_hasher: Sha256,
    /// Execution-time default parameters for tags. These parameters can be
    /// inherited or overridden by individual tags.
    default_parameters: Parameters,
    /// The latest processed range in the document. Used primarily for error reporting
    /// to pinpoint the location of issues.
    latest_range: Range,
    /// The latest task anchor encountered. This is used to manage the state
    /// and behavior of task-specific directives.
    latest_task: Option<Anchor>,
    /// The hash of the latest agent that contributed to the context. This helps
    /// in maintaining agent identity across turns.
    latest_agent_hash: Option<String>,
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

    /// Returns the SHA256 hash of the accumulated `ModelContent`.
    ///
    /// This hash represents the current state of the prompt and can be used for
    /// caching or detecting changes.
    ///
    /// # Returns
    ///
    /// A `String` representing the hexadecimal SHA256 hash.
    pub fn context_hash(&self) -> String {
        format!("{:x}", self.context_hasher.clone().finalize())
    }

    /// Computes the SHA256 hash of a given input string.
    ///
    /// # Arguments
    ///
    /// * `input` - The string to hash.
    ///
    /// # Returns
    ///
    /// A `String` representing the hexadecimal SHA256 hash of the input.
    pub fn hash(input: &str) -> String {
        let mut context_hasher = Sha256::new();
        context_hasher.update(input);
        format!("{:x}", context_hasher.finalize())
    }

    pub fn normalize_text(input: &str) -> String {
        let mut out = String::with_capacity(input.len());
        let mut chars = input.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '\r' => {
                    if matches!(chars.peek(), Some('\n')) {
                        chars.next();
                    }
                    out.push('\n');
                }
                '\n' => out.push('\n'),
                _ => {
                    let mut line_buf = String::new();
                    let mut col = 0;

                    Self::push_with_tabs(c, &mut line_buf, &mut col);

                    while let Some(&ch) = chars.peek() {
                        if ch == '\n' || ch == '\r' {
                            break;
                        }
                        chars.next();
                        Self::push_with_tabs(ch, &mut line_buf, &mut col);
                    }

                    let trimmed_len = line_buf.trim_end().len();
                    out.push_str(&line_buf[..trimmed_len]);
                }
            }
        }

        out
    }

    #[inline]
    fn push_with_tabs(ch: char, buf: &mut String, col: &mut usize) {
        if ch == '\t' {
            let spaces = TAB_SIZE - (*col % TAB_SIZE);
            buf.extend(std::iter::repeat(' ').take(spaces));
            *col += spaces;
        } else {
            buf.push(ch);
            *col += 1;
        }
    }

    /// Computes the normalized SHA256 hash of a given input string.
    ///
    /// Normalization involves replacing `\r\n` with `\n` and trimming whitespace
    /// from each line before hashing. This ensures consistent hashes across
    /// different operating systems and minor formatting variations.
    ///
    /// # Arguments
    ///
    /// * `input` - The string to hash and normalize.
    ///
    /// # Returns
    ///
    /// A `String` representing the hexadecimal SHA256 hash of the normalized input.
    pub fn normalized_hash(input: &str) -> String {
        let normalized_input = Self::normalize_text(input);
        Self::hash(&normalized_input)
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

    /// Checks if the collector is currently inside an anchor of a specific `CommandKind`.
    ///
    /// This method iterates through the `anchor_stack` to determine if any active
    /// anchor matches the provided `kind`.
    ///
    /// # Arguments
    ///
    /// * `kind` - The `CommandKind` to check for (e.g., `CommandKind::Task`).
    ///
    /// # Returns
    ///
    /// An `Option<&Anchor>`: `Some(&Anchor)` if an anchor of the specified kind is found
    /// in the stack, `None` otherwise.
    pub fn is_in_this_kind_of_anchor(&self, kind: CommandKind) -> Option<&Anchor> {
        for anchor in &self.anchor_stack {
            if anchor.command == kind {
                return Some(anchor);
            }
        }
        None
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
            context_hasher: Sha256::new(),
            default_parameters: Parameters::new(),
            latest_range: Range::null(),
            latest_task: None,
            latest_agent_hash: None,
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
            context_hasher: self.context_hasher.clone(),
            default_parameters: self.default_parameters.clone(),
            latest_range: Range::null(),
            latest_task: None,
            latest_agent_hash: None,
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
        self.context_hasher = descent_collector.context_hasher;
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
        self.context_hasher = Sha256::new();
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

    /// Exits the given anchor, removing it from the anchor stack.
    ///
    /// # Arguments
    ///
    /// * `anchor` - The [`Anchor`] to exit.
    ///
    /// # Returns
    ///
    /// A `Result<Self>`: `Ok(Collector)` if an anchor was successfully popped,
    /// or an `Err(ExecuteError::EmptyAnchorStack)` if the stack was already empty.
    ///
    /// # Errors
    ///
    /// Returns [`ExecuteError::EmptyAnchorStack`] if there are no anchors to exit.
    fn exit(mut self, anchor: &Anchor) -> Result<Self> {
        let size_before = self.anchor_stack.len();
        self.anchor_stack = self
            .anchor_stack
            .into_iter()
            .filter(|x| x.uuid != anchor.uuid)
            .collect::<Vec<Anchor>>();
        let size_after = self.anchor_stack.len();
        if size_before == size_after {
            return Err(ExecuteError::EmptyAnchorStack(self.latest_range.clone()));
        }
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
        let normalized_item = Self::normalize_text(&item.to_string());
        self.context_hasher.update(normalized_item);
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

    /// Returns the latest task anchor encountered by the collector.
    ///
    /// # Returns
    ///
    /// An `Option<Anchor>` containing a clone of the latest task anchor, or `None` if no task anchor has been set.
    pub fn latest_task(&self) -> Option<Anchor> {
        self.latest_task.clone()
    }

    /// Sets the latest task anchor encountered by the collector.
    ///
    /// # Arguments
    ///
    /// * `task_anchor` - The `Anchor` representing the latest task.
    ///
    /// # Returns
    ///
    /// The `Collector` with the updated latest task anchor.
    pub fn set_latest_task(mut self, task_anchor: &Anchor) -> Self {
        self.latest_task = Some(task_anchor.clone());
        self
    }

    /// Returns the hash of the latest agent that contributed to the context.
    ///
    /// # Returns
    ///
    /// An `Option<String>` containing a clone of the latest agent hash, or `None` if no agent hash has been set.
    pub fn latest_agent_hash(&self) -> Option<String> {
        self.latest_agent_hash.clone()
    }

    /// Sets the hash of the latest agent that contributed to the context.
    ///
    /// # Arguments
    ///
    /// * `latest_agent_hash` - An `Option<String>` containing the new agent hash.
    ///
    /// # Returns
    ///
    /// The `Collector` with the updated latest agent hash.
    pub fn set_latest_agent_hash(mut self, latest_agent_hash: Option<String>) -> Self {
        self.latest_agent_hash = latest_agent_hash;
        self
    }
}

/// The stateless engine that drives the context execution.
///
/// The `Worker` holds thread-safe handles to the tools needed for execution,
/// such as the file accessor and path resolver. It contains the core logic
/// for the multi-pass execution strategy.
#[derive(Debug, Clone)]
pub(crate) struct Worker {
    file_access: Arc<dyn FileAccessor>,
    path_res: Arc<dyn PathResolver>,
    task_manager: TaskManager<String, String, String>,
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
            task_manager: TaskManager::new(),
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
    pub fn execute(
        &self,
        context_name: &str,
        data: Option<&JsonPlusObject>,
    ) -> Result<ModelContent> {
        match self._execute(Collector::new(), context_name, 77, data)? {
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
    pub fn collect(
        &self,
        context_name: &str,
        data: Option<&JsonPlusObject>,
    ) -> Result<ModelContent> {
        match self._execute(Collector::new(), context_name, 0, data)? {
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
        data: Option<&JsonPlusObject>,
    ) -> Result<Option<Collector>> {
        tracing::debug!(
            "Worker::execute Executing context: {} with data {:?}",
            context_name,
            data,
        );
        let context_path = self.path_res.resolve_input_file(context_name)?;
        tracing::debug!(
            "Worker::execute Context resolved to file: {}",
            context_path.display(),
        );

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
                    tracing::debug!(
                        "execute::Worker::execute: Running pass {}/{}.",
                        i,
                        max_rewrite_steps
                    );
                    // Lock file, read it (could be edited outside), parse it, execute fast things that may modify context and save it
                    let (do_next_pass, _) =
                        self.execute_pass(descent_collector.clone(), &context_path, data)?;
                    match do_next_pass {
                        true => {
                            tracing::debug!(
                                "execute::Worker::execute: After {}/{} modifying pass, needs another pass for context: {:?}",
                                i, max_rewrite_steps, context_path
                            );
                        }
                        false => {
                            // Ready for final collect pass
                            break;
                        }
                    };
                    // Re-read file, parse it, execute slow things that do not modify context, collect data
                    let (do_next_pass, _) =
                        self.collect_pass(descent_collector.clone(), &context_path, data)?;
                    match do_next_pass {
                        true => {
                            tracing::debug!(
                                "execute::Worker::execute: After {}/{} readonly pass, needs another pass for context: {:?}",
                                i, max_rewrite_steps, context_path
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
                    self.collect_pass(descent_collector, &context_path, data)?;
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

    /// Prepends a `ModelContent` to another.
    ///
    /// This utility function takes a `prefix` content and adds it to the beginning
    /// of the main `content`.
    ///
    /// # Arguments
    ///
    /// * `content` - The main [`ModelContent`].
    /// * `prefix` - The [`ModelContent`] to prepend.
    ///
    /// # Returns
    ///
    /// The combined `ModelContent` with the prefix at the start.
    pub fn prefix_content(&self, content: ModelContent, mut prefix: ModelContent) -> ModelContent {
        prefix.extend(content);
        prefix
    }

    /// Prepends content to a `ModelContent` based on a `prefix` parameter.
    ///
    /// This function checks for a `prefix` parameter. If found, it executes the
    /// context specified by the parameter's value and prepends the resulting content
    /// as a `system` message to the main `content`.
    ///
    /// # Arguments
    ///
    /// * `content` - The main [`ModelContent`].
    /// * `parameters` - The [`Parameters`] to check for a `prefix` value.
    ///
    /// # Returns
    ///
    /// A `Result` containing the potentially prefixed `ModelContent`.
    ///
    /// # Errors
    ///
    /// Returns [`ExecuteError::UnsupportedParameterValue`] if the `prefix` parameter
    /// is not a string. Returns errors from the underlying context execution if the
    /// specified prefix context fails to execute.
    pub fn prefix_content_from_parameters(
        &self,
        content: ModelContent,
        parameters: &Parameters,
    ) -> Result<ModelContent> {
        let prefix = self.process_context_with_data_from_parameters(parameters, "prefix")?;
        match prefix {
            Some(prefix) => {
                let prefix = ModelContentItem::system(&prefix.to_string());
                let prefix = ModelContent::from_item(prefix);
                Ok(self.prefix_content(content, prefix))
            }
            None => Ok(content),
        }
    }

    /// Appends a `ModelContent` to another.
    ///
    /// This utility function takes a `postfix` content and adds it to the end
    /// of the main `content`.
    ///
    /// # Arguments
    ///
    /// * `content` - The main [`ModelContent`].
    /// * `postfix` - The [`ModelContent`] to append.
    ///
    /// # Returns
    ///
    /// The combined `ModelContent` with the postfix at the end.
    pub fn postfix_content(
        &self,
        mut content: ModelContent,
        postfix: ModelContent,
    ) -> ModelContent {
        content.extend(postfix);
        content
    }

    /// Appends content to a `ModelContent` based on a `postfix` parameter.
    ///
    /// This function checks for a `postfix` parameter. If found, it executes the
    /// context specified by the parameter's value and appends the resulting content
    /// as a `system` message to the main `content`.
    ///
    /// # Arguments
    ///
    /// * `content` - The main [`ModelContent`].
    /// * `parameters` - The [`Parameters`] to check for a `postfix` value.
    ///
    /// # Returns
    ///
    /// A `Result` containing the potentially postfixed `ModelContent`.
    ///
    /// # Errors
    ///
    /// Returns [`ExecuteError::UnsupportedParameterValue`] if the `postfix` parameter
    /// is not a string. Returns errors from the underlying context execution if the
    /// specified postfix context fails to execute.
    pub fn postfix_content_from_parameters(
        &self,
        content: ModelContent,
        parameters: &Parameters,
    ) -> Result<ModelContent> {
        let postfix = self.process_context_with_data_from_parameters(parameters, "postfix")?;
        match postfix {
            Some(postfix) => {
                let postfix = ModelContentItem::merge_upstream(&postfix.to_string());
                let postfix = ModelContent::from_item(postfix);
                Ok(self.postfix_content(content, postfix))
            }
            None => Ok(content),
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
        prompt: String,
        progress_callback: impl Fn(&str) + Send + Sync + 'static,
    ) -> Result<String> {
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
        let response = crate::agent::shell::shell_call(&provider, &prompt, progress_callback)
            .map_err(|e| ExecuteError::ShellError(e.to_string()))?;
        Ok(response)
    }

    pub fn craft_prompt(
        &self,
        agent_hash: Option<String>,
        parameters: &Parameters,
        prompt: &ModelContent,
    ) -> Result<String> {
        let prompt_config = PromptConfig {
            agent: agent_hash,
            format: PromptFormat::Parts,
            with_agent_names: parameters.get_as_bool("with_agent_names").unwrap_or(false),
            with_invitation: parameters.get_as_bool("with_invitation").unwrap_or(false),
        };
        let prompt = prompt.to_prompt(&prompt_config);
        Ok(prompt)
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
    fn collect_pass(
        &self,
        collector: Collector,
        context_path: &Path,
        data: Option<&JsonPlusObject>,
    ) -> Result<(bool, Collector)> {
        self._pass_internal(collector, context_path, true, data)
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
    fn execute_pass(
        &self,
        collector: Collector,
        context_path: &Path,
        data: Option<&JsonPlusObject>,
    ) -> Result<(bool, Collector)> {
        let _lock = crate::utils::file::FileLock::new(self.file_access.clone(), context_path)?;
        self._pass_internal(collector, context_path, false, data)
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
        data: Option<&JsonPlusObject>,
    ) -> Result<(bool, Collector)> {
        let document = self.file_access.read_file(context_path)?;
        let document = match data {
            Some(data) => self.process_context_with_data(document, data)?,
            None => document,
        };
        let ast = crate::ast2::parse_document(&document)?;
        let anchor_index = super::utils::AnchorIndex::new(&ast.content);

        for item in &ast.content {
            let (do_next_pass, next_collector, patches) = match item {
                Content::Comment(_) => {
                    // Ignore comments
                    (false, collector, vec![])
                }
                Content::Text(text) => {
                    collector = collector.set_latest_range(&text.range);
                    if let Some(_) = collector.is_in_this_kind_of_anchor(CommandKind::Task) {
                        // Do not collect text inside task anchor
                        // TODO spostare altrove? logica di task in pass? come fare?
                        tracing::debug!("Removed by task anchor: {:?}", text.content);
                    } else if let Some(anchor) =
                        collector.is_in_this_kind_of_anchor(CommandKind::Answer)
                    {
                        // Inside answer anchor, check if anchor is edited
                        if anchor.status == Some(AnswerStatus::Edited.to_string()) {
                            // Edited content, then it's user
                            collector = collector.push_item(ModelContentItem::user(&text.content));
                        } else {
                            // Unedited content, then it's assistant
                            let agent_hash = collector.latest_agent_hash();
                            collector = collector
                                .push_item(ModelContentItem::agent(agent_hash, &text.content));
                        }
                    } else {
                        // User writes outside answer anchors
                        collector = collector.push_item(ModelContentItem::user(&text.content));
                    }
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
                        TagBehaviorDispatch::execute_tag(
                            self,
                            collector,
                            &document,
                            &integrated_tag,
                        )?
                    }
                }
                Content::Anchor(anchor) => match anchor.kind {
                    AnchorKind::Begin => {
                        collector = collector.set_latest_range(&anchor.range);
                        match anchor.command {
                            CommandKind::Task => {
                                collector = collector.set_latest_task(&anchor);
                            }
                            _ => {}
                        };
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
                                self, collector, &document, anchor, anchor_end, false,
                            )?;
                            (do_next_pass, collector, vec![])
                        } else {
                            TagBehaviorDispatch::execute_anchor(
                                self, collector, &document, anchor, anchor_end, false,
                            )?
                        };
                        (do_next_pass, new_collector.enter(anchor), patches)
                    }
                    AnchorKind::End => {
                        collector = collector.set_latest_range(&anchor.range);
                        let anchor_begin = anchor_index
                            .get_begin(&anchor.uuid)
                            .ok_or(ExecuteError::EndAnchorNotFound(anchor.uuid))?;
                        let anchor_begin = ast
                            .content
                            .get(anchor_begin)
                            .and_then(|c| match c {
                                Content::Anchor(a) => Some(a),
                                _ => None,
                            })
                            .ok_or(ExecuteError::EndAnchorNotFound(anchor.uuid))?;
                        let (do_next_pass, new_collector, patches) = if readonly {
                            let (do_next_pass, collector) = TagBehaviorDispatch::collect_anchor(
                                self,
                                collector,
                                &document,
                                anchor_begin,
                                anchor,
                                true,
                            )?;
                            (do_next_pass, collector, vec![])
                        } else {
                            TagBehaviorDispatch::execute_anchor(
                                self,
                                collector,
                                &document,
                                anchor_begin,
                                anchor,
                                true,
                            )?
                        };
                        (do_next_pass, new_collector.exit(anchor)?, patches)
                    }
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
                let new_content = Self::apply_patches(&document, patches)?;
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
    /// * `document` - The original string content to which patches will be applied.
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
    fn apply_patches(document: &str, mut patches: Vec<(Range, String)>) -> Result<String> {
        let mut result = document.to_string();
        patches.sort();
        for (range, replace) in patches.iter().rev() {
            tracing::debug!("Replacing range {:?} with ***{}***", range, replace);
            let start_byte = document
                .char_indices()
                .nth(range.begin.offset)
                .map(|(i, _)| i)
                .unwrap_or(document.len());
            let end_byte = document
                .char_indices()
                .nth(range.end.offset)
                .map(|(i, _)| i)
                .unwrap_or(document.len());
            result.replace_range(start_byte..end_byte, replace);
        }
        Ok(result)
    }

    /// Extracts a substring from the document based on the provided `Range`.
    ///
    /// This is a utility function to safely get a slice of the document content
    /// corresponding to a specific character range.
    ///
    /// # Arguments
    ///
    /// * `document` - The full document string.
    /// * `range` - The [`Range`] specifying the start and end character offsets.
    ///
    /// # Returns
    ///
    /// A `Result` containing a string slice (`&str`) of the content within the range.
    ///
    /// # Errors
    ///
    /// This function does not directly return errors, but panics if byte offsets
    /// derived from character offsets are out of bounds, which should not happen
    /// with valid `Range` objects.
    pub fn get_range<'a>(document: &'a str, range: &Range) -> Result<&'a str> {
        let start_byte = document
            .char_indices()
            .nth(range.begin.offset)
            .map(|(i, _)| i)
            .unwrap_or(document.len());
        let end_byte = document
            .char_indices()
            .nth(range.end.offset)
            .map(|(i, _)| i)
            .unwrap_or(document.len());
        Ok(&document[start_byte..end_byte])
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
        _collector: &Collector,
        tag: &Tag,
        status: Option<String>,
        output: &str,
    ) -> Result<(Uuid, (Range, String))> {
        let (a0, a1) = Anchor::new_couple(tag.command, status, &tag.parameters, &tag.arguments);
        match self.redirect_output(&tag.parameters, output)? {
            true => {
                // Output redirected, just convert tag into anchor
                Ok((
                    a0.uuid,
                    (
                        tag.range,
                        format!("{}\n{}\n", a0.to_string(), a1.to_string()),
                    ),
                ))
            }
            false => {
                // Output not redirected, include in new anchors
                Ok((
                    a0.uuid,
                    (
                        tag.range,
                        format!("{}\n{}{}\n", a0.to_string(), output, a1.to_string()),
                    ),
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
        _collector: &Collector,
        anchor_begin: &Anchor,
        anchor_end: &Anchor,
        output: &str,
    ) -> Result<(Range, String)> {
        match self.redirect_output(&anchor_begin.parameters, output)? {
            true => {
                // Output redirected, delete anchor contents
                Ok((
                    Range {
                        begin: anchor_begin.range.end,
                        end: anchor_end.range.begin,
                    },
                    String::new(),
                ))
            }
            false => {
                // Output not redirected, patch anchor contents
                Ok((
                    Range {
                        begin: anchor_begin.range.end,
                        end: anchor_end.range.begin,
                    },
                    output.to_string(),
                ))
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
                let output_path = self.path_res.resolve_output_file(&x)?;
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
    /// the current `ModelContent` with the result of the redirected execution.
    /// If no `input` parameter is found, the original `ModelContent` from the `collector`
    /// and its hash are returned.
    ///
    /// # Arguments
    ///
    /// * `collector` - The current [`Collector`] state, used to retrieve the current
    ///                 `ModelContent` if no redirection occurs, and to hash the content.
    /// * `parameters` - The [`Parameters`] which may contain an `input` path and
    ///                  optional `input_data` for the redirected context.
    ///
    /// # Returns
    ///
    /// A `Result` containing a tuple `(ModelContent, String)`. The `ModelContent` is
    /// either the original from the collector or the content from the redirected/executed
    /// context. The `String` is the SHA256 hash of the returned `ModelContent`.
    ///
    /// # Errors
    ///
    /// Returns [`ExecuteError::UnsupportedParameterValue`] if the `input` or `input_data`
    /// parameters have invalid values.
    /// Returns [`ExecuteError::PathResolutionError`] if the specified input path cannot be resolved.
    /// Returns other [`ExecuteError`] variants if the redirected context execution fails.
    pub fn redirect_input(
        &self,
        collector: &Collector,
        parameters: &Parameters,
    ) -> Result<(ModelContent, String)> {
        let input = self.process_context_with_data_from_parameters(parameters, "input")?;
        match input {
            None => Ok((collector.context().clone(), collector.context_hash())),
            Some(input) => {
                let input_hash = Collector::normalized_hash(&input.to_string());
                Ok((input, input_hash))
            }
        }
    }

    /// Generates patches to mutate an existing anchor in the document.
    ///
    /// This function creates a patch that replaces the content of an anchor
    /// with its string representation. This is typically used to update an anchor's
    /// parameters or state directly in the source file.
    ///
    /// # Arguments
    ///
    /// * `anchor` - The [`Anchor`] to be mutated.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `(Range, String)` tuples representing the
    /// modification to be applied to the source file.
    pub fn mutate_anchor(&self, anchor: &Anchor) -> Result<(Range, String)> {
        Ok((anchor.range, format!("{}\n", anchor.to_string())))
    }

    /// Reads the raw content of a context file.
    ///
    /// This is a utility function that resolves a context name to its file path
    /// and reads its entire content into a string.
    ///
    /// # Arguments
    ///
    /// * `context_name` - The name of the context to read.
    ///
    /// # Returns
    ///
    /// A `Result` containing the file's content as a `String`.
    ///
    /// # Errors
    ///
    /// Returns [`ExecuteError::PathResolutionError`] if the context name cannot be resolved.
    /// Returns [`ExecuteError::IoError`] if the file cannot be read.
    pub fn read_context(&self, context_name: &str) -> Result<String> {
        self.read_context_from_path(&self.path_res.resolve_input_file(context_name)?)
    }

    /// Reads the raw content of a context file from a given path.
    ///
    /// This is a utility function that reads the entire content of a file
    /// into a string.
    ///
    /// # Arguments
    ///
    /// * `context_path` - The path to the context file to read.
    ///
    /// # Returns
    ///
    /// A `Result` containing the file's content as a `String`.
    ///
    /// # Errors
    ///
    /// Returns [`ExecuteError::IoError`] if the file cannot be read.
    pub fn read_context_from_path(&self, context_path: &Path) -> Result<String> {
        let context_content = self.file_access.read_file(&context_path)?;
        Ok(context_content)
    }

    /// Processes the given context string using Handlebars templating with provided data.
    ///
    /// This allows for dynamic injection of data into the context content before
    /// further AST parsing and execution.
    ///
    /// # Arguments
    ///
    /// * `context` - The raw context string, potentially containing Handlebars expressions.
    /// * `data` - A `JsonPlusObject` containing the data to be used for templating.
    ///
    /// # Returns
    ///
    /// A `Result` containing the processed context string.
    ///
    /// # Errors
    ///
    /// Returns [`ExecuteError::RenderError`] if there is an issue during Handlebars rendering.
    pub fn process_context_with_data(
        &self,
        context: String,
        data: &JsonPlusObject,
    ) -> Result<String> {
        let handlebars = Handlebars::new();
        let data: serde_json::Value = data.into();
        let context = handlebars.render_template(&context, &data)?;
        Ok(context)
    }

    pub fn process_context_with_data_from_parameters(
        &self,
        parameters: &Parameters,
        key: &str,
    ) -> Result<Option<ModelContent>> {
        match parameters.get(key) {
            None => Ok(None),
            Some(x) => Ok(Some(self.process_context_from_jpe(&x, &parameters.range)?)),
        }
    }

    pub fn process_context_from_jpe(
        &self,
        jpe: &JsonPlusEntity,
        range: &Range,
    ) -> Result<ModelContent> {
        match jpe {
            JsonPlusEntity::NudeString(file_name)
            | JsonPlusEntity::SingleQuotedString(file_name)
            | JsonPlusEntity::DoubleQuotedString(file_name) => {
                let output = self.execute(&file_name, None)?;
                Ok(output)
            }
            JsonPlusEntity::Object(jpo) => {
                let file_name = jpo.get_as_string_only("context").ok_or_else(|| {
                    ExecuteError::MissingContextParameter {
                        range: range.clone(),
                    }
                })?;
                let data = jpo.get_as_object("data");
                let output = self.execute(&file_name, data)?;
                Ok(output)
            }
            JsonPlusEntity::Array(jpa) => {
                let mut output = ModelContent::new();
                for item in jpa {
                    output.extend(self.process_context_from_jpe(&item, range)?);
                }
                Ok(output)
            }
            _ => Err(ExecuteError::UnsupportedContextParameter {
                range: range.clone(),
            }),
        }
    }

    pub fn read_file(&self, file_path: &Path) -> Result<String> {
        Ok(self.file_access.read_file(file_path)?)
    }

    pub fn start_task(
        &self,
        id: &Uuid,
        task: impl FnOnce(mpsc::Sender<String>) -> std::result::Result<String, String> + Send + 'static,
    ) 
    {
        self.task_manager.start_task(id.clone(), move |sender| task(sender));
    }

    pub fn wait_task(&self, id: &Uuid) -> Option<String> {
        self.task_manager.wait_output(id).map(|x| x.join("\n"))
    }

    pub fn task_status(&self, id: &Uuid) -> TaskStatus<String, String> {
        self.task_manager.task_status(id)
    }
}
