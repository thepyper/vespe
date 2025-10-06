use pest::iterators::Pair;
use pest::error::{Error as PestError, InputLocation};

use pest::Parser; // This is the trait
use pest_derive::Parser; // This is the derive macro
use std::collections::HashMap;
use uuid::Uuid;

use super::types::*;

#[derive(Parser)]
#[grammar = "ast/ast.pest"]
pub struct AstParser;

pub fn parse_document(input: &str) -> Result<Vec<Line>, PestError<Rule>> {
    input.lines().enumerate().map(|(line_num, line_str)| {
        // Skip empty lines or lines that are just whitespace
        if line_str.trim().is_empty() {
            return Ok(Line { kind: LineKind::Text(String::new()), anchor: None });
        }
        parse_line(line_str).map_err(|e| {
            let error_span = match e.location {
                InputLocation::Pos(pos) => pest::Span::new(line_str, pos, pos).unwrap(),
                InputLocation::Span((start, end)) => pest::Span::new(line_str, start, end).unwrap(),
            };

            PestError::new_from_span(
                pest::error::ErrorVariant::CustomError {
                    message: format!("Error on line {}: {}", line_num + 1, e),
                },
                error_span
            )
        })
    }).collect()
}

fn parse_line(input: &str) -> Result<Line, PestError<Rule>> {
    let mut pairs = AstParser::parse(Rule::line, input)?;
    let pair = pairs.next().unwrap();

    let mut line_kind = LineKind::Text(String::new());
    let mut anchor: Option<Anchor> = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::tagged_line_content => {
                line_kind = parse_tagged_line(inner_pair)?;
            },
            Rule::text_content => {
                line_kind = LineKind::Text(inner_pair.as_str().trim_end().to_string());
            },
            Rule::anchor_comment => {
                anchor = Some(parse_anchor(inner_pair)?);
            },
            _ => {},
        }
    }

    Ok(Line { kind: line_kind, anchor })
}

fn parse_anchor(pair: Pair<Rule>) -> Result<Anchor, PestError<Rule>> {
    let span = pair.as_span();
    let mut kind: Option<AnchorKind> = None;
    let mut uid: Option<Uuid> = None;
    let mut tag: AnchorTag = AnchorTag::None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::anchor_kind => {
                kind = Some(inner_pair.as_str().parse().map_err(|e| PestError::new_from_span(pest::error::ErrorVariant::CustomError { message: e }, inner_pair.as_span()))?);
            },
            Rule::uuid => {
                uid = Some(Uuid::parse_str(inner_pair.as_str()).map_err(|e| PestError::new_from_span(pest::error::ErrorVariant::CustomError { message: e.to_string() }, inner_pair.as_span()))?);
            },
            Rule::anchor_tag => {
                tag = inner_pair.as_str().parse().map_err(|e| PestError::new_from_span(pest::error::ErrorVariant::CustomError { message: e }, inner_pair.as_span()))?;
            },
            _ => {},
        }
    }

    Ok(Anchor {
        kind: kind.ok_or_else(|| PestError::new_from_span(pest::error::ErrorVariant::CustomError { message: "Missing anchor kind".to_string() }, span.clone()))?,
        uid: uid.ok_or_else(|| PestError::new_from_span(pest::error::ErrorVariant::CustomError { message: "Missing UUID".to_string() }, span.clone()))?,
        tag,
    })
}

fn parse_tagged_line(pair: Pair<Rule>) -> Result<LineKind, PestError<Rule>> {
    let span = pair.as_span();
    let mut tag_kind: Option<TagKind> = None;
    let mut parameters: HashMap<String, String> = HashMap::new();
    let mut arguments: Vec<String> = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::tag_kind => {
                tag_kind = Some(inner_pair.as_str().parse().map_err(|e| PestError::new_from_span(pest::error::ErrorVariant::CustomError { message: e }, inner_pair.as_span()))?);
            },
            Rule::parameters => {
                for param_pair in inner_pair.into_inner() {
                    if let Rule::parameter = param_pair.as_rule() {
                        let mut key: Option<String> = None;
                        let mut value: Option<String> = None;
                        for kv_pair in param_pair.into_inner() {
                            match kv_pair.as_rule() {
                                Rule::key => key = Some(kv_pair.as_str().to_string()),
                                Rule::value => value = Some(kv_pair.as_str().to_string()),
                                _ => {},
                            }
                        }
                        if let (Some(k), Some(v)) = (key, value) {
                            parameters.insert(k, v);
                        }
                    }
                }
            },
            Rule::arguments => {
                for arg_pair in inner_pair.into_inner() {
                    if let Rule::argument = arg_pair.as_rule() {
                        let arg_str = arg_pair.as_str();
                        if arg_str.starts_with('"') && arg_str.ends_with('"') {
                            // Remove quotes and unescape inner quotes
                            arguments.push(arg_str[1..arg_str.len() - 1].replace("\"", "\""));
                        } else {
                            arguments.push(arg_str.to_string());
                        }
                    }
                }
            },
            _ => {},
        }
    }

    Ok(LineKind::Tagged {
        tag: tag_kind.ok_or_else(|| PestError::new_from_span(pest::error::ErrorVariant::CustomError { message: "Missing tag kind".to_string() }, span.clone()))?,
        parameters,
        arguments,
    })
}
