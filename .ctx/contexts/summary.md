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

Okay, I understand the new feature request for the `ctx` binary: implementing a `@summary` tag that summarizes included content via an LLM and memoizes the results.

Before I propose a plan, I need to understand the current implementation of the `ctx` binary, especially how it handles `@include` tags and file processing. I'll start by examining `Cargo.toml` and then `src/bin/ctx/main.rs`.

Alright, `Cargo.toml` shows a Rust project with core dependencies like `anyhow`, `thiserror`, `markdown`, `tokio`, `serde`, `serde_json`, `uuid`, `clap`, `dirs`, `notify`, and `ctrlc`. `genai`, `sha2`, and `walkdir` are commented out, indicating LLM integration and memoization are not yet implemented.

Next, I'll examine `src/bin/ctx/main.rs` to understand the existing context composition.I've analyzed `src/bin/ctx/main.rs` and understand the `ctx` binary's structure, context handling, and command functionalities. The `Project` struct and its `compose` method are central to context resolution, and `execute_context` likely handles LLM interaction. The `Watch` command is relevant for memoization.

My plan for the `@summary` feature is:

1.  **Analyze Existing Context Resolution:** Deep dive into `src/project.rs` and `src/context.rs` to understand `@include` parsing and resolution.
2.  **Introduce LLM Integration:** Uncomment `genai` in `Cargo.toml` and create an `llm_client.rs` module for LLM API calls and prompt formatting.
3.  **Implement `@summary` Tag Parsing:** Modify context parsing to recognize `@summary` tags and identify referenced files.
4.  **Implement Memoization:** Create a `.ctx/summaries` directory. Store summarized content and a hash of the *original* context file's content (requiring `sha2` and `walkdir` dependencies). Use cached summaries if the hash matches.
5.  **Integrate Summarization into `compose`:** When `@summary` is encountered, recursively compose the referenced context, check for a cached summary, generate a new summary via LLM if needed, store it, and replace the tag with the summary.
6.  **Error Handling:** Implement robust error handling for all new components.
7.  **Testing:** Add unit tests for parsing, memoization, and summarization.

**Doubts/Considerations:**

*   **LLM Choice/Configuration:** Assuming Google Gemini API; API key configuration needed.
*   **Summarization Prompt:** Critical for summary quality; needs a good default and potential customization.
*   **Cost of LLM Calls:** Memoization is essential to mitigate this.
*   **Composed Context Complexity:** Recursive handling of nested `@include` and `@summary` tags.
*   **Hash Granularity:** Hashing the *original* file content for memoization seems most straightforward.
*   **File Naming:** `bla.md.summary` for cached summaries.

**No significant overengineering or alternative considered at this stage.**

Do you approve this plan, or have any feedback on LLM integration or memoization?

Rispondo ai dubbi:
1) niente genai, usiamo una logica tipo execute_context che chiama llm con shell / piping.
2) prompt summarization: per ora hardcoded, in un template handlebars.
3) recursive handling e' gia' fatto in project.rs, con compose()
4) dove salvi la hash? bla.md.summary puo' essere qualsiasi formato, anche json se vuoi aggiungere metadata. oppure proponi

Okay, I've incorporated your feedback into the plan.

**Revised Plan:**

1.  **Analyze Existing Context Resolution:** Deep dive into `src/project.rs` and `src/context.rs` to understand ` @include` parsing and resolution.
2.  **Identify LLM Shell Command Integration Point:** Determine how `execute_context` or a similar mechanism can be used to invoke an external LLM via a shell command, passing the content to be summarized and receiving the summary. This will likely involve defining a specific shell command format (e.g., `ollama run llama3 "Summarize this: <content>" `).
3.  **Implement ` @summary` Tag Parsing:** Modify context parsing (likely in `src/project.rs` or `src/context.rs`) to recognize ` @summary` tags and identify referenced files.
4.  **Implement Memoization:**
    *   Create a `.ctx/summaries` directory (or similar, based on project conventions).
    *   Use `sha2` to calculate the hash of the *original* content of the file being summarized.
    *   Store the summarized content and the original file's hash in a JSON file (e.g., `bla.md.summary`) within the `.ctx/summaries` directory. The JSON structure will be:
        ```json
        {
          "original_hash": "sha256_hash_of_original_file",
          "summary_content": "This is the summarized content."
        }
        ```
    *   When a ` @summary` tag is encountered, check if a corresponding `.summary` file exists and if its stored hash matches the current hash of the original file. If they match, use the cached summary; otherwise, generate a new one.
5.  **Integrate Summarization into `compose`:**
    *   Within the `compose()` function in `src/project.rs`, when a ` @summary` tag is encountered:
        *   Recursively compose the referenced context.
        *   Calculate the hash of the composed content.
        *   Check for a cached summary.
        *   If no valid cache, render the composed content into a Handlebars template with a hardcoded summarization prompt.
        *   Execute a shell command (via `run_shell_command` or similar internal mechanism) to pass the templated prompt to an LLM and capture its output (the summary).
        *   Store the new summary and hash in the `.summary` file.
        *   Replace the ` @summary` tag with the generated/cached summary.
6.  **Error Handling:** Implement robust error handling for all new components, especially LLM calls and file operations.
7.  **Testing:** Add unit tests for parsing, memoization, and summarization.

Do you approve this revised plan?

Mi sembra un buon piano, un po astratto.
Ora scrivi un piano operativo, leggi i files di cui hai bisogno, e delinea un piano chiaro, completo e dettagliato.

Okay, I will create a detailed operational plan. First, I need to read the relevant files to understand the current implementation.

My apologies, it seems I made an error in the file paths for `project.rs` and `context.rs`. They are likely modules within `src/lib.rs`. I will read `src/lib.rs` to determine their correct locations and then read those files.

The `src/lib.rs` is empty, which is unusual given the `mod context;` and `mod project;` statements in `src/bin/ctx/main.rs`. This suggests `context.rs` and `project.rs` are likely located within the `src/bin/ctx/` directory, as `main.rs` would resolve them relative to its own location. I will attempt to read those files from `src/bin/ctx/`.Okay, I have a clear understanding of the project structure and the relevant files. Here's the detailed operational plan for implementing the `@summary` feature:

**Operational Plan for `@summary` Feature**

**Phase 1: Setup and Dependencies**

1.  **Modify `Cargo.toml`**:
    *   **Action**: Uncomment `sha2 = "0.10"` and `handlebars = "5.1.0"`. Add `walkdir = "2.3"` and `serde_json = "1"` (already present, but ensure it's enabled for `derive` feature).
    *   **File**: `c:/Proj/xxx/025_006.Vespe/vespe/Cargo.toml`
    *   **Reason**: `sha2` is needed for hashing file content for memoization. `handlebars` is for templating the LLM prompt. `walkdir` is for potential future use, but `serde_json` is crucial for reading/writing cached summaries.

**Phase 2: `context.rs` Modifications**

1.  **Update `LineData` Enum**:
    *   **Action**: Add a new variant `Summary { context_name: String }` to the `LineData` enum.
    *   **File**: `c:/Proj/xxx/025_006.Vespe/vespe/src/bin/ctx/context.rs`
    *   **Code Snippet**:
        ```rust
        // In src/bin/ctx/context.rs
        pub enum LineData {
            Include { context_name: String },
            Answer,
            Summary { context_name: String }, // New variant
            Text(String),
        }
        ```
    *   **Reason**: To represent the new `@summary` tag in the parsed context.

2.  **Update `Context::parse` Function**:
    *   **Action**: Add logic to `Context::parse` to recognize `@summary` tags, similar to how `@include` is handled.
    *   **File**: `c:/Proj/xxx/025_006.Vespe/vespe/src/bin/ctx/context.rs`
    *   **Code Snippet**:
        ```rust
        // In src/bin/ctx/context.rs, inside impl Context for parse function
        // ...
        } else if let Some(context_name) = line.strip_prefix("@summary ") {
            Line {
                data: LineData::Summary {
                    context_name: context_name.trim().to_string(),
                },
                source_file: file_path.clone(),
                source_line_number: line_number,
            }
        } else if line.trim() == "@answer" {
        // ... (existing @answer handling)
        ```
    *   **Reason**: To correctly parse the new `@summary` tag from context files.

**Phase 3: `project.rs` Modifications (Core Logic)**

1.  **Add `summaries_dir` Function**:
    *   **Action**: Create a new public function `summaries_dir(&self) -> Result<PathBuf>` in the `Project` struct to return the path to the `.ctx/summaries` directory.
    *   **File**: `c:/Proj/xxx/025_006.Vespe/vespe/src/bin/ctx/project.rs`
    *   **Code Snippet**:
        ```rust
        // In src/bin/ctx/project.rs, inside impl Project
        pub fn summaries_dir(&self) -> Result<PathBuf> {
            let path = self.root_path.join("summaries");
            std::fs::create_dir_all(&path)?;
            Ok(path)
        }
        ```
    *   **Reason**: To manage a dedicated directory for storing cached summaries.

2.  **Modify `compose_recursive` Function**:
    *   **Action**: Add a new `match` arm for `LineData::Summary` within the loop that processes `Line`s. This arm will handle hashing, memoization, LLM interaction, and replacement.
    *   **File**: `c:/Proj/xxx/025_006.Vespe/vespe/src/bin/ctx/project.rs`
    *   **Code Snippet (Conceptual - will be implemented in steps)**:
        ```rust
        // In src/bin/ctx/project.rs, inside compose_recursive function
        // ...
        for line in Context::parse(&content, path.to_path_buf()) {
            match line.data {
                LineData::Include { context_name } => {
                    // Existing logic
                }
                LineData::Summary { context_name } => {
                    let summary_target_path = self.resolve_context(&context_name)?;
                    let original_content_to_summarize = std::fs::read_to_string(&summary_target_path)?;
                    let original_hash = sha256::digest(original_content_to_summarize.as_bytes());

                    let summary_filename = format!("{}.summary", Context::to_filename(&context_name));
                    let summary_file_path = self.summaries_dir()?.join(summary_filename);

                    let mut summarized_text = String::new();
                    let mut cache_hit = false;

                    if summary_file_path.exists() {
                        let cached_summary_json = std::fs::read_to_string(&summary_file_path)?;
                        let cached_data: serde_json::Value = serde_json::from_str(&cached_summary_json)?;
                        if let (Some(cached_hash), Some(cached_content)) = (
                            cached_data["original_hash"].as_str(),
                            cached_data["summary_content"].as_str(),
                        ) {
                            if cached_hash == original_hash {
                                summarized_text = cached_content.to_string();
                                cache_hit = true;
                                println!("Using cached summary for {}", context_name);
                            }
                        }
                    }

                    if !cache_hit {
                        println!("Generating new summary for {}", context_name);
                        // Recursively compose the content to be summarized
                        let mut summary_visited = visited.clone(); // Clone visited for sub-composition
                        let lines_to_summarize = self.compose_recursive(&summary_target_path, &mut summary_visited)?;
                        let content_to_summarize: String = lines_to_summarize.into_iter()
                            .filter_map(|l| if let LineData::Text(t) = l.data { Some(t) } else { None })
                            .collect::<Vec<String>>()
                            .join("\\n");

                        // Use handlebars for templating the prompt
                        let mut handlebars = handlebars::Handlebars::new();
                        handlebars.register_template_string("summary_prompt", "Summarize the following content concisely:\\n\\n{{content}}")?;
                        let prompt_data = serde_json::json!({ "content": content_to_summarize });
                        let llm_prompt = handlebars.render("summary_prompt", &prompt_data)?;

                        summarized_text = self._execute_llm_command(llm_prompt)?;

                        // Save to cache
                        let cache_data = serde_json::json!({
                            "original_hash": original_hash,
                            "summary_content": summarized_text,
                        });
                        std::fs::write(&summary_file_path, serde_json::to_string_pretty(&cache_data)?)?;
                    }

                    composed_lines.push(Line {
                        data: LineData::Text(summarized_text),
                        source_file: line.source_file, // Keep original source file for traceability
                        source_line_number: line.source_line_number,
                    });
                }
                _ => {
                    composed_lines.push(line);
                }
            }
        }
        // ...
        ```
    *   **Reason**: This is the core logic for handling `@summary` tags, including recursive composition, hashing, caching, LLM interaction, and replacing the tag with the summary.

**Phase 4: Error Handling and Testing**

1.  **Robust Error Handling**:
    *   **Action**: Ensure all new file operations (`std::fs::read_to_string`, `std::fs::write`), JSON serialization/deserialization (`serde_json::from_str`, `serde_json::to_string_pretty`), and Handlebars operations have appropriate `anyhow::Context` messages for better error reporting.
    *   **File**: `c:/Proj/xxx/025_006.Vespe/vespe/src/bin/ctx/project.rs`
    *   **Reason**: To provide clear and actionable error messages to the user.

2.  **Unit Tests**:
    *   **Action**: Add unit tests to verify the functionality of the `@summary` tag.
    *   **File**: (New test file, e.g., `c:/Proj/xxx/025_006.Vespe/vespe/src/bin/ctx/tests/summary_tests.rs` or integrated into existing tests)
    *   **Test Cases**:
        *   Test parsing of `@summary` tags.
        *   Test that a summary is generated and correctly cached in the `.ctx/summaries` directory.
        *   Test that a cached summary is used when the original file content has not changed (hash matches).
        *   Test that a new summary is generated when the original file content *has* changed (hash mismatch).
        *   Test scenarios with nested `@summary` and `@include` tags to ensure correct recursive behavior.
        *   Test error conditions (e.g., referenced context for summary not found).
    *   **Reason**: To ensure the new feature works as expected and doesn't introduce regressions.

**Phase 5: Refinement (Future Considerations)**

1.  **Prompt Customization**: Consider making the summarization prompt configurable (e.g., via a command-line argument or a project configuration file).
2.  **LLM Model Selection**: Allow specifying the LLM model for summarization.

Do you approve this detailed operational plan?

Approvo. unico dettaglio: separa la funzionalita' di summary in una funzione separata che chiami da compose(), altrimenti compose() diventa troppo gigante.

@answer