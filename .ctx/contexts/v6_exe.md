@include rules

Prossimo improvement e' spostare ContextManager direttamente in Project.
Ho deciso che invece di essere una struttura locale usata in alcune funzioni,
la capacita' di ContextManager di tenere una cache locale in-memory dei context
vada bene per Project. E sia globale.
Sposta quindi le capacita' di ContextManager in Project.
Una funzione aggiuntiva che deve essere disponibile e' un flush_modified() che
scriva su file tutti i context marcati come modificati e cancelli poi
la memoria dei modificati.

@answer                                                                         <!-- answer-49e3069d-786d-4fc7-a4aa-4c4a85cfe9ad:begin -->
Okay, I understand. I will refactor the `ContextManager` functionality into the `Project` struct, ensuring it manages contexts globally with an in-memory cache. I will also add a `flush_modified()` method to `Project` to handle writing modified contexts to files.

First, I need to locate the `ContextManager` definition.
Okay, `ContextManager` is in `src/project.rs` and used across `src/execute/`. I'll read `src/project.rs` to understand both `ContextManager` and `Project` structures.
Alright, `src/project.rs` refactor plan: move `ContextManager` fields (`contexts`, `modified_contexts`) to `Project`, initializing them in `Project::init` and `Project::find`. Then, transfer all `ContextManager` methods to `Project`, adjusting signatures (removing `&mut self` from `ContextManager` and adding to `Project`, dropping `project: &Project` parameter). Internal calls will update to `self.read_and_parse_context_file` and `self.update_context_lines`. Finally, I'll remove the `ContextManager` struct and update `execute` module files. Starting with `Project` struct modification in `src/project.rs`.
Fields added to `Project`. Now, initialize them in `Project::init` and `Project::find`. Starting with `Project::init`.
Okay, `Project::find` fields initialized.
`Project` now has `contexts` and `modified_contexts`, initialized. I'll move `ContextManager` methods to `Project`, starting with `insert_context`.
I messed up. I replaced the whole `impl ContextManager` block instead of just moving `insert_context`. Reverting, then I'll read the file and move each method to `impl Project` individually.
Right, `src/project.rs` is loaded. I'll append `ContextManager` methods (`insert_context`, `get_context_mut`, `remove_context`, `contains_context`) to `impl Project`, then delete `ContextManager` and its `impl` block.
Okay, `ContextManager` methods are now in `Project`, `save_modified_contexts` is `flush_modified_contexts`, and `load_context`/`get_context` are updated for `Project`'s internal cache, including `self.modified_contexts.clear()`. Next, I'll delete the `ContextManager` struct and its `impl` block.
I messed up. My last `replace` command was too aggressive, deleting part of the `Project` struct. I'll revert by rereading the file, then precisely target and remove only the `ContextManager` definition and its `impl` block.

                                                                                <!-- answer-49e3069d-786d-4fc7-a4aa-4c4a85cfe9ad:end -->

continua applicazione del piano, commit quando finisci

@answer

