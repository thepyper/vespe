use super::*;

#[test]
fn test_parser_consume_char_if() {
    let doc = "abc";
    let mut parser = Parser::new(doc);

    assert_eq!(parser.consume_char_if(|c| c == 'a'), Some('a'));
    assert_eq!(parser.remain(), "bc");

    assert_eq!(parser.consume_char_if(|c| c == 'x'), None);
    assert_eq!(parser.remain(), "bc");

    assert_eq!(parser.consume_char_if(|c| c == 'b'), Some('b'));
    assert_eq!(parser.remain(), "c");

    assert_eq!(parser.consume_char_if(|c| c == 'c'), Some('c'));
    assert_eq!(parser.remain(), "");

    assert_eq!(parser.consume_char_if(|_| true), None);
}

#[test]
fn test_parser_consume_char_if_immutable() {
    let doc = "abc";
    let parser = Parser::new(doc);

    let (c1, p1) = parser.consume_char_if_immutable(|c| c == 'a').unwrap();
    assert_eq!(c1, 'a');
    assert_eq!(p1.remain(), "bc");
    assert_eq!(parser.remain(), "abc"); // Original unchanged

    assert!(parser.consume_char_if_immutable(|c| c == 'x').is_none());

    let (c2, p2) = p1.consume_char_if_immutable(|c| c == 'b').unwrap();
    assert_eq!(c2, 'b');
    assert_eq!(p2.remain(), "c");
}

#[test]
fn test_parser_consume_matching_char() {
    let doc = "abc";
    let mut parser = Parser::new(doc);

    assert_eq!(parser.consume_matching_char('a'), Some('a'));
    assert_eq!(parser.remain(), "bc");

    assert_eq!(parser.consume_matching_char('x'), None);
    assert_eq!(parser.remain(), "bc");

    assert_eq!(parser.consume_matching_char('b'), Some('b'));
    assert_eq!(parser.remain(), "c");
}

#[test]
fn test_parser_consume_matching_char_immutable() {
    let doc = "abc";
    let parser = Parser::new(doc);

    let p1 = parser.consume_matching_char_immutable('a').unwrap();
    assert_eq!(p1.remain(), "bc");
    assert_eq!(parser.remain(), "abc");

    assert!(parser.consume_matching_char_immutable('x').is_none());

    let p2 = p1.consume_matching_char_immutable('b').unwrap();
    assert_eq!(p2.remain(), "c");
}

#[test]
fn test_parser_consume_matching_string() {
    let doc = "hello world";
    let mut parser = Parser::new(doc);

    assert_eq!(
        parser.consume_matching_string("hello"),
        Some("hello".to_string())
    );
    assert_eq!(parser.remain(), " world");

    assert_eq!(parser.consume_matching_string("foo"), None);
    assert_eq!(parser.remain(), " world");

    assert_eq!(
        parser.consume_matching_string(" world"),
        Some(" world".to_string())
    );
    assert_eq!(parser.remain(), "");
}

#[test]
fn test_parser_consume_matching_string_immutable() {
    let doc = "hello world";
    let parser = Parser::new(doc);

    let p1 = parser.consume_matching_string_immutable("hello").unwrap();
    assert_eq!(p1.remain(), " world");
    assert_eq!(parser.remain(), "hello world");

    assert!(parser.consume_matching_string_immutable("foo").is_none());

    let p2 = p1.consume_matching_string_immutable(" world").unwrap();
    assert_eq!(p2.remain(), "");
}

#[test]
fn test_parser_consume_many_if() {
    let doc = "aaabbc";
    let mut parser = Parser::new(doc);

    assert_eq!(
        parser.consume_many_if(|c| c == 'a'),
        Some("aaa".to_string())
    );
    assert_eq!(parser.remain(), "bbc");

    assert_eq!(parser.consume_many_if(|c| c == 'b'), Some("bb".to_string()));
    assert_eq!(parser.remain(), "c");

    assert_eq!(parser.consume_many_if(|c| c == 'x'), None);
    assert_eq!(parser.remain(), "c");
}

#[test]
fn test_parser_consume_many_if_immutable() {
    let doc = "aaabbc";
    let parser = Parser::new(doc);

    let (s1, p1) = parser.consume_many_if_immutable(|c| c == 'a');
    assert_eq!(s1, "aaa");
    assert_eq!(p1.remain(), "bbc");
    assert_eq!(parser.remain(), "aaabbc");

    let (s2, p2) = p1.consume_many_if_immutable(|c| c == 'b');
    assert_eq!(s2, "bb");
    assert_eq!(p2.remain(), "c");
}

#[test]
fn test_parser_consume_many_of() {
    let doc = "abccba";
    let mut parser = Parser::new(doc);

    assert_eq!(parser.consume_many_of("abc"), Some("abccba".to_string()));
    assert_eq!(parser.remain(), "");

    let doc2 = "123xyz";
    let mut parser2 = Parser::new(doc2);
    assert_eq!(parser2.consume_many_of("123"), Some("123".to_string()));
    assert_eq!(parser2.remain(), "xyz");
}

#[test]
fn test_parser_skip_many_whitespaces() {
    let doc = "   \t\rhello";
    let mut parser = Parser::new(doc);
    parser.skip_many_whitespaces();
    assert_eq!(parser.remain(), "hello");

    let doc2 = "hello";
    let mut parser2 = Parser::new(doc2);
    parser2.skip_many_whitespaces();
    assert_eq!(parser2.remain(), "hello");
}

#[test]
fn test_parser_skip_many_whitespaces_immutable() {
    let doc = "   \t\rhello";
    let parser = Parser::new(doc);
    let p1 = parser.skip_many_whitespaces_immutable();
    assert_eq!(p1.remain(), "hello");
    assert_eq!(parser.remain(), "   \t\rhello");
}

#[test]
fn test_parser_skip_many_whitespaces_or_eol() {
    let doc = "   \t\r\n\nhello";
    let mut parser = Parser::new(doc);
    parser.skip_many_whitespaces_or_eol();
    assert_eq!(parser.remain(), "hello");
    assert_eq!(parser.get_position().line, 3);
    assert_eq!(parser.get_position().column, 1);

    let doc2 = "hello";
    let mut parser2 = Parser::new(doc2);
    parser2.skip_many_whitespaces_or_eol();
    assert_eq!(parser2.remain(), "hello");
}

#[test]
fn test_parser_skip_many_whitespaces_or_eol_immutable() {
    let doc = "   \t\r\n\nhello";
    let parser = Parser::new(doc);
    let p1 = parser.skip_many_whitespaces_or_eol_immutable();
    assert_eq!(p1.remain(), "hello");
    assert_eq!(p1.get_position().line, 3);
    assert_eq!(p1.get_position().column, 1);
    assert_eq!(parser.remain(), "   \t\r\n\nhello");
}
