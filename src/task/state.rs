use std::path::Path;
use thiserror::Error;
use anyhow::anyhow;

use markdown::mdast::{Node, Heading, Root, Paragraph};

pub enum PlanSectionItem{
    LocalTask(String),
    ReferencedTask(String),
}

pub enum Section {
    Intent{ title: String, text: String },
    Plan{ title: String, items: Vec<PlanSectionItem> },
    Text{ title: String, items: String }, 
}

pub struct State {
    /// Original markdown file 
    md: String,
    /// Original markdown file parsed ast
    //mdast: markdown::mdast::Node,
    /// State structure parsed
    sections: Vec<Section>,
}

/* TODO decidere come fare
pub enum ReconcileQueryKind {
    DeleteConfirm,
}

pub struct ReconcileQuery {
    uid: String,    
    kind: ReconcileQueryKind,
}

pub enum ReconcileResponseKind {
    Accept,
    Reject,
}

pub enum ReconcileResponse {
    uid: String,
    query_kind: ReconcileQueryKind,
    response_kind: ReconcileResponseKind,
}
*/

#[derive(Debug, Error)]
enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),
    #[error("Markdown error: {0}")]
    Markdown(String),
    
}

struct SectionParsing {

    section: Section,
}

impl State {

    pub fn new() -> State {
        State {
            md: String::new(),
            sections: Vec::new(),
        }
    }
    /// Ricarica da files interni al task
    pub fn load(task_root_path: &Path) -> Result<State, Error> {
        // TODO se file non esiste, usa new()
        // TODO se file esiste, carica ed esegui parsing in mdast, e poi in sections
        // TODO se file malformato, errore
        unimplemented!();
    }
    /// Riconcilia file modificato da utente con file interno
    pub fn reconcile(md_file_path: &Path) -> Result<(), Error> { // TODO return type? result ok, result errore con problemi (potenzialmente da sistemare con utente)
        // TODO carica md
        // TODO parsing md -> mdast
        // TODO parsing mdast -> new_sections
        // TODO reconcile new_sections con sections esistenti; in questa fase marking nuove sezioni, invenzione uid, genera modifiche per marking
        // TODO se reconcile ok, salva nuovo file sia internamente che sovrascrivi quello passato; altrimenti non salvare nulla

        // TODO decidere come dare a utente domande / risposte su reconcile... 1 - passare per file scrivendo dei tag; 2 - passare struttura dati runtime;
        // forse 1 piu' coerente con il design?
        unimplemented!();
    }        

    fn parse_markdown_heading_into_section(md_heading: Heading) -> Result<SectionParsing, Error> {
        unimplemented!();
    }

    fn parse_markdown_paragraph_into_section(md_paragraph: Paragraph) -> Result<SectionParsing, Error> {
        unimplemented!();   
    }

    fn parse_markdown_node_into_section(md_ast: Node) -> Result<Option<SectionParsing>, Error> {
        match md_ast {
            Node::Heading(heading) => Ok(Some(Self::parse_markdown_heading_into_section(heading)?)),
            Node::Paragraph(paragraph) => Ok(Some(Self::parse_markdown_paragraph_into_section(paragraph)?)),
            _ => Ok(None),
        }
    }

    fn parse_markdown_ast_into_sections(md_ast: Node) -> Result<Vec<SectionParsing>, Error> {
        match md_ast {
            Node::Root(root) => root.children.into_iter()
                .map(|x| Self::parse_markdown_node_into_section(x))
                .filter_map(|x| x.transpose())
                .collect(),
            _ => Err(anyhow!("No Root node in md_ast").into()),
        }
    }

    fn load_markdown_and_parse_sections(md_file_path: &Path) -> Result<(String, Vec<SectionParsing>), Error> {
        let md_file_content = std::fs::read_to_string(md_file_path)?;
        let md_ast = markdown::to_mdast(&md_file_content, &markdown::ParseOptions::default()).map_err(|e| Error::Markdown(e.to_string()))?;
        let sections = Self::parse_markdown_ast_into_sections(md_ast)?;
        Ok((md_file_content, sections))
    }
}