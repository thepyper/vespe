@include ctx

# exe_refactor

Ora voglio fare un refactor di execute_context.
Prima di toccare la funzione, facciamo dei Visitor opportuni, uno per ogni cosa che devo fare.

Primo visitor: InlinerVisitor

Questo visitor visita il Context, e dove trova degli snippet, RISCRIVE il file originale che conteneva la direttiva @inline con lo snippet.

Per il resto, non fa nulla.

Implementiamo questo intanto.

Okay, I understand. The goal is to implement an `InlinerVisitor` that processes `@inline` directives within a `Context` and rewrites the original files with the snippet content. This is a preparatory step before refactoring `execute_context`.

First, I'll examine `Cargo.toml` and the `src/bin/ctx/` directory to understand the project structure and existing code related to context processing and snippets.
