@include v2_ctx

# v2_ast

Come primo step del progetto, voglio scrivere nella libreria il codice necessario per fare il parsing in una AST ben fatta di alcuni elementi testuali.
La base del formato di cui fare parsing e' Markdown, ma noi lo tratteremo, per semplicita', riga per riga.

Ogni riga del file puo':
- avere una ancora a fine riga, con il formato <!-- xxx-yyy:zzz -->
  con xxx stringa, yyy uid (numero), zzz stringa

- avere un tag ad inizio riga, con il formato @xxx[yyy0 = zzz0; yyy1 = zzz1] 
  che rende la riga di un tipo speciale
  
- altrimenti essere una riga di testo normale.

Voglio differenziare per ogni tag, non avere un parsing unico per singolo tag.

La struttura che voglio per la linea parsata e' la seguente:

struct Line {
	kind: LineKind,
	text: String,
	anchor: AnchorData,
}

struct Parameters e' un alias per HashMap<String, serde_json::Value>

LineKind serve a differenziare il tipo di linea, quindi:

enum LineKind{
	Text,
	Include{ context: Context, parameters: Parameters },
	Inline{ snippet: Snippet, parameters: Parameters },
	Answer{ parameters: Parameters },
	Summary{ Context, parameters: Parameters },
}

La struttura Context incapsula un file markdown intero:

struct Context {
	path: PathBuf,
	lines: Vec<Line>,
}

La struttura Snippet incapsula un file snippet intero (stesso formato markdown):

struct Snippet {
	path: PathBuf,
	lines: Vec<Line>,
}

Il parsing mediamente parte con un path ad un file context, quindi la funzione parse_context ritorna Context.

Durante il parsing sara' necessario passare un Resolver, che risolve i path a partire da dei nomi, ad esempio una riga @include <ctx> deve venire risolta perche' ctx non e' un path, ma un nome che va risolto.

trait Resolver {
	pub fn resolve_context(ctx_name: &str) -> PathBuf
	pub fn resolve_snippet ... simile
}

Mi aspetto un ast.rs del genere:

fn parse_context(path: &str) -> Result<Context>

fn parse_snippet(path: &str) -> Result<Snippet>

fn parse_line(text: &str) -> Result<Line>

Voglio che questa roba sia implementat in src/ast/ un nuovo modulo della libreria.

Tutto chiaro?
Se si procedi, se no chiedi.






