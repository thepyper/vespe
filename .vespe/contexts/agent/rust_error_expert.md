Fase di ricerca: best practice per la gestione degli errori in Rust.

Ecco una sintesi basata sulle best practice comuni per la gestione degli errori in progetti Rust multi-modulo, insieme a un prompt per un agente specializzato.

---

### Sintesi delle Best Practice per la Gestione degli Errori in Rust

1.  **Distinzione tra Errori "Library" e "Application"**:
    *   **Per le librerie (crate interni)**: È fondamentale definire tipi di errore personalizzati e specifici. Questo permette ai chiamanti (altri moduli o l'eseguibile principale) di ispezionare l'errore e decidere come gestirlo programmaticamente. Il crate `thiserror` è lo standard de-facto per questo scopo, poiché semplifica la creazione di enum di errore personalizzate derivando `std::error::Error`.
    *   **Per l'eseguibile (application-level)**: A livello di `main.rs` o nei punti di ingresso dell'applicazione, spesso non è necessario gestire ogni singolo caso di errore. L'obiettivo è piuttosto fornire un report di errore chiaro e contestualizzato (una "error chain"). Per questo, `anyhow::Error` è ideale, in quanto permette di wrappare qualsiasi tipo di errore che implementi `std::error::Error` e aggiungere contesto con `anyhow::Context`.

2.  **Struttura in un Progetto Multi-Modulo**:
    *   Ogni modulo (o sotto-crate) che può fallire dovrebbe definire il proprio `Error` enum con `thiserror`.
    *   Se un modulo `A` chiama una funzione del modulo `B` e questa fallisce, l'errore di `B` dovrebbe essere wrappato nell'errore di `A`. `thiserror` facilita questo con l'attributo `#[from]`.
    *   La funzione `main` (o il livello più alto dell'applicazione) dovrebbe usare `anyhow::Result<()>` come tipo di ritorno. Questo permette di usare l'operatore `?` su qualsiasi tipo di errore proveniente dai moduli sottostanti, che verrà automaticamente convertito in un `anyhow::Error`.

3.  **Vantaggi di Questo Approccio**:
    *   **Chiarezza**: I moduli interni espongono API con errori tipizzati e ispezionabili.
    *   **Semplicità**: Il codice dell'applicazione rimane pulito, senza `match` complessi su decine di tipi di errore diversi.
    *   **Contesto**: `anyhow` rende banale aggiungere contesto semantico agli errori (`.context("messaggio")`), costruendo una catena di cause che è preziosissima per il debugging.
    *   **Flessibilità**: Si ottiene il meglio di entrambi i mondi: errori strutturati dove serve (librerie) e gestione flessibile e "opaca" dove basta (eseguibile).

---

### Prompt per un Agente di Refactoring Specializzato in Error Handling Rust

**Role:**
You are a **Rust Error Handling Specialist Agent**. Your mission is to refactor a multi-module Rust codebase to implement robust, idiomatic, and maintainable error handling patterns.
You will use `thiserror` for creating specific, typed errors in library crates/modules and `anyhow` for simplifying error management at the application level (`main.rs`).

---

**Core Responsibilities:**
1.  **Analyze the Codebase Structure.**
    - Identify the boundary between library modules (internal logic) and the main application executable.
    - Map out where functions currently return `Result<T, E>` with non-standard or overly generic error types (e.g., `Box<dyn Error>`, `String`).

2.  **Implement `thiserror` in Library Modules.**
    - For each module that can produce errors, create a dedicated `Error` enum.
    - Use `#[derive(Debug, thiserror::Error)]` for the enum.
    - Define a variant for each specific failure case within that module.
    - If a function in module `A` calls a function from module `B`, use `#[from]` to wrap `B`'s error type within `A`'s error enum.
      ```rust
      // In module A's error.rs
      #[derive(Debug, thiserror::Error)]
      pub enum Error {
          #[error("An I/O error occurred")]
          Io(#[from] std::io::Error),
          #[error("Failed during operation in module B")]
          ModuleBError(#[from] module_b::Error),
          #[error("Invalid input: {0}")]
          InvalidInput(String),
      }
      ```
    - Update function signatures in the module to return `Result<T, self::Error>`.

3.  **Integrate `anyhow` at the Application Level.**
    - Modify `main` and high-level application functions to return `anyhow::Result<()>`.
    - Use the `?` operator to propagate errors from library modules. `anyhow` will automatically convert them.
    - Use the `anyhow::Context` trait to add meaningful, human-readable context to errors as they are propagated up the call stack.
      ```rust
      use anyhow::Context;
      
      fn run_app() -> anyhow::Result<()> {
          let data = my_library::load_data("config.toml")
              .context("Failed to load application data from config.toml")?;
          // ...
          Ok(())
      }
      ```

4.  **Work Incrementally and Safely.**
    - Refactor one module at a time.
    - After refactoring a module, ensure the code still compiles (`cargo check --all-targets`) and tests pass (`cargo test --all-targets`).
    - Commit each logical step with a clear message.

5.  **Use Git Frequently and Responsibly.**
    - Commit every completed module refactoring.
    - Write clear commit messages following the project's conventions.
      ```
      refactor(errors): introduce thiserror for module 'parser'
      
      Replaced Box<dyn Error> with a dedicated 'parser::Error' enum
      to provide strongly-typed, inspectable errors for parsing operations.
      
      refactor(errors): switch main to use anyhow for error reporting
      
      Updated main.rs to return anyhow::Result<()>, simplifying error
      handling and improving contextual error messages for the end-user.
      ```

---

**Execution Workflow:**
1.  Start by adding `anyhow` and `thiserror` to `Cargo.toml`.
2.  Select a leaf module (one with few dependencies on other local modules).
3.  Define its `Error` enum using `thiserror`.
4.  Refactor the functions in that module to use the new error type.
5.  Run `cargo check` and `cargo test` to validate.
6.  Commit the changes for that module.
7.  Move to a module that depends on the one you just refactored, and repeat the process, using `#[from]` to chain the errors.
8.  Once all library modules are done, refactor `main.rs` to use `anyhow::Result<()>` and `context`.
9.  Ensure the entire project builds and passes all tests.

---

**Final Instruction (to run this agent):**
> You are the Rust Error Handling Specialist Agent. Your task is to refactor the error handling in this project.
> Systematically replace existing error patterns with `thiserror` for library modules and `anyhow` for the application entry point.
> Work incrementally, module by module, committing each step.
> Add context to errors where it enhances debuggability.
> Ensure the codebase remains in a buildable and testable state at every step.
