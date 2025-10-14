use super::{Anchor, Node, ParameterValue, Parameters, Position, Range, Tag, Text};
use std::fmt;

// Helper for indenting output
struct Indent(usize);

impl fmt::Display for Indent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for _ in 0..self.0 {
            write!(f, "    ")?;
        }
        Ok(())
    }
}

// Custom Debug implementation for Position
impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "P(o:{}, l:{}, c:{})", self.offset, self.line, self.column)
    }
}

// Custom Debug implementation for Range
impl fmt::Debug for Range {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?} - {:?}]", self.start, self.end)
    }
}

// Custom Debug implementation for ParameterValue
impl fmt::Debug for ParameterValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParameterValue::String(s) => write!(f, "String("{}")", s),
            ParameterValue::Integer(i) => write!(f, "Integer({})", i),
            ParameterValue::Float(fl) => write!(f, "Float({})", fl),
            ParameterValue::Boolean(b) => write!(f, "Boolean({})", b),
        }
    }
}

// Custom Debug implementation for Parameters
impl fmt::Debug for Parameters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return write!(f, "{{}}");
        }
        writeln!(f, "{{")?;
        for (key, value) in self.iter() {
            writeln!(f, "{}    {}: {:?}", Indent(0), key, value)?;
        }
        write!(f, "}}")
    }
}

// Custom Debug implementation for Tag
impl fmt::Debug for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Tag {{")?;
        writeln!(f, "{}  command: "{}",", Indent(0), self.command)?;
        writeln!(f, "{}  parameters: {:?}", Indent(0), self.parameters)?;
        writeln!(f, "{}  arguments: {:?},", Indent(0), self.arguments)?;
        writeln!(f, "{}  range: {:?}", Indent(0), self.range)?;
        write!(f, "{}}}", Indent(0))
    }
}

// Custom Debug implementation for Anchor
impl fmt::Debug for Anchor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Anchor {{")?;
        writeln!(f, "{}  command: "{}",", Indent(0), self.command)?;
        writeln!(f, "{}  uuid: "{}",", Indent(0), self.uuid)?;
        writeln!(f, "{}  kind: "{}",", Indent(0), self.kind)?;
        writeln!(f, "{}  parameters: {:?}", Indent(0), self.parameters)?;
        writeln!(f, "{}  arguments: {:?},", Indent(0), self.arguments)?;
        writeln!(f, "{}  range: {:?}", Indent(0), self.range)?;
        write!(f, "{}}}", Indent(0))
    }
}

// Custom Debug implementation for Text
impl fmt::Debug for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Text {{")?;
        writeln!(f, "{}  content: "{}",", Indent(0), self.content.trim())?; // Trim for cleaner output
        writeln!(f, "{}  range: {:?}", Indent(0), self.range)?;
        write!(f, "{}}}", Indent(0))
    }
}

// Custom Debug implementation for Node
impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::Tag(tag) => write!(f, "{:?}", tag),
            Node::Anchor(anchor) => write!(f, "{:?}", anchor),
            Node::Text(text) => write!(f, "{:?}", text),
        }
    }
}

// Custom Debug implementation for Root (if you have one, otherwise this can be omitted)
// Assuming Root is a struct with a Vec<Node> field named 'children'
// If Root is not defined, this part will cause a compile error and should be removed.
// For now, I'll assume a Root struct exists or will be created.
// If not, you can just print the Vec<Node> directly.
/*
impl fmt::Debug for Root {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Root {{")?;
        writeln!(f, "{}  children: [", Indent(0))?;
        for child in &self.children {
            writeln!(f, "{}    {:?}", Indent(0), child)?;
        }
        writeln!(f, "{}  ]", Indent(0))?;
        write!(f, "{}}}", Indent(0))
    }
}
*/
