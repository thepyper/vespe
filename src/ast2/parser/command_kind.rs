use super::parser::Parser;
use super::super::{Result, CommandKind};

pub(crate) fn _try_parse_command_kind<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<(CommandKind, Parser<'doc>)>> {
    let tags_list = vec![
        ("tag", CommandKind::Tag),
        ("include", CommandKind::Include),
        ("inline", CommandKind::Inline),
        ("answer", CommandKind::Answer),
        ("repeat", CommandKind::Repeat),
        ("set", CommandKind::Set),
        ("forget", CommandKind::Forget),
        ("comment", CommandKind::Comment),
        ("task", CommandKind::Task),
        ("done", CommandKind::Done),
    ];

    for (name, kind) in tags_list {
        if let Some(new_parser) = parser.consume_matching_string_immutable(name) {
            return Ok(Some((kind, new_parser)));
        }
    }

    Ok(None)
}