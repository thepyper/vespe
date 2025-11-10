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

## Core Tags

`ctx` provides a set of powerful tags to control the interaction with LLMs and manage your content.

### @answer

The `@answer` tag is the primary way to interact with an LLM. It sends the content preceding it as a prompt and injects the model's response into the document.

**Usage:**
```markdown
What is the capital of France?

@answer { provider: "gemini -y" }
```

**Dynamic Answers:**
You can make an answer dynamic, so it automatically updates if the preceding context changes.

```markdown
@answer { provider: "gemini -y", dynamic: true }
```

**Multiple Choices:**
Force the LLM to choose from a predefined set of options. `ctx` will insert the content associated with the chosen option.

```markdown
What is the best programming language?

@answer {
  provider: "gemini -y",
  choose: {
    "Rust": "Rust is the best because of its safety and performance.",
    "Python": "Python is the best for its simplicity and vast libraries."
  }
}
```

### @include

The `@include` tag statically inserts the content of another context file. This is useful for reusing prompts or structuring complex contexts.

**Usage:**
```markdown
@include "my_common_prompts/preamble.md"

Now, do something specific.

@answer { provider: "gemini -y" }
```

You can also pass data to the included file, which can be used for templating with Handlebars syntax.

**`data-example.md`:**
```markdown
Hello, {{name}}!
```

**`main.md`:**
```markdown
@include "data-example.md" { data: { name: "World" } }
```
This will resolve to "Hello, World!".

### @inline

The `@inline` tag dynamically includes content from another file. Unlike `@include`, this creates a dynamic anchor that can be re-executed, for example, by a `@repeat` tag. This is useful for content that needs to be refreshed.

**Usage:**
```markdown
@inline "path/to/dynamic_content.md"
```

Like `@include`, it also supports passing `data` for templating.

### @repeat

The `@repeat` tag forces the re-execution of the dynamic anchor it is placed within (like `@answer` or `@inline`). Context will be re-read so any correction to query can be made. The repeated tag will inherith parameters from the @repeat tag.

**Usage:**
```markdown
<!-- answer-some-uuid:begin { provider: "gemini -y" } -->
Initial answer from the model.

@repeat { provider: "gemini -m gemini-2.5-flash" }
<!-- answer-some-uuid:end -->
```
In the next run, the `@answer` block will be executed again, with different parameters and re-reading the surrounding context.

`@repeat` can also modify the parameters of the anchor it is repeating.

### @set

The `@set` tag defines default parameters for all subsequent tags in the current context. This helps to avoid repetition.

**Usage:**
```markdown
@set { provider: "gemini -y -m gemini-2.5-pro" }

What is the meaning of life?
@answer
<!-- This will use the provider set above -->

Tell me a joke.
@answer
<!-- This will also use the same provider -->
```

### @forget

The `@forget` tag clears all the context that came before it. This is like starting a fresh conversation with the LLM within the same file.

**Usage:**
```markdown
--- First Conversation ---
Prompt for the first question.
@answer { provider: "gemini -y" }

@forget

--- Second Conversation ---
This prompt is sent without the context of the first one.
@answer { provider: "gemini -y" }
```

### @comment

The `@comment` tag is used to add comments within your context files. The content of this tag is completely ignored by the `ctx` engine and is not sent to the LLM.

**Usage:**
```markdown
@comment {
  This is a note for myself.
  The LLM will not see this.
}
```


