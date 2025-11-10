# ctx: Your Collaborative Mind-Mapping Tool with LLMs

`ctx` is a powerful command-line interface (CLI) tool designed to facilitate a seamless, collaborative mind-mapping experience between users and Large Language Models (LLMs). It enables you to co-create, refine, and expand your ideas by treating your documents as dynamic, interactive canvases.

## How it Works

At its core, `ctx` operates on a collection of textual documents, referred to as "contexts." These documents are managed within the `.ctx` folder, which acts as a sidecar to your main project. These are standard Markdown files augmented with special custom commands (tags) that allow for direct interaction with LLMs and dynamic content generation. This approach transforms static documents into living, evolving knowledge bases.

## Getting Started

here is a minimal example
create a project:

>ctx project init

then create a context:

>ctx context create hello

then edit context, write:

>Tell me something nice!
>@answer { provider: "gemini -y -m gemini-2.5-flash" }

you obtain:

>Tell me something nice!
><!-- answer-a98dc897-1e4b-4361-b530-5c602f358cef:begin { provider: "gemini -y -m gemini-2.5-flash" } -->
>You are doing great!
><!-- answer-a98dc897-1e4b-4361-b530-5c602f358cef:end -->

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