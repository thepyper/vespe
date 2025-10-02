@include project

# summary

Stiamo lavorando in src/bin/ctx al binario ctx che e' il tool principale che stiamo costruendo adesso. Leggi bene i sorgenti e Cargo.toml per avere del contesto.

Ok, abbiamo un prototipo funzionante.

La prossima feature che vorrei aggiungere e' la seguente:

- un tag @summary che funzioni in modo simile a @include, solo che:
	- crei la composizione del context incluso, poi la riassuma
	  (chiamando un llm), poi includa questo riassunto e non il contesto intero
	  
- vorrei che fosse memoizzata la cosa, ovvero che alla chiamata di summary 
   venga salvato un file (ad esempio, se ho bla.md il summary salvarlo in bla.md.summary)
  dovrebbe venir salvata anche una hash del file originale, in modo da rilevare
  cambiamento e rifare il summary in questo caso.
  
Chiaro?
Dubbi?
Alternative?
Overengineering?

Okay, I understand the new feature request for the `ctx` binary: a `@summary` tag that includes an LLM-generated, memoized summary of a referenced context.

Before I dive into implementation, I need to familiarize myself with the existing codebase, specifically how context composition and the `@include` tag are currently handled.

Here's my plan:
1.  **Read `Cargo.toml`**: To understand the project's dependencies and overall structure.
2.  **Read `src/bin/ctx` directory**: To understand the current implementation of context processing, especially how `@include` works, and identify the best place to integrate the new `@summary` logic.

I'll start by reading `Cargo.toml`.Alright, `Cargo.toml` shows `test_ctx` as a binary, which I'll assume is the `ctx` tool we're focusing on. Dependencies like `markdown`, `tokio`, `serde`, and `clap` are present, with `genai` and `sha2` commented out, hinting at future LLM and hashing needs.

Next, I'll examine `src/bin/test_ctx.rs` to understand its current context composition and `@include` implementation.Okay, I've analyzed `src/bin/test_ctx.rs` and understand the existing context handling, including the `@include` tag and circular dependency prevention.

My plan for the `@summary` feature is as follows:

1.  **`@summary` Tag Handling**: I'll modify `compose_recursive` to detect `@summary` lines.
    *   When found, it will first recursively compose the referenced context.
    *   Then, it will calculate a hash of this composed content.
    *   It will check for an existing summary file (`original_file.md.summary`) and compare its stored hash with the new one.
    *   If the hashes match and a summary exists, the cached summary will be used.
    *   Otherwise, an LLM call will be made to summarize the composed content, and the new summary (along with its hash) will be saved to `original_file.md.summary`.
    *   Finally, the summary will be inserted into the output.

2.  **LLM Integration**: I'll need to add an LLM client dependency. `genai` (currently commented out in `Cargo.toml`) seems like a good candidate.

3.  **Hashing**: I'll add a `sha2` dependency (also commented out in `Cargo.toml`) for content hashing.

4.  **Summary File Naming**: Summaries will be stored as `original_file.md.summary`.

**Before I proceed, I need your input on the following:**

*   **LLM Choice**: Is Google's Gemini API (via the `genai` crate) the preferred LLM for summarization?
*   **LLM API Key/Configuration**: How should the LLM API key and any other configuration (e.g., model name, temperature) be managed? Environment variables or a configuration file?
*   **LLM Error Handling**: What should happen if the LLM call fails? Should it fall back to including the full content, or should it return an error?
*   **Summary Length/Style**: Are there any specific requirements for the summary's length or style (e.g., concise, bullet points, paragraph)?
*   **`tree` command**: For now, I'll assume `@summary` in the `tree` command should behave like `@include` and just show the reference. Please confirm if this is acceptable.

Does this plan align with your expectations?



