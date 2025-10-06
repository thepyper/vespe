
Stiamo scrivendo src/project.rs, che contiene una minimale struttura Project.

Project fa da risolutore dei path, serve per orientarsi a partire da un root path nel progetto.

Migliorie:

1) vorrei che la cartella ".ctx" non fosse hardcoded ma fosse una costante;
2) vorrei che il file ".ctx_root" non fosse hardcoded ma una costante;
3) vorrei una funzione per ritornare il "project_home" ovvero project_root.join(".ctx")
4) vorrei una funzione per ritornare il "metadata_home" ovvero project_root.join(".ctx").join(".meta")
5) una funzione per il contexts root project_root.join("contexts")
6) una funzione per il snippets root project_root.join("snippets")
7) una funzione per "risolvere" un context, ovvero resolve_context(name) -> project_root.join(contexts).join("name.md")
8) una per risolvere gli snippet
9) una per risolvere metadati ancora (project_root.join(".meta").join("xxxx-yyyy")) con xxxx-yyyy anchor kind + uuid

Per capire i tipi di dato leggi in src/ast/types.rs

chiaro?

@answer



