# ctx: Your Collaborative Mind-Mapping Tool with LLMs

`ctx` is a powerful command-line interface (CLI) tool designed to facilitate a seamless, collaborative mind-mapping experience between users and Large Language Models (LLMs). It enables you to co-create, refine, and expand your ideas by treating your documents as dynamic, interactive canvases.

## How it Works

At its core, `ctx` operates on a collection of textual documents, referred to as "contexts." These documents are managed within the `.ctx` folder, which acts as a sidecar to your main project. These are standard Markdown files augmented with special custom commands (tags) that allow for direct interaction with LLMs and dynamic content generation. This approach transforms static documents into living, evolving knowledge bases.

## Getting Started: A Quick Glimpse

Let's dive right in with a simple example.

1.  **Initialize your project:**

    Open your terminal and run:

    ```shell
    ctx project init
    ```

    This creates a `.ctx` directory in your project, where all your contexts will be stored.

2.  **Create your first context:**

    A "context" is just a Markdown file where you can interact with the AI. Let's create one called `hello`:

    ```shell
    ctx context create hello
    ```

    This will create a file named `hello.md` inside the `.ctx/contexts` directory.

3.  **Add a prompt and an AI command:**

    Open `hello.md` in your favorite editor and add the following lines:

    ```markdown
    Tell me something nice!

    @answer { provider: "gemini -y -m gemini-2.5-flash" }
    ```

4.  **Run `ctx` to get a response:**

    Now, execute the context:

    ```shell
    ctx run hello
    ```

    `ctx` will process the file, send the prompt to the Gemini model, and inject the answer directly into your `hello.md` file. It will look something like this:

    ```markdown
    Tell me something nice!

    <!-- answer-a98dc897-1e4b-4361-b530-5c602f358cef:begin { provider: "gemini -y -m gemini-2.5-flash" } -->
    You are an amazing person, and you're capable of achieving wonderful things!
    <!-- answer-a98dc897-1e4b-4361-b530-5c602f358cef:end -->
    ```

And that's it! You've just used `ctx` to collaborate with an LLM on a document.

## Tag Syntax

The power of `ctx` lies in its simple yet powerful tag syntax. Each tag follows a consistent structure that is easy to read and write.

The general syntax for a tag is:

```
@tag_name {key1: "value1", key2: value2} positional_argument
```

Let's break it down:

*   **`@tag_name`**: This is the command you want to execute (e.g., `@answer`, `@include`). It always starts with an `@` symbol.
*   **`{...}` (Parameters)**: This is a JSON-like object containing key-value pairs that configure the tag's behavior. `ctx` uses a more flexible version of JSON for convenience:
    *   Quotes around keys are optional (e.g., `provider` is the same as `"provider"`).
    *   Quotes around string values are optional if the value doesn't contain spaces or special characters.
*   **Positional Arguments**: Some tags can also take additional arguments after the parameter block. For example, `@include` takes a file path.

### Example

Consider the `@answer` tag from our "Getting Started" example:

```markdown
@answer { provider: "gemini -y -m gemini-2.5-flash" }
```

*   **Tag Name**: `@answer`
*   **Parameters**: `{ provider: "gemini -y -m gemini-2.5-flash" }`
    *   The key is `provider`.
    *   The value is `"gemini -y -m gemini-2.5-flash"`. In this case, quotes are necessary because the value contains spaces.

This flexible syntax makes writing `ctx` commands feel natural and unobtrusive within your Markdown files.

## Core Commands (Tags)

Overview dei comandi

@answer - chiama llm per rispondere al contesto sovrastante (uso di default)
@include - include un contesto
@inline - copia ed incolla nel contesto attuale un altro contesto
@forget - dimentica tutto il contesto sovrastante
@set - imposta parametri di default per i tag successivi
@repeat - ripete la anchor in cui e' contenuto copiando i propri parametri a quelli dell'ancora
@comment - no-op per scrivere commenti non visti da llm


