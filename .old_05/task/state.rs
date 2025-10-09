use anyhow::anyhow;
use std::path::Path;
use thiserror::Error;

use markdown::mdast::{Heading, Node, Paragraph, Root};
use markdown::unist::{Point, Position};

pub enum PlanSectionItem {
    LocalTask(String),
    ReferencedTask(String),
}

pub enum SectionContent {
    Intent {
        title: String,
        text: String,
    },
    Plan {
        title: String,
        items: Vec<PlanSectionItem>,
    },
    Text {
        title: String,
        text: String,
    },
}

pub struct Section {
    uid: String,
    content: SectionContent,
    position: Position,
}

pub struct State {
    /// Current markdown content
    md: String,
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
    pub fn reconcile(md_file_path: &Path) -> Result<(), Error> {
        // TODO return type? result ok, result errore con problemi (potenzialmente da sistemare con utente)
        // TODO carica md
        // TODO parsing md -> mdast
        // TODO parsing mdast -> new_sections
        // TODO reconcile new_sections con sections esistenti; in questa fase marking nuove sezioni, invenzione uid, genera modifiche per marking
        // TODO se reconcile ok, salva nuovo file sia internamente che sovrascrivi quello passato; altrimenti non salvare nulla

        // TODO decidere come dare a utente domande / risposte su reconcile... 1 - passare per file scrivendo dei tag; 2 - passare struttura dati runtime;
        // forse 1 piu' coerente con il design?
        unimplemented!();
    }

    fn parse_markdown_heading_into_section(
        md_node_content: &str,
        md_heading: Heading,
    ) -> Result<SectionContent, Error> {
        unimplemented!();
    }

    fn parse_markdown_paragraph_into_section(
        md_node_content: &str,
        md_paragraph: Paragraph,
    ) -> Result<SectionContent, Error> {
        Ok(SectionContent::Text {
            title: String::new(),
            text: md_node_content.into(),
        })
    }

    fn parse_markdown_node_into_section(
        md_content: &str,
        md_ast: Node,
    ) -> Result<Option<Section>, Error> {
        let position = md_ast
            .position()
            .ok_or(anyhow!("No position in md_ast"))?
            .clone();

                let md_node_content = 
            &md_content[position.start.offset as usize..position.end.offset as usize];

        let content = match md_ast {
            Node::Heading(ref heading) => Some(Self::parse_markdown_heading_into_section(
                md_node_content,
                heading.clone(),
            )?),
            Node::Paragraph(ref paragraph) => Some(Self::parse_markdown_paragraph_into_section(
                md_node_content,
                paragraph.clone(),
            )?),
            _ => None,
        };

        Ok(content.map(|content| Section {
            uid: uuid::Uuid::new_v4().to_string(),
            content,
            position,
        }))
    }

    fn parse_markdown_ast_into_sections(
        md_content: &str,
        md_ast: Node,
    ) -> Result<Vec<Section>, Error> {
        match md_ast {
            Node::Root(root) => root
                .children
                .into_iter()
                .map(|x| Self::parse_markdown_node_into_section(md_content, x))
                .filter_map(|x| x.transpose())
                .collect(),
            _ => Err(anyhow!("No Root node in md_ast").into()),
        }
    }

    fn parse_markdown_into_sections(md_content: &str) -> Result<Vec<Section>, Error> {
        let md_ast = markdown::to_mdast(&md_content, &markdown::ParseOptions::default())
            .map_err(|e| Error::Markdown(e.to_string()))?;
        let sections = Self::parse_markdown_ast_into_sections(md_content, md_ast)?;
        Ok(sections)
    }
}
