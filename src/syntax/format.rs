use super::types::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {}

pub type Result<T> = std::result::Result<T, Error>;

pub fn format_document(lines: &Vec<Line>) -> String {
    lines
        .iter()
        .map(|line| line.to_string())
        .collect::<Vec<String>>()
        .join("\n")
}
