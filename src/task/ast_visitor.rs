
use markdown::mdast::*;

#[allow(unused_variables)]
pub trait Visitor {
    fn visit_node(&mut self, node: &Node) {}
    fn visit_root(&mut self, root: &Root) {}
    fn visit_blockquote(&mut self, blockquote: &Blockquote) {}
    fn visit_footnote_definition(&mut self, footnote_definition: &FootnoteDefinition) {}
    fn visit_list(&mut self, list: &List) {}
    fn visit_list_item(&mut self, list_item: &ListItem) {}
    fn visit_code(&mut self, code: &Code) {}
    fn visit_math(&mut self, math: &Math) {}
    fn visit_heading(&mut self, heading: &Heading) {}
    fn visit_table(&mut self, table: &Table) {}
    fn visit_table_row(&mut self, table_row: &TableRow) {}
    fn visit_table_cell(&mut self, table_cell: &TableCell) {}
    fn visit_thematic_break(&mut self, thematic_break: &ThematicBreak) {}
    fn visit_definition(&mut self, definition: &Definition) {}
    fn visit_paragraph(&mut self, paragraph: &Paragraph) {}
    fn visit_break(&mut self, r#break: &Break) {}
    fn visit_inline_code(&mut self, inline_code: &InlineCode) {}
    fn visit_inline_math(&mut self, inline_math: &InlineMath) {}
    fn visit_delete(&mut self, delete: &Delete) {}
    fn visit_emphasis(&mut self, emphasis: &Emphasis) {}
    fn visit_footnote_reference(&mut self, footnote_reference: &FootnoteReference) {}
    fn visit_html(&mut self, html: &Html) {}
    fn visit_image(&mut self, image: &Image) {}
    fn visit_image_reference(&mut self, image_reference: &ImageReference) {}
    fn visit_link(&mut self, link: &Link) {}
    fn visit_link_reference(&mut self, link_reference: &LinkReference) {}
    fn visit_strong(&mut self, strong: &Strong) {}
    fn visit_text(&mut self, text: &Text) {}
    fn visit_mdx_jsx_flow_element(&mut self, mdx_jsx_flow_element: &MdxJsxFlowElement) {}
    fn visit_mdxjs_esm(&mut self, mdxjs_esm: &MdxjsEsm) {}
    fn visit_mdx_text_expression(&mut self, mdx_text_expression: &MdxTextExpression) {}
    fn visit_mdx_jsx_text_element(&mut self, mdx_jsx_text_element: &MdxJsxTextElement) {}
    fn visit_mdx_flow_expression(&mut self, mdx_flow_expression: &MdxFlowExpression) {}
    fn visit_toml(&mut self, toml: &Toml) {}
    fn visit_yaml(&mut self, yaml: &Yaml) {}
}

pub fn walk<'a, V: Visitor>(visitor: &mut V, node: &'a Node) {
    visitor.visit_node(node);
    match node {
        Node::Root(root) => {
            visitor.visit_root(root);
            for child in &root.children {
                walk(visitor, child);
            }
        }
        Node::Blockquote(blockquote) => {
            visitor.visit_blockquote(blockquote);
            for child in &blockquote.children {
                walk(visitor, child);
            }
        }
        Node::FootnoteDefinition(footnote_definition) => {
            visitor.visit_footnote_definition(footnote_definition);
            for child in &footnote_definition.children {
                walk(visitor, child);
            }
        }
        Node::List(list) => {
            visitor.visit_list(list);
            for child in &list.children {
                walk(visitor, child);
            }
        }
        Node::ListItem(list_item) => {
            visitor.visit_list_item(list_item);
            for child in &list_item.children {
                walk(visitor, child);
            }
        }
        Node::Code(code) => visitor.visit_code(code),
        Node::Math(math) => visitor.visit_math(math),
        Node::Heading(heading) => {
            visitor.visit_heading(heading);
            for child in &heading.children {
                walk(visitor, child);
            }
        }
        Node::Table(table) => {
            visitor.visit_table(table);
            for child in &table.children {
                walk(visitor, child);
            }
        }
        Node::TableRow(table_row) => {
            visitor.visit_table_row(table_row);
            for child in &table_row.children {
                walk(visitor, child);
            }
        }
        Node::TableCell(table_cell) => {
            visitor.visit_table_cell(table_cell);
            for child in &table_cell.children {
                walk(visitor, child);
            }
        }
        Node::ThematicBreak(thematic_break) => visitor.visit_thematic_break(thematic_break),
        Node::Definition(definition) => visitor.visit_definition(definition),
        Node::Paragraph(paragraph) => {
            visitor.visit_paragraph(paragraph);
            for child in &paragraph.children {
                walk(visitor, child);
            }
        }
        Node::Break(r#break) => visitor.visit_break(r#break),
        Node::InlineCode(inline_code) => visitor.visit_inline_code(inline_code),
        Node::InlineMath(inline_math) => visitor.visit_inline_math(inline_math),
        Node::Delete(delete) => {
            visitor.visit_delete(delete);
            for child in &delete.children {
                walk(visitor, child);
            }
        }
        Node::Emphasis(emphasis) => {
            visitor.visit_emphasis(emphasis);
            for child in &emphasis.children {
                walk(visitor, child);
            }
        }
        Node::FootnoteReference(footnote_reference) => visitor.visit_footnote_reference(footnote_reference),
        Node::Html(html) => visitor.visit_html(html),
        Node::Image(image) => visitor.visit_image(image),
        Node::ImageReference(image_reference) => visitor.visit_image_reference(image_reference),
        Node::Link(link) => {
            visitor.visit_link(link);
            for child in &link.children {
                walk(visitor, child);
            }
        }
        Node::LinkReference(link_reference) => {
            visitor.visit_link_reference(link_reference);
            for child in &link_reference.children {
                walk(visitor, child);
            }
        }
        Node::Strong(strong) => {
            visitor.visit_strong(strong);
            for child in &strong.children {
                walk(visitor, child);
            }
        }
        Node::Text(text) => visitor.visit_text(text),
        Node::MdxJsxFlowElement(mdx_jsx_flow_element) => {
            visitor.visit_mdx_jsx_flow_element(mdx_jsx_flow_element);
            for child in &mdx_jsx_flow_element.children {
                walk(visitor, child);
            }
        }
        Node::MdxjsEsm(mdxjs_esm) => visitor.visit_mdxjs_esm(mdxjs_esm),
        Node::MdxTextExpression(mdx_text_expression) => visitor.visit_mdx_text_expression(mdx_text_expression),
        Node::MdxJsxTextElement(mdx_jsx_text_element) => {
            visitor.visit_mdx_jsx_text_element(mdx_jsx_text_element);
            for child in &mdx_jsx_text_element.children {
                walk(visitor, child);
            }
        }
        Node::MdxFlowExpression(mdx_flow_expression) => visitor.visit_mdx_flow_expression(mdx_flow_expression),
        Node::Toml(toml) => visitor.visit_toml(toml),
        Node::Yaml(yaml) => visitor.visit_yaml(yaml),
    }
}
