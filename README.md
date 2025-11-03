# ctx: Your Collaborative Mind-Mapping Tool with LLMs

`ctx` is a powerful command-line interface (CLI) tool designed to facilitate a seamless, collaborative mind-mapping experience between users and Large Language Models (LLMs). It enables you to co-create, refine, and expand your ideas by treating your documents as dynamic, interactive canvases.

## How it Works

At its core, `ctx` operates on a collection of textual documents, referred to as "contexts." These are standard Markdown files augmented with special custom commands (tags) that allow for direct interaction with LLMs and dynamic content generation. This approach transforms static documents into living, evolving knowledge bases.

## Practical Examples

*(Space reserved for practical examples. You can add snippets demonstrating how to use `ctx` for brainstorming, drafting, or refining content with LLMs.)*

## Core Commands (Tags)

`ctx` extends Markdown with a set of intuitive tags that control its behavior and interaction with LLMs:

*   **`@include <file_path>`**: Incorporates content from another context file, allowing for modular and organized documents.
*   **`@set <variable_name>=<value>`**: Defines and assigns values to variables within your context, enabling dynamic content and configuration. This tag updates the local variables of the current context.
*   **`@answer <query>`**: Sends a query to a configured LLM and injects its response directly into your document. This is the primary mechanism for AI collaboration, managing the state of the interaction through multiple passes.
*   **`@repeat`**: (Further details on usage to be added)
*   **`@decide`**: (Further details on usage to be added)
*   **`@choose`**: (Further details on usage to be added)
*   **`@derive`**: (Further details on usage to be added)

More tags are planned and will be documented here as they are implemented.

By leveraging these commands, you can guide LLMs to generate content, summarize information, answer questions, and much more, all while maintaining a clear, human-readable document structure.