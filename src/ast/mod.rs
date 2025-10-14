use sha2::digest::typenum::Double;



struct Range
{
    begin: usize,  // 0-based offset
    end: usize,  // 0-based offset
}

struct Root
{
    children: Vec<Node>,
    range: Range,
}

struct Text
{
    range: Range,
}

enum Command
{
    Include,
    Inline,
    Answer,
    Derive,
    Summarize,
    Set,
    Repeat,
}

struct TagOpening
{
    command: Command,
    range: Range,
}

struct Parameters
{
    parameters: serde_json::Value,
    range: Range,
}

struct Argument
{
    range: Range,
}

struct Arguments
{
    children: Vec<Argument>,
    range: Range,
}

struct Tag
{
    opening: TagOpening,
    parameters: Parameters,
    arguments: Arguments,
    range: Range,
}

enum Kind
{
    Begin,
    End,
}

struct AnchorOpening
{
    command: Command,
    uuid: Uuid,
    kind: Kind,
    range: Range,
}

struct Anchor
{
    opening: AnchorOpening,
    parameters: Parameters,
    arguments: Arguments,
    range: Range,
}

enum Node
{
    Root(Root),
    Tag(Tag),
    Anchor(Anchor),
    Text(Text),
}

struct ParsingError
{
    // TODO definisci opportunamente
}

pub fn parse(document: &str) -> Result<Root, ParsingError> 
{
    let begin = 0usize;

    let (children, range) = parse_many_nodes(document, begin)?;

    Ok(Root{
        children,
        range,
    })

}

fn parse_many_nodes(document: &str, begin: usize) -> Result<Vec<Node>, ParsingError>
{
    let mut nodes = Vec::new();

    let end_offset = document.len();
    let mut position = begin;

    while position < end_offset {

        let (node, range) = parse_node(document, position)?;
        nodes.push(node);

        position = range.end;
    }

    Ok(nodes)
}

fn parse_node(document: &str, begin: usize) -> Result<Node, ParsingError>
{
    if let Some(tag) = parse_tag(document, begin)? {
        Ok(Node::Tag(tag))
    } else if let Some(anchor) = parse_anchor(document, begin)? {
        Ok(Node::Anchor(anchor))
    } else if let Some(text) = parse_text(document, begin)? {
        Ok(Node::Text(text))
    }

    // TODO errore: parsing not advanced!!
}

fn parse_tag(document: &str, begin: usize) -> Result<Option<Tag>, ParsingError>
{
    // ASSERT: begin.column == 1 altrimenti errore, partenza tag e' SEMPRE ad inizio linea

    // TODO: parse di un tag, fatto da:
    // 1) parse di @<nome-tag><spaces?> 
    // 2) call di parse_parameters che fa parsing di {} oggetto JSON (possibile che non ci sia, allora parameters e' un oggetto vuoto {})
    // 3) call di parse_arguments che fa il parsing del resto della linea dove e' finito il JSON con }, e separa le words in diversi argument; gestire ', e " per accorpare
    // ritornare struttura Tag, completa di calcolo del Range che comprende tutto il Tag compreso fine-linea
}

fn parse_anchor(document: &str, begin: usize) -> Result<Option<Anchor>, ParsingError>
{
    // ASSERT: begin.column == 1 altrimenti errore, partenza anchor e' SEMPRE ad inizio linea

    // TODO: parse di una anchor, fatto da:
    // 1) parse di <!--<spaces?><nome-tag>-<uuid>:<kind><spaces?> 
    // 2) call di parse_parameters, come in parse_tag 
    // 3) call di parse_arguments, come in parse_tag
    // 4) parse di <spaces?>-->
    // ritornare struttura Anchor, completa di calcolo del Range che comprende tutto il Tag compreso fine-linea
}

fn parse_parameters(document: &str, begin: usize) -> Result<Parameters, ParsingError>
{
    // TODO: parse dei parametri, fatto da:
    // 1) se non c'e' un { allora parameters ritorna json!({}) e range "nullo"
    // 2) se c'e' { allora parameters fa parse di json fino al } corrispondente
    // ritornare struttura Parameters, completa di calcolo del Range che comprende tutto il Tag compreso fine-linea
}

fn parse_arguments(document: &str, begin: usize) -> Result<Arguments, ParsingError>
{
    let mut arguments = Vec::new();

    let end_offset = ; // TODO cerca fine linea a partire da begin
    let mut position = begin;

    while position < end_offset {

        let (argument, range) = parse_argument(document, position)?;
        arguments.push(argument);

        position = range.end;
    }

    Ok(arguments)
}

fn parse_argument(document: &str, begin: usize) -> Result<Argument, ParsingError>
{
    // TODO parsing di una word, gestendo anche virgolette ' e " e tutto escaping standard (\" \' \n \r almeno)
}


