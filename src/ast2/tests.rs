use anyhow::Result;

#[cfg(test)]
#[path = "tests/utils.rs"]
mod utils;

#[cfg(test)]
#[path = "tests/test_parser_new.rs"]
mod test_parser_new;

#[cfg(test)]
#[path = "tests/test_parser_advance.rs"]
mod test_parser_advance;

#[cfg(test)]
#[path = "tests/test_parser_advance_newline.rs"]
mod test_parser_advance_newline;

#[cfg(test)]
#[path = "tests/test_parser_consume_matching_char.rs"]
mod test_parser_consume_matching_char;

#[cfg(test)]
#[path = "tests/test_parser_consume_matching_string.rs"]
mod test_parser_consume_matching_string;

#[cfg(test)]
#[path = "tests/test_parser_consume_char_if.rs"]
mod test_parser_consume_char_if;

#[cfg(test)]
#[path = "tests/test_parser_skip_many_whitespaces.rs"]
mod test_parser_skip_many_whitespaces;

#[cfg(test)]
#[path = "tests/test_parser_skip_many_whitespaces_or_eol.rs"]
mod test_parser_skip_many_whitespaces_or_eol;

#[cfg(test)]
#[path = "tests/test_try_parse_nude_integer.rs"]
mod test_try_parse_nude_integer;

#[cfg(test)]
#[path = "tests/test_try_parse_nude_float.rs"]
mod test_try_parse_nude_float;

#[cfg(test)]
#[path = "tests/test_try_parse_nude_bool.rs"]
mod test_try_parse_nude_bool;

#[cfg(test)]
#[path = "tests/test_try_parse_nude_string.rs"]
mod test_try_parse_nude_string;

#[cfg(test)]
#[path = "tests/test_try_parse_enclosed_value_double_quote.rs"]
mod test_try_parse_enclosed_value_double_quote;

#[cfg(test)]
#[path = "tests/test_try_parse_enclosed_value_single_quote.rs"]
mod test_try_parse_enclosed_value_single_quote;

#[cfg(test)]
#[path = "tests/test_try_parse_enclosed_value_with_escapes.rs"]
mod test_try_parse_enclosed_value_with_escapes;

#[cfg(test)]
#[path = "tests/test_try_parse_enclosed_value_unterminated.rs"]
mod test_try_parse_enclosed_value_unterminated;

#[cfg(test)]
#[path = "tests/test_try_parse_identifier.rs"]
mod test_try_parse_identifier;

#[cfg(test)]
#[path = "tests/test_try_parse_parameter.rs"]
mod test_try_parse_parameter;

#[cfg(test)]
#[path = "tests/test_try_parse_parameters0_empty.rs"]
mod test_try_parse_parameters0_empty;

#[cfg(test)]
#[path = "tests/test_try_parse_parameters0_single.rs"]
mod test_try_parse_parameters0_single;

#[cfg(test)]
#[path = "tests/test_try_parse_parameters0_multiple.rs"]
mod test_try_parse_parameters0_multiple;

#[cfg(test)]
#[path = "tests/test_try_parse_parameters0_multiline.rs"]
mod test_try_parse_parameters0_multiline;

#[cfg(test)]
#[path = "tests/test_try_parse_parameters0_missing_colon.rs"]
mod test_try_parse_parameters0_missing_colon;

#[cfg(test)]
#[path = "tests/test_try_parse_parameters0_missing_value.rs"]
mod test_try_parse_parameters0_missing_value;

#[cfg(test)]
#[path = "tests/test_try_parse_parameters0_missing_comma_or_brace.rs"]
mod test_try_parse_parameters0_missing_comma_or_brace;

#[cfg(test)]
#[path = "tests/test_try_parse_argument_quoted.rs"]
mod test_try_parse_argument_quoted;

#[cfg(test)]
#[path = "tests/test_try_parse_argument_unquoted.rs"]
mod test_try_parse_argument_unquoted;

#[cfg(test)]
#[path = "tests/test_try_parse_arguments_single.rs"]
mod test_try_parse_arguments_single;

#[cfg(test)]
#[path = "tests/test_try_parse_arguments_multiple.rs"]
mod test_try_parse_arguments_multiple;

#[cfg(test)]
#[path = "tests/test_try_parse_arguments_multiline.rs"]
mod test_try_parse_arguments_multiline;

#[cfg(test)]
#[path = "tests/test_try_parse_command_kind.rs"]
mod test_try_parse_command_kind;

#[cfg(test)]
#[path = "tests/test_try_parse_uuid.rs"]
mod test_try_parse_uuid;

#[cfg(test)]
#[path = "tests/test_try_parse_anchor_kind.rs"]
mod test_try_parse_anchor_kind;

#[cfg(test)]
#[path = "tests/test_try_parse_tag0_simple.rs"]
mod test_try_parse_tag0_simple;

#[cfg(test)]
#[path = "tests/test_try_parse_tag0_with_parameters.rs"]
mod test_try_parse_tag0_with_parameters;

#[cfg(test)]
#[path = "tests/test_try_parse_anchor0_simple.rs"]
mod test_try_parse_anchor0_simple;

#[cfg(test)]
#[path = "tests/test_try_parse_anchor0_with_parameters_and_args.rs"]
mod test_try_parse_anchor0_with_parameters_and_args;

#[cfg(test)]
#[path = "tests/test_try_parse_text_simple.rs"]
mod test_try_parse_text_simple;

#[cfg(test)]
#[path = "tests/test_try_parse_text_until_tag.rs"]
mod test_try_parse_text_until_tag;

#[cfg(test)]
#[path = "tests/test_try_parse_text_until_anchor.rs"]
mod test_try_parse_text_until_anchor;

#[cfg(test)]
#[path = "tests/test_parse_content_mixed.rs"]
mod test_parse_content_mixed;

#[cfg(test)]
#[path = "tests/test_parse_document_simple.rs"]
mod test_parse_document_simple;

#[cfg(test)]
#[path = "tests/test_parse_document_empty.rs"]
mod test_parse_document_empty;

#[cfg(test)]
#[path = "tests/test_parse_document_only_whitespace.rs"]
mod test_parse_document_only_whitespace;

#[cfg(test)]
#[path = "tests/test_parse_document_error.rs"]
mod test_parse_document_error;