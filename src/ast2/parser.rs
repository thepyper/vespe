use std::str::Chars;
use super::types::Position; // Import Position from types.rs

#[derive(Debug, Clone)]
pub struct Parser<'a> {
    document: &'a str,
    position: Position,
    iterator: Chars<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(document: &'a str) -> Self {
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

    pub fn advance_immutable(&self) -> Option<(char, Parser<'a>)> {
        let mut new_parser = self.clone();
        if let Some(char) = new_parser.advance() {
            Some((char, new_parser))
        } else {
            None
        }
    }

    pub fn consume_char_if_immutable<F>(&self, filter: F) -> Option<(char, Parser<'a>)> 
    where
        F: FnOnce(char) -> bool,
    {
        let mut new_parser = self.clone();
        match new_parser.consume_char_if(filter) {
            Some(c) => Some((c, new_parser)),
            None => None,
        }
    }

    pub fn consume_matching_char_immutable(&self, x: char) -> Option<Parser<'a>> {
        let mut new_parser = self.clone();
        match new_parser.consume_matching_char(x) {
            Some(_) => Some(new_parser),
            None => None,
        }
    }

    pub fn consume_matching_string_immutable(&self, xs: &str) -> Option<Parser<'a>> {
        let mut new_parser = self.clone();
        match new_parser.consume_matching_string(xs) {
            Some(_) => Some(new_parser),
            None => None,
        }
    }
    
    pub fn consume_many_if_immutable<F>(&self, filter: F) -> (String, Parser<'a>)
    where
        F: Fn(char) -> bool,
    {
        let mut new_parser = self.clone();
        let result = new_parser.consume_many_if(filter).unwrap_or_default();
        (result, new_parser)
    }

    pub fn skip_many_whitespaces_immutable(&self) -> Parser<'a> {
        let mut new_parser = self.clone();
        new_parser.skip_many_whitespaces();
        new_parser
    }

    pub fn skip_many_whitespaces_or_eol_immutable(&self) -> Parser<'a> {
        let mut new_parser = self.clone();
        new_parser.skip_many_whitespaces_or_eol();
        new_parser
    }

    pub fn get_position(&self) -> Position {
        self.position.clone()
    }
    pub fn get_offset(&self) -> usize {
        self.position.offset
    }
    pub fn remain(&self) -> &'a str {
        self.iterator.as_str()
    }
    pub fn is_eod(&self) -> bool {
        self.remain().is_empty()
    }
    pub fn is_eol(&self) -> bool {
        self.remain().starts_with("\n")
    }
    pub fn is_begin_of_line(&self) -> bool {
        self.position.column == 1
    }
    pub fn consume_matching_string(&mut self, xs: &str) -> Option<String> {
        if !self.remain().starts_with(xs) {
            None
        } else {
            for _ in xs.chars() {
                self.advance();
            }
            Some(xs.into())
        }
    }
    pub fn consume_matching_char(&mut self, x: char) -> Option<char> {
        self.consume_char_if(|y| x == y)
    }
    pub fn consume_char_if<F>(&mut self, filter: F) -> Option<char> 
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
    pub fn consume_many_if<F>(&mut self, filter: F) -> Option<String> 
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
    pub fn skip_many_whitespaces(&mut self) {
        let _ = self.consume_many_of(" \t\r");
    }
    pub fn skip_many_whitespaces_or_eol(&mut self) {
        let _ = self.consume_many_of(" \t\r\n");
    }
    pub fn advance(&mut self) -> Option<char> {
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
}
