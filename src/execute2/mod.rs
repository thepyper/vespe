//! # `execute2` Module: Execution Engine for Directive-Driven Text Processing
//!
//! This module provides the core execution engine responsible for processing text documents
//! that contain special directives, referred to as "tags" (e.g., `@include`, `@answer`)
//! and "anchors" (e.g., `<!-- @@answer...@@ -->`). Its primary goal is to resolve these
//! directives, interact with the file system, manage execution state, and orchestrate
//! calls to external models (LLMs) to produce a structured `ModelContent` output.
//!
//! The engine operates on a multi-pass execution strategy, iteratively processing the
//! document until all dynamic directives are resolved and no further modifications
//! to the source files are required.
//!
//! ## Key Components and Their Roles:
//!
//! - **`execute.rs`**: Contains the main execution logic, including the public entry points
//!   `execute_context` (for full execution with file modifications) and `collect_context`
//!   (for read-only execution). It defines the `Worker` (a stateless executor) and the
//!   `Collector` (which manages the execution state and accumulates `ModelContent`).
//!   The multi-pass mechanism, which re-evaluates the document after dynamic tag resolution
//!   or content injection, is central to this file.
//!
//! - **`content.rs`**: Defines the data structures, primarily `ModelContent` and
//!   `ModelContentItem` (`System`, `User`, `Agent`), which represent the structured
//!   prompt or conversation built during the execution process. This is the final
//!   output format consumed by external models.
//!
//! - **`tags.rs`**: Establishes the framework for handling different types of tags and anchors.
//!   It defines the `TagBehavior` trait, which all tag implementations must adhere to,
//!   and `TagBehaviorDispatch` for dynamically selecting the correct tag handler.
//!
//! - **`tag_*.rs` (e.g., `tag_answer.rs`, `tag_include.rs`)**: Implementations of specific
//!   tag behaviors. These are categorized into:
//!     - **Static Tags** (`@include`, `@set`, `@forget`, `@comment`): Processed in a single pass,
//!       directly modifying the `Collector`'s state or content. `@comment` tags are ignored.
//!     - **Dynamic Tags** (`@answer`, `@repeat`, `@inline`): Involve a state machine and can trigger
//!       multiple execution passes. They transform into anchors (`<!-- @@...@@ -->`)
//!       and manage their state (e.g., `JustCreated`, `NeedProcessing`, `NeedInjection`, `Completed`)
//!       persisted in external JSON files. These tags can involve calling external models
//!       and injecting their responses back into the source document.
//!
//! - **`error.rs`**: Defines custom error types specific to the execution engine.
//!
//! - **`utils.rs`**: Provides utility functions used across the module, such as path resolution
//!   and file system interactions.
//!
//! ## Execution Flow Overview:
//!
//! The engine processes a document by identifying tags and anchors. Static tags are
//! resolved immediately. Dynamic tags initiate a state-driven process:
//! 1.  An initial dynamic tag (e.g., `@answer`, `@inline`) is converted into an anchor (`<!-- @@...@@ -->`).
//! 2.  The anchor's state progresses (e.g., `NeedProcessing`), potentially triggering
//!     an external model call.
//! 3.  The model's response is then injected back into the document (state `NeedInjection`).
//! 4.  This injection necessitates a new execution pass to re-evaluate the modified document.
//! 5.  The cycle continues until all dynamic tags reach a `Completed` state or no further
//!     modifications are needed.
//!
//! The `readonly` flag (used by `collect_context`) prevents file modifications and new model
//! calls, allowing for state inspection without altering the document or triggering new AI responses.
//!
//! This module is crucial for enabling an interactive, AI-driven document generation and
//! modification workflow.

mod content;
mod error;
mod execute;
mod tag_answer;
mod tag_comment;
mod tag_forget;
mod tag_include;
mod tag_inline;
mod tag_repeat;
mod tag_set;
mod tag_task;
mod tags;
mod utils;

pub use self::error::{ExecuteError, Result};
pub use content::{ModelContent, ModelContentItem};

pub use execute::collect_context;
pub use execute::execute_context;

const TASK_ANCHOR_PLACEHOLDER : &str = "Content here has been used, so has been removed as it is no more useful.";
const REDIRECTED_OUTPUT_PLACEHOLDER : &str = "Context here has been answered but output has been redirected, so do not respond anymore to context above this sentence.\n";
const CHOICE_TEMPLATE : &str = "You MUST reply with ONLY ONE of the following choices: {{{choices}}}.\nYou MUST represent these in your output with ONLY ONE of the following tags {{{choice_tags}}}.\n";
const NO_CHOICE_MESSAGE: &str = "No choice was taken.";
const MANY_CHOICES_MESSAGE: &str = "Many choices were taken.";
