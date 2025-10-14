@include project
@include agent/planner

Voglio aggiungere in src/execute/mod.rs un oggetto in ExecuteWorker, ovvero

variables: HashMap<String, String>   // insieme dei valori attuali delle variabili

Queste variabili possono variare durante il parsing del file context, ogni volta che viene incontrato un tag @set[variable=value].

Il tag @set va aggiunto alla gestione correttamente in src/syntax e in src/semantic.

Pianifica questo cambiamento, senza eseguirlo.

<!-- answer-6213b6ce-76eb-452a-9f6e-06dccbdbafc4:begin -->
Okay, I understand. You want to introduce a new feature that allows dynamic variable assignment within context files using an `@set[variable=value]` tag. This will require modifications to the `ExecuteWorker` struct, and the `syntax` and `semantic` modules to correctly parse and process this new tag.

Here's a detailed plan to implement this change:

### Plan: Implement `@set` Tag for Dynamic Variables

1.  **Understand Existing Codebase:**
    *   **Read `src/execute/mod.rs`:** Analyze the `ExecuteWorker` struct, its initialization, and how it processes context content. Identify the appropriate place to add and manage the `variables` `HashMap`.
    *   **Read `src/syntax/mod.rs`, `src/syntax/parser.rs`, `src/syntax/types.rs`:** Understand how existing tags are defined (e.g., `Tag` enum, `TagKind`), parsed (`parse_tag` function), and represented in the Abstract Syntax Tree (AST).
    *   **Read `src/semantic/mod.rs`, `src/semantic/context.rs`:** Examine how parsed tags are semantically analyzed and how their information is propagated for execution.

2.  **Modify `src/execute/mod.rs`:**
    *   **Add `variables` field to `ExecuteWorker`:** Introduce `variables: HashMap<String, String>` to the `ExecuteWorker` struct.
    *   **Initialize `variables`:** Ensure the `variables` `HashMap` is properly initialized (e.g., as an empty map) when a new `ExecuteWorker` instance is created.
    *   **Implement variable update logic:** Add a method or logic within `ExecuteWorker` that, upon encountering a `SetTag` (to be defined later), updates the `self.variables` `HashMap` with the extracted `variable` and `value`.

3.  **Modify `src/syntax/types.rs`:**
    *   **Define `SetTag` struct:** Create a new struct to represent the `@set` tag's data:
        ```rust
        pub struct SetTag {
            pub variable: String,
            pub value: String,
        }
        ```
    *   **Update `Tag` enum:** Add a new variant to the `Tag` enum to include the `SetTag`:
        ```rust
        pub enum Tag {
            // ... existing variants
            Set(SetTag),
        }
        ```

4.  **Modify `src/syntax/parser.rs`:**
    *   **Update `parse_tag` function:**
        *   Add a new `match` arm or `if let` condition to detect the "set" keyword after the `@` symbol.
        *   Parse the content within the square brackets `[variable=value]`. This will involve extracting the `variable` name and its `value`.
        *   Construct a `SetTag` instance with the parsed data.
        *   Return `Ok(Tag::Set(set_tag_instance))`.
        *   Ensure robust error handling for malformed `@set` tags (e.g., missing `=` or invalid format).

5.  **Modify `src/semantic/context.rs` (and potentially `src/semantic/mod.rs`):**
    *   **Integrate `SetTag` processing:** Update the semantic analysis logic to recognize and process the `Tag::Set` variant.
    *   **Propagate variable updates:** The semantic analysis phase should extract the `variable` and `value` from the `SetTag` and ensure this information is passed to the `ExecuteWorker` during the execution phase. This might involve:
        *   Modifying the `SemanticContext` to temporarily hold these variable assignments.
        *   Or, more likely, modifying the `execute_context` function (or similar) in `src/project.rs` to iterate through the semantically analyzed tags and apply the `SetTag` updates to the `ExecuteWorker`'s `variables` `HashMap`.

6.  **Testing:**
    *   **Create a new test case:** Add a new integration test (e.g., in `src/bin/ctx/main.rs`'s test module or a dedicated test file) that:
        *   Creates a temporary context file containing one or more `@set` tags.
        *   Executes this context file using the `ctx` CLI (or directly calls the `execute_context` function).
        *   Asserts that the `ExecuteWorker`'s `variables` `HashMap` contains the expected key-value pairs after execution.
    *   **Run existing tests:** Execute all existing tests to ensure that the changes haven't introduced any regressions.

7.  **Code Quality and Documentation:**
    *   **Run `cargo fmt`:** Ensure all new and modified code adheres to Rust formatting standards.
    *   **Run `cargo clippy`:** Address any linter warnings.
    *   **Add comments:** Provide clear and concise comments for the new structs, enum variants, and complex logic.
    *   **Update documentation (if applicable):** If there's user-facing documentation for tags, update it to include the new `@set` tag.

This plan ensures a systematic approach to integrating the new `@set` tag, covering parsing, semantic analysis, execution, and verification.
<!-- answer-6213b6ce-76eb-452a-9f6e-06dccbdbafc4:end -->
