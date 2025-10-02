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

@answer


