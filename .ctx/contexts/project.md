

Write a summary about the project.
Gather information by reading src/ast2/* and src/execute2/* and src/bin/ctx/main.rs.
Explain me the project as you understand it.

<!-- answer-d76e1844-7125-4155-86ab-484a89f85fbc:begin { system: agent/gemini_25_pro_yolo }  -->

The `vespe` project is a Rust-based command-line interface (CLI) tool named `ctx` designed for managing and executing dynamic text "contexts."

At its core, `ctx` parses special text files (contexts) that contain custom "tags" (e.g., `@include`, `@set`, `@answer`) and "anchors" (HTML-like comments, `<!-- ... -->`). These elements drive an execution engine that can:

*   **Include other files:** `@include`
*   **Set variables:** `@set`
*   **Interact with external AI models:** Commands like `@answer` send queries to models (configured via a `provider`) and inject the responses back into the document. The system manages the state of these multi-step interactions.
*   **Control flow:** Other commands like `@repeat`, `@decide`, `@choose`, and `@derive` suggest advanced logic for conditional processing and content transformation.

The `ctx` CLI provides commands to `init`ialize projects, `new` contexts (with templating support), `execute` contexts, `list` them, and even `watch` for changes to automatically re-process files. The project relies on a custom Abstract Syntax Tree (AST) for parsing and a sophisticated multi-pass execution strategy to handle dynamic content generation and integration with external services.
<!-- answer-d76e1844-7125-4155-86ab-484a89f85fbc:end {}  -->
