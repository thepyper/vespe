use std::str::Chars;
use super::types::Position; // Import Position from types.rs

#[derive(Debug, Clone)]
pub(crate) struct Parser<'a> {
    document: &'a str,
    position: Position,
    iterator: Chars<'a>,
}

impl<'a> Parser<'a> {
    pub(crate) fn new(document: &'a str) -> Self {
        Self {
            document,
            position: Position {
                offset: 0,
                line: 1,
                column: 1,
            },
            iterator: document.chars(),
        }
    }

    pub(crate) fn advance_immutable(&self) -> Option<(char, Parser<'a>)> {
        let mut new_parser = self.clone();
        if let Some(char) = new_parser.advance() {
            Some((char, new_parser))
        } else {
            None
        }
    }

    pub(crate) fn consume_char_if_immutable<F>(&self, filter: F) -> Option<(char, Parser<'a>)> 
    where
        F: FnOnce(char) -> bool,
    {
        let mut new_parser = self.clone();
        match new_parser.consume_char_if(filter) {
            Some(c) => Some((c, new_parser)),
            None => None,
        }
    }

    pub(crate) fn consume_matching_char_immutable(&self, x: char) -> Option<Parser<'a>> {
        let mut new_parser = self.clone();
        match new_parser.consume_matching_char(x) {
            Some(_) => Some(new_parser),
            None => None,
        }
    }

    pub(crate) fn consume_matching_string_immutable(&self, xs: &str) -> Option<Parser<'a>> {
        let mut new_parser = self.clone();
        match new_parser.consume_matching_string(xs) {
            Some(_) => Some(new_parser),
            None => None,
        }
    }
    
    pub(crate) fn consume_many_if_immutable<F>(&self, filter: F) -> (String, Parser<'a>)
    where
        F: Fn(char) -> bool,
    {
        let mut new_parser = self.clone();
        let result = new_parser.consume_many_if(filter).unwrap_or_default();
        (result, new_parser)
    }

    pub(crate) fn skip_many_whitespaces_immutable(&self) -> Parser<'a> {
        let mut new_parser = self.clone();
        new_parser.skip_many_whitespaces();
        new_parser
    }

    pub(crate) fn skip_many_whitespaces_or_eol_immutable(&self) -> Parser<'a> {
        let mut new_parser = self.clone();
        new_parser.skip_many_whitespaces_or_eol();
        new_parser
    }

    pub(crate) fn get_position(&self) -> Position {
        self.position.clone()
    }
    pub(crate) fn get_offset(&self) -> usize {
        self.position.offset
    }
    pub(crate) fn remain(&self) -> &'a str {
        self.iterator.as_str()
    }
    pub(crate) fn is_eod(&self) -> bool {
        self.remain().is_empty()
    }
    pub(crate) fn is_eol(&self) -> bool {
        self.remain().starts_with("\n")
    }
    pub(crate) fn is_begin_of_line(&self) -> bool {
        self.position.column == 1
    }
    pub(crate) fn consume_matching_string(&mut self, xs: &str) -> Option<String> {
        if !self.remain().starts_with(xs) {
            None
        } else {
            for _ in xs.chars() {
                self.advance();
            }
            Some(xs.into())
        }
    }
    pub(crate) fn consume_matching_char(&mut self, x: char) -> Option<char> {
        self.consume_char_if(|y| x == y)
    }
    pub(crate) fn consume_char_if<F>(&mut self, filter: F) -> Option<char> 
    where
        F: FnOnce(char) -> bool,
    {
        match self.remain().chars().next() {
            None => None,
            Some(y) => {
                if !filter(y) {
                    None
                } else {
                    self.advance()
                }
            }
        }
    }
    pub(crate) fn consume_many_if<F>(&mut self, filter: F) -> Option<String> 
    where
        F: Fn(char) -> bool,
    {
        let mut xs = String::new();
        loop {
            match self.consume_char_if(|c| filter(c)) {
                None => {
                    break;
                }
                Some(x) => xs.push(x),
            }
        }
        if xs.is_empty() {
            None
        } else {
            Some(xs)
        }
    }
    fn consume_many_of(&mut self, xs: &str) -> Option<String> {
        self.consume_many_if(|y| xs.contains(y))
    }
    pub(crate) fn skip_many_whitespaces(&mut self) {
        let _ = self.consume_many_of(" \t\r");
    }
    pub(crate) fn skip_many_whitespaces_or_eol(&mut self) {
        let _ = self.consume_many_of(" \t\r\n");
    }
    pub(crate) fn advance(&mut self) -> Option<char> {
        match self.iterator.next() {
            None => None,
            Some(c) => {
                self.position.offset += 1;
                if c == '\n' {
                    self.position.line += 1;
                    self.position.column = 1;
                } else {
                    self.position.column += 1;
                }
                Some(c)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Parser;

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

    use super::Position;

    #[test]
    fn test_parser_new() {
        let doc = "hello";
        let parser = Parser::new(doc);
        assert_eq!(parser.get_offset(), 0);
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
        assert_eq!(parser.get_offset(), 1);
        assert_eq!(parser.get_position().line, 1);
        assert_eq!(parser.get_position().column, 2);
        assert_eq!(parser.remain(), "bc\ndef");

        // 'b'
        assert_eq!(parser.advance(), Some('b'));
        assert_eq!(parser.get_offset(), 2);
        assert_eq!(parser.get_position().line, 1);
        assert_eq!(parser.get_position().column, 3);
        assert_eq!(parser.remain(), "c\ndef");

        // 'c'
        assert_eq!(parser.advance(), Some('c'));
        assert_eq!(parser.get_offset(), 3);
        assert_eq!(parser.get_position().line, 1);
        assert_eq!(parser.get_position().column, 4);
        assert_eq!(parser.remain(), "\ndef");

        // '\n'
        assert_eq!(parser.advance(), Some('\n'));
        assert_eq!(parser.get_offset(), 4);
        assert_eq!(parser.get_position().line, 2);
        assert_eq!(parser.get_position().column, 1);
        assert_eq!(parser.remain(), "def");
        assert!(parser.is_begin_of_line());
        assert!(!parser.is_eol()); // is_eol checks if the *next* char is EOL

        // 'd'
        assert_eq!(parser.advance(), Some('d'));
        assert_eq!(parser.get_offset(), 5);
        assert_eq!(parser.get_position().line, 2);
        assert_eq!(parser.get_position().column, 2);
        assert_eq!(parser.remain(), "ef");

        // 'e'
        assert_eq!(parser.advance(), Some('e'));
        assert_eq!(parser.get_offset(), 6);
        assert_eq!(parser.get_position().line, 2);
        assert_eq!(parser.get_position().column, 3);
        assert_eq!(parser.remain(), "f");

        // 'f'
        assert_eq!(parser.advance(), Some('f'));
        assert_eq!(parser.get_offset(), 7);
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
        assert_eq!(p1.get_offset(), 1);
        assert_eq!(p1.remain(), "b");
        assert_eq!(parser.remain(), "ab"); // Original parser is unchanged

        let (c2, p2) = p1.advance_immutable().unwrap();
        assert_eq!(c2, 'b');
        assert_eq!(p2.get_offset(), 2);
        assert_eq!(p2.remain(), "");

        assert!(p2.advance_immutable().is_none());
    }

    #[test]
    fn test_parser_is_eol() {
        let doc = "line1\nline2";
        let mut parser = Parser::new(doc);

        // Advance to just before '\n'
        parser.advance(); // 'l'
        parser.advance(); // 'i'
        parser.advance(); // 'n'
        parser.advance(); // 'e'
        parser.advance(); // '1'

        assert!(parser.is_eol());
        parser.advance(); // Consume '\n'
        assert!(!parser.is_eol());

        let doc_no_eol = "no_eol";
        let parser_no_eol = Parser::new(doc_no_eol);
        assert!(!parser_no_eol.is_eol());
    }

    #[test]
    fn test_parser_get_position() {
        let doc = "a\nbc";
        let mut parser = Parser::new(doc);

        assert_eq!(parser.get_position(), Position { offset: 0, line: 1, column: 1 });
        parser.advance(); // 'a'
        assert_eq!(parser.get_position(), Position { offset: 1, line: 1, column: 2 });
        parser.advance(); // '\n'
        assert_eq!(parser.get_position(), Position { offset: 2, line: 2, column: 1 });
        parser.advance(); // 'b'
        assert_eq!(parser.get_position(), Position { offset: 3, line: 2, column: 2 });
    }

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

        assert_eq!(parser.consume_matching_string("hello"), Some("hello".to_string()));
        assert_eq!(parser.remain(), " world");

        assert_eq!(parser.consume_matching_string("foo"), None);
        assert_eq!(parser.remain(), " world");

        assert_eq!(parser.consume_matching_string(" world"), Some(" world".to_string()));
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

        assert_eq!(parser.consume_many_if(|c| c == 'a'), Some("aaa".to_string()));
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
}
