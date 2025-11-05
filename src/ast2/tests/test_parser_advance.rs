use super::*;

#[test]
fn test_parser_new() {
    let doc = "hello";
    let parser = Parser::new(doc);
    assert_eq!(parser.get_position().line, 1);
    assert_eq!(parser.get_position().column, 1);
    assert_eq!(parser.remain(), "hello");
    assert!(!parser.is_eod());
}

#[test]
fn test_parser_advance() {
    let doc = "abc\ndef";
    let mut parser = Parser::new(doc);

    // 'a'
    assert_eq!(parser.advance(), Some('a'));
    assert_eq!(parser.get_position().line, 1);
    assert_eq!(parser.get_position().column, 2);
    assert_eq!(parser.remain(), "bc\ndef");

    // 'b'
    assert_eq!(parser.advance(), Some('b'));
    assert_eq!(parser.get_position().line, 1);
    assert_eq!(parser.get_position().column, 3);
    assert_eq!(parser.remain(), "c\ndef");

    // 'c'
    assert_eq!(parser.advance(), Some('c'));
    assert_eq!(parser.get_position().line, 1);
    assert_eq!(parser.get_position().column, 4);
    assert_eq!(parser.remain(), "\ndef");

    // '\n'
    assert_eq!(parser.advance(), Some('\n'));
    assert_eq!(parser.get_position().line, 2);
    assert_eq!(parser.get_position().column, 1);
    assert_eq!(parser.remain(), "def");
    assert!(parser.is_begin_of_line());

    // 'd'
    assert_eq!(parser.advance(), Some('d'));
    assert_eq!(parser.get_position().line, 2);
    assert_eq!(parser.get_position().column, 2);
    assert_eq!(parser.remain(), "ef");

    // 'e'
    assert_eq!(parser.advance(), Some('e'));
    assert_eq!(parser.get_position().line, 2);
    assert_eq!(parser.get_position().column, 3);
    assert_eq!(parser.remain(), "f");

    // 'f'
    assert_eq!(parser.advance(), Some('f'));
    assert_eq!(parser.get_position().line, 2);
    assert_eq!(parser.get_position().column, 4);
    assert_eq!(parser.remain(), "");
    assert!(parser.is_eod());

    // EOD
    assert_eq!(parser.advance(), None);
    assert!(parser.is_eod());
}

#[test]
fn test_parser_advance_immutable() {
    let doc = "ab";
    let parser = Parser::new(doc);

    let (c1, p1) = parser.advance_immutable().unwrap();
    assert_eq!(c1, 'a');
    assert_eq!(p1.remain(), "b");
    assert_eq!(parser.remain(), "ab"); // Original parser is unchanged

    let (c2, p2) = p1.advance_immutable().unwrap();
    assert_eq!(c2, 'b');
    assert_eq!(p2.remain(), "");

    assert!(p2.advance_immutable().is_none());
}

#[test]
fn test_parser_get_position() {
    let doc = "a\nbc";
    let mut parser = Parser::new(doc);

    assert_eq!(
        parser.get_position(),
        Position {
            offset: 0,
            line: 1,
            column: 1
        }
    );
    parser.advance(); // 'a'
    assert_eq!(
        parser.get_position(),
        Position {
            offset: 1,
            line: 1,
            column: 2
        }
    );
    parser.advance(); // '\n'
    assert_eq!(
        parser.get_position(),
        Position {
            offset: 2,
            line: 2,
            column: 1
        }
    );
    parser.advance(); // 'b'
    assert_eq!(
        parser.get_position(),
        Position {
            offset: 3,
            line: 2,
            column: 2
        }
    );
}
