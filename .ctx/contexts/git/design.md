@include rules

# git/design - design

Voglio implementare una integrazione (opzionale) di git.
Lo scopo e' evitare il cluttering che si genera nella cartella .meta,
e il cluttering che si genera nella cartella context.

Di fatto, tutto cio' che accade in .ctx dovrebbe essere gestito
con dei commit da ctx, per evitare cluttering inutile all'utente.
Inoltre un commit con un messaggio utile puo' essere
utile anche a capire dove fare dei revert se necessario.
Potrebbe essere anche un automatismo ad un certo punto, da capire.

<!-- summary-c051e158-f935-4db5-9a0f-f6fafc74d1fa:begin -->
The provided content discusses Git integration options in Rust and proposes a strategy for managing the `.ctx` directory within a Git repository.

**Git Integration Options in Rust:**

1.  **`std::process::Command` (CLI execution):**
    *   **Pros:** Full Git functionality, robust, simple for basic commands.
    *   **Cons:** External dependency (Git must be installed), process overhead, fragile output parsing, complex error handling.
    *   **Use Case:** When full Git features are needed and performance isn't critical, or for interacting with the main project repository.

2.  **`gix` (Pure Rust implementation):**
    *   **Pros:** Pure Rust (no C dependencies), high performance, memory safe, granular control over Git operations, actively developed.
    *   **Cons:** Steeper learning curve, potentially less mature than `libgit2` for all edge cases, documentation might be less extensive.
    *   **Use Case:** Ideal for high-performance, self-contained applications requiring fine-grained control, especially for internal agent operations. `gix` supports `commit`, `status`, `add`, `branch`, and `merge`.

3.  **`git2` (Bindings for `libgit2`):**
    *   **Pros:** Mature and comprehensive API, cross-platform, no dependency on Git binary.
    *   **Cons:** C dependency (requires C compiler), FFI overhead, potential low-level security risks.
    *   **Use Case:** When a complete and mature Git API is needed, and managing C dependencies is acceptable.

**Strategy for `.ctx` Directory Management:**

Initially, the suggestion was to treat `.ctx` as a separate Git repository to avoid clutter in the main project's history. However, based on user feedback preferring a unified history and checkpoints within the main repository, the strategy was revised:

*   **`gix` for Internal Operations:** `gix` is the preferred library for the agent's internal Git operations within `.ctx` (e.g., `add`, `commit`) due to its pure Rust nature, performance, and granular control.
*   **Commit to Active Branch:** The agent will commit changes directly to the **currently active Git branch** of the main repository.
*   **Fine-grained, Descriptive Commits:** Each commit by the agent will be fine-grained and include a descriptive message (title and body) explaining the changes, their purpose, and any relevant notes. This ensures the agent's actions are well-documented in the project history.
*   **User Control over Merge/Push:** The agent will **not** automatically perform `merge` or `push` operations. These actions remain under the user's explicit control, allowing the user to decide when and how to integrate the agent's local commits with remote repositories or other branches.

This revised strategy balances the need for unified checkpoints and well-documented history with the agent's ability to perform frequent, detailed Git operations efficiently.
<!-- summary-c051e158-f935-4db5-9a0f-f6fafc74d1fa:end -->