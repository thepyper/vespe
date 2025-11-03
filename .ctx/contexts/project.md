

Write a summary about the project.
Gather information by reading src/ast2/* and src/execute2/* and src/bin/ctx/main.rs.
Explain me the project as you understand it.

<!-- answer-1bd7a29c-939e-4c63-a385-af3bf1fa44f1:begin { system: agent/gemini_25_pro_yolo }  -->

The `vespe` project is a command-line interface (CLI) tool named `ctx` designed for managing and executing "contexts," which are structured documents. It facilitates dynamic, AI-assisted document generation or processing workflows.

The project is composed of three main parts:

1.  **`src/ast2` (Abstract Syntax Tree Module):** This module is responsible for defining and parsing a custom template language. It converts source documents into a typed Abstract Syntax Tree (AST). Key elements of this language include:
    *   **Tags:** Commands prefixed with `@` (e.g., `@include`, `@answer`, `@set`) that can have parameters and arguments.
    *   **Anchors:** HTML-style comments (`<!-- ... -->`) used for commands that encapsulate a block of content and are paired (Begin/End) using a UUID.
    *   **JSON-like Structures:** A custom `JsonPlusEntity` and `JsonPlusObject` system for handling parameters, supporting various data types including flags, booleans, numbers, and different string formats.
    *   It also includes robust error reporting with precise `Position` and `Range` tracking.

2.  **`src/execute2` (Execution Engine Module):** This module interprets and executes the AST. It employs a multi-pass execution strategy to:
    *   **Resolve Dependencies:** Handle `@include` tags by incorporating content from other context files.
    *   **Execute Commands:** Process various tags and anchors. "Static" commands (like `@include`, `@set`) execute immediately, while "dynamic" commands (like `@answer`, `@repeat`) manage persistent state across multiple execution passes.
    *   **Interact with External Models:** Commands like `@answer` (and potentially `@decide`, `@choose`, `@derive`) interact with external AI models (likely Large Language Models) via a shell interface (`agent::shell::shell_call`).
    *   **Dynamically Modify Documents:** Inject results from model interactions or other command executions back into the context files, often by converting tags into anchors and updating their content.
    *   **State Management:** Dynamic commands save and load their execution state (e.g., `AnswerState`) to/from `state.json` files within a `.ctx/.meta` directory, enabling iterative and persistent workflows.
    *   **Variables and Redirection:** Manages execution-time variables (e.g., `provider`, `system` prompts) and supports redirecting command input from or output to other context files.

3.  **`src/bin/ctx/main.rs` (CLI Application):** This is the main entry point for the user. It provides commands to:
    *   `init`: Initialize a new `.ctx` project.
    *   `context new`: Create new context files, optionally using Handlebars templates and automatic naming (e.g., `diary/YYYY-mm-DD`).
    *   `context execute`: Run the execution engine on a specified context.
    *   `context list`: List all available contexts.
    *   `watch`: Monitor context files for changes and trigger re-execution.

In summary, `vespe` provides a powerful framework for defining and automating complex document-centric workflows, especially those involving interaction with AI models, by combining a structured document format with a sophisticated, state-aware execution engine.
<!-- answer-1bd7a29c-939e-4c63-a385-af3bf1fa44f1:end {}  -->
