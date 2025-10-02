@include ctx

# ast

Abbiamo un prototipo ben funzionante, ma implementato un po male.
Per migliorare prima di tutto dobbiamo migliorare le strutture.

La AST non va bene, non e' una vera AST perche' un nodo ContextAstNode contiene due 
sottostrutture ridondanti: children sono altri ContextAstNode (files), e lines sono le righe.

In realta', dovrebbe essere una struttura ad albero ben fatta, tipo:

struct Context {
    file_path,
    lines: Vec<AstNode>,
}

struct Snippet {
    file_path,
    lines: Vec<AstNode>,
}

enum LineData {
    Text(String),
    Include(Context),
    Inline(Snippet),
    Answer,
    Summary(Context),
}

struct Line {
    line_number,
    data: LineData,
}

enum AstNode {
    Context(Context),
    Line(Line),
    Snippet(Snippet),
} 

In questo modo sono tutti nodi.

Poi ci vuole un Visitor pulito, che li navighi, tipo

trait Visitor {
    fn pre_visit_context()
    fn post_visit_context()
    fn pre_visit_snippet()
    fn post_visit_snippet()
    fn pre_visit_line()
    fn post_visit_line()
}

fn walk() 

chiaro?
fai un piano pulito per questa modifica molto ampia.
fai domande se qualcosa non ti e' chiaro.

Okay, I understand. The goal is to refactor the AST structure for the `ctx` tool to be more robust and tree-like, as well as to implement a `Visitor` pattern for navigating it. This is a significant change, so a clear plan and granular commits are essential.

Before I propose a detailed plan, I need to understand the current implementation of the AST. I'll start by listing the files in `src/bin/ctx/` to get an overview, and then I'll read `Cargo.toml` to understand the project dependencies and structure. After that, I'll delve into the source files to grasp the existing AST.

Okay, I see the files in `src/bin/ctx/`. `ast.rs` is the likely candidate for the current AST definition.

Now, I'll read `Cargo.toml` to understand the project's dependencies and structure.`Cargo.toml` reviewed. "vespe" is a Rust project, using `anyhow`, `thiserror`, `markdown`, `tokio`, `serde`, `serde_json`, `uuid`, `clap`, `dirs`, `notify`, `ctrlc`, `handlebars`, `sha2`, and `walkdir`. `markdown` is notable for LLM context.

Next, I'll examine `ast.rs` for the current AST structure.Okay, I've analyzed `ast.rs` and agree the current `ContextAstNode` is not a true AST. Your proposed structure is much better.

My plan is divided into four phases:

**Phase 1: Define New AST Structures**
1.  Create `struct Context` (`file_path: PathBuf`, `lines: Vec<AstNode>`).
2.  Create `struct Snippet` (`file_path: PathBuf`, `lines: Vec<AstNode>`).
3.  Modify `LineData` enum to use `Context` and `Snippet` (e.g., `Include(Context)`).
4.  Modify `Line` struct (`line_number: usize`, `data: LineData`). Remove `source_file`.
5.  Create `enum AstNode` (`Context(Context)`, `Line(Line)`, `Snippet(Snippet)`).

**Phase 2: Update Parsing and Building Logic**
1.  Refactor `ContextAstNode::parse` to return `Vec<AstNode>` (wrapping `Line` in `AstNode::Line`).
2.  Completely rewrite `ContextAstNode::build_context_ast` to construct the new `Context` and `Snippet` structs, embedding them directly into `LineData` variants.
3.  Create a new function for parsing snippets, similar to `build_context_ast`.

**Phase 3: Implement the Visitor Pattern**
1.  Define `trait Visitor` with `pre_visit_*` and `post_visit_*` methods for `Context`, `Snippet`, and `Line`.
2.  Implement `fn walk(node: &AstNode, visitor: &mut impl Visitor)` for AST traversal.

**Phase 4: Integrate and Test**
1.  Update `composer.rs` and `main.rs` to use the new AST and `walk` function.
2.  Verify functionality with existing or new test cases.

**Clarifications:**
*   I will maintain `anyhow::Result` for error handling.
*   Circular dependencies will be handled by replicating the current behavior (returning an empty node).
*   `source_file` will be removed from `Line`, and `line_number` will be relative to its parent `Context` or `Snippet`.

Do you approve this plan?

Approvo pienamente il piano. Commit granulari mi raccomando.

@answer
