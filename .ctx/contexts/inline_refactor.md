@include ctx

# inline_refactor

Abbiamo fatto una nuova feature, te la riassumo:

@summary prompt

E' venuta abbastanza bene, ma ha un problema:
- @inline correttamente include inline lo snippet
- tuttavia, nel file markdown viene sostituita la riga @inline dalla risposta del llm solamente,
  e non anche dello snippet;

praticamente quello che voglio e' che lo snippet venga di fatto incluso nel file richiamante,
e che quel file venga effettivamente modificato.

Come modifica mi viene in mente un cambiamento della logica di execute_context (in project.rs);
Ora la funzione riscrive il file solamente se trova un tag @answer.

La funzione potrebbe invece ricrivere SEMPRE il file, in questo modo:
- se trova linee Text, le copia cosi come sono;
- se trova una linea Inline, processa il file incluso ugualmente con execute, partendo dal contesto presente al momento dell'inlining;
- alla prima linea Answer che trova (anche se trovata durante l'inlining) lancia la chiamata llm per sostituirla con la risposta;
- ogni successiva linea Answer la copia cosi come e';

Praticamente "rigenera" il file ogni volta, facendo delle operazioni, ovvero:
- processando tutti gli @inline
- processando la prima @answer (anche trovata dopo inline)

Oppure, 2-pass:
- 1 pass processa gli inline e li sostituisce con i contenuti (operando sempre a livello di Vec<Line>)
- 2 pass processa copia tutto fino alla prima answer, poi copia tutto comprese le altre answer

La scelta piu' pulita? future-proof?

Okay, I understand the problem with the `@inline` and `@answer` tags. The goal is to ensure that when `@inline` is used, the content of the inlined snippet is actually written back into the calling file, and that only the *first* `@answer` tag (even if it appears within an inlined snippet) triggers an LLM call, with subsequent `@answer` tags being copied as-is.

You've proposed two approaches: a single-pass "regenerate" method and a two-pass method. To properly evaluate these and suggest the cleaner, more future-proof option, I need to understand the current implementation of `execute_context` and how `Text`, `Inline`, and `Answer` lines are currently processed.

Let's start by listing the files in `src/bin/ctx/` to get an overview of the relevant source code.Alright, `project.rs` and `ast.rs` appear to be key. I'll examine `project.rs` first for `execute_context`'s current logic, then `ast.rs` for the `Line` enum.`project.rs` confirms `execute_context` processes `@inline` during `compose` but only replaces the `@answer` line in its original file, ignoring inlined content. Next, I'll examine `ast.rs` to understand `Line` and `LineData` enums, particularly `Inline`, to inform the two-pass approach.Okay, I've analyzed the current `execute_context` and `ast.rs`. The existing `execute_context` only replaces the `@answer` line in its original file, not incorporating inlined content.

Based on your desire to "regenerate" the file, including inlined snippets and processing only the first `@answer` (even if inlined), I propose a **Revised Single-Pass Recursive Approach**.

**Plan:**

1.  **Refactor `Project::execute_context`:**
    *   This will become the public entry point.
    *   It will resolve the initial context's path.
    *   It will initialize a `HashSet<PathBuf>` for `visited_contexts` (to prevent circular dependencies) and a `bool` flag `first_answer_found_globally = false`.
    *   It will call a new private helper function, `_process_context_recursive`, passing these mutable references.
    *   It will take the `Vec<String>` returned by `_process_context_recursive` and write it back to the original file of the initial context.

2.  **Create `Project::_process_context_recursive` (private helper function):**
    *   **Signature:** `fn _process_context_recursive(&self, name: &str, agent: &dyn AgentCall, visited_contexts: &mut HashSet<PathBuf>, first_answer_found_globally: &mut bool) -> Result<Vec<String>>`
    *   **Path Resolution:** Resolve the `PathBuf` for the given `name` (context or snippet).
    *   **Circular Dependency Check:** If the current path is already in `visited_contexts`, return an empty `Vec<String>` to prevent infinite loops. Otherwise, insert the current path.
    *   **Read and Parse:** Read the file content and parse it into `Vec<Line>` using `ContextAstNode::parse`.
    *   **Initialize Buffers:**
        *   `output_lines_for_current_file: Vec<String>`: Accumulates processed lines for the current context/snippet.
        *   `prompt_content_for_llm: String`: Accumulates text for the LLM call *only if* `first_answer_found_globally` is `false`.
    *   **Iterate and Process Lines:** Loop through the parsed `lines`:
        *   **`LineData::Text(text)`:** Add `text` to `output_lines_for_current_file`. If `!*first_answer_found_globally`, add `text` to `prompt_content_for_llm`.
        *   **`LineData::Include { context_name }`:** Recursively call `_process_context_recursive` for `context_name`. Append the returned `Vec<String>` to `output_lines_for_current_file`. If `!*first_answer_found_globally`, append the returned content to `prompt_content_for_llm`.
        *   **`LineData::Inline { snippet_name }`:** Recursively call `_process_context_recursive` for `snippet_name`. Append the returned `Vec<String>` to `output_lines_for_current_file`. If `!*first_answer_found_globally`, append the returned content to `prompt_content_for_llm`.
        *   **`LineData::Answer`:**
            *   If `!*first_answer_found_globally`: Set `*first_answer_found_globally = true`. Call the LLM with `prompt_content_for_llm`. Append the LLM response to `output_lines_for_current_file`.
            *   Else: Append the literal `@answer` tag to `output_lines_for_current_file`.
        *   **`LineData::Summary { context_name }`:** Call `_handle_summary_tag` to get the summary. Append the summary to `output_lines_for_current_file`. If `!*first_answer_found_globally`, append the summary to `prompt_content_for_llm`.
    *   **Return:** Return `Ok(output_lines_for_current_file)`.

3.  **Adjust `Project::compose`:** This function's role in inlining will be superseded by `_process_context_recursive`. It might need to be adjusted or become redundant.

4.  **Error Handling:** Implement robust error handling throughout.

This plan directly addresses your requirements for regenerating the file with inlined content and processing only the first `@answer`.

Do you approve this plan?

