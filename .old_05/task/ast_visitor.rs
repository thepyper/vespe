use markdown::mdast::*;

#[allow(unused_variables)]
pub trait Visitor {
    // Container Nodes
    fn pre_visit_root(&mut self, root: &Root) {}
    fn post_visit_root(&mut self, root: &Root) {}
    fn pre_visit_blockquote(&mut self, blockquote: &Blockquote) {}
    fn post_visit_blockquote(&mut self, blockquote: &Blockquote) {}
    fn pre_visit_footnote_definition(&mut self, footnote_definition: &FootnoteDefinition) {}
    fn post_visit_footnote_definition(&mut self, footnote_definition: &FootnoteDefinition) {}
    fn pre_visit_list(&mut self, list: &List) {}
    fn post_visit_list(&mut self, list: &List) {}
    fn pre_visit_list_item(&mut self, list_item: &ListItem) {}
    fn post_visit_list_item(&mut self, list_item: &ListItem) {}
    fn pre_visit_heading(&mut self, heading: &Heading) {}
    fn post_visit_heading(&mut self, heading: &Heading) {}
    fn pre_visit_table(&mut self, table: &Table) {}
    fn post_visit_table(&mut self, table: &Table) {}
    fn pre_visit_table_row(&mut self, table_row: &TableRow) {}
    fn post_visit_table_row(&mut self, table_row: &TableRow) {}
    fn pre_visit_table_cell(&mut self, table_cell: &TableCell) {}
    fn post_visit_table_cell(&mut self, table_cell: &TableCell) {}
    fn pre_visit_paragraph(&mut self, paragraph: &Paragraph) {}
    fn post_visit_paragraph(&mut self, paragraph: &Paragraph) {}
    fn pre_visit_delete(&mut self, delete: &Delete) {}
    fn post_visit_delete(&mut self, delete: &Delete) {}
    fn pre_visit_emphasis(&mut self, emphasis: &Emphasis) {}
    fn post_visit_emphasis(&mut self, emphasis: &Emphasis) {}
    fn pre_visit_link(&mut self, link: &Link) {}
    fn post_visit_link(&mut self, link: &Link) {}
    fn pre_visit_link_reference(&mut self, link_reference: &LinkReference) {}
    fn post_visit_link_reference(&mut self, link_reference: &LinkReference) {}
    fn pre_visit_strong(&mut self, strong: &Strong) {}
    fn post_visit_strong(&mut self, strong: &Strong) {}
    fn pre_visit_mdx_jsx_flow_element(&mut self, element: &MdxJsxFlowElement) {}
    fn post_visit_mdx_jsx_flow_element(&mut self, element: &MdxJsxFlowElement) {}
    fn pre_visit_mdx_jsx_text_element(&mut self, element: &MdxJsxTextElement) {}
    fn post_visit_mdx_jsx_text_element(&mut self, element: &MdxJsxTextElement) {}

    // Leaf Nodes
    fn visit_code(&mut self, code: &Code) {}
    fn visit_math(&mut self, math: &Math) {}
    fn visit_thematic_break(&mut self, thematic_break: &ThematicBreak) {}
    fn visit_definition(&mut self, definition: &Definition) {}
    fn visit_break(&mut self, r#break: &Break) {}
    fn visit_inline_code(&mut self, inline_code: &InlineCode) {}
    fn visit_inline_math(&mut self, inline_math: &InlineMath) {}
    fn visit_footnote_reference(&mut self, footnote_reference: &FootnoteReference) {}
    fn visit_html(&mut self, html: &Html) {}
    fn visit_image(&mut self, image: &Image) {}
    fn visit_image_reference(&mut self, image_reference: &ImageReference) {}
    fn visit_text(&mut self, text: &Text) {}
    fn visit_mdxjs_esm(&mut self, mdxjs_esm: &MdxjsEsm) {}
    fn visit_mdx_text_expression(&mut self, mdx_text_expression: &MdxTextExpression) {}
    fn visit_mdx_flow_expression(&mut self, mdx_flow_expression: &MdxFlowExpression) {}
    fn visit_toml(&mut self, toml: &Toml) {}
    fn visit_yaml(&mut self, yaml: &Yaml) {}
}

pub fn walk<'a, V: Visitor>(visitor: &mut V, node: &'a Node) {
    match node {
        Node::Root(root) => {
            visitor.pre_visit_root(root);
            for child in &root.children {
                walk(visitor, child);
            }
            visitor.post_visit_root(root);
        }
        Node::Blockquote(blockquote) => {
            visitor.pre_visit_blockquote(blockquote);
            for child in &blockquote.children {
                walk(visitor, child);
            }
            visitor.post_visit_blockquote(blockquote);
        }
        Node::FootnoteDefinition(footnote_definition) => {
            visitor.pre_visit_footnote_definition(footnote_definition);
            for child in &footnote_definition.children {
                walk(visitor, child);
            }
            visitor.post_visit_footnote_definition(footnote_definition);
        }
        Node::List(list) => {
            visitor.pre_visit_list(list);
            for child in &list.children {
                walk(visitor, child);
            }
            visitor.post_visit_list(list);
        }
        Node::ListItem(list_item) => {
            visitor.pre_visit_list_item(list_item);
            for child in &list_item.children {
                walk(visitor, child);
            }
            visitor.post_visit_list_item(list_item);
        }
        Node::Heading(heading) => {
            visitor.pre_visit_heading(heading);
            for child in &heading.children {
                walk(visitor, child);
            }
            visitor.post_visit_heading(heading);
        }
        Node::Table(table) => {
            visitor.pre_visit_table(table);
            for child in &table.children {
                walk(visitor, child);
            }
            visitor.post_visit_table(table);
        }
        Node::TableRow(table_row) => {
            visitor.pre_visit_table_row(table_row);
            for child in &table_row.children {
                walk(visitor, child);
            }
            visitor.post_visit_table_row(table_row);
        }
        Node::TableCell(table_cell) => {
            visitor.pre_visit_table_cell(table_cell);
            for child in &table_cell.children {
                walk(visitor, child);
            }
            visitor.post_visit_table_cell(table_cell);
        }
        Node::Paragraph(paragraph) => {
            visitor.pre_visit_paragraph(paragraph);
            for child in &paragraph.children {
                walk(visitor, child);
            }
            visitor.post_visit_paragraph(paragraph);
        }
        Node::Delete(delete) => {
            visitor.pre_visit_delete(delete);
            for child in &delete.children {
                walk(visitor, child);
            }
            visitor.post_visit_delete(delete);
        }
        Node::Emphasis(emphasis) => {
            visitor.pre_visit_emphasis(emphasis);
            for child in &emphasis.children {
                walk(visitor, child);
            }
            visitor.post_visit_emphasis(emphasis);
        }
        Node::Link(link) => {
            visitor.pre_visit_link(link);
            for child in &link.children {
                walk(visitor, child);
            }
            visitor.post_visit_link(link);
        }
        Node::LinkReference(link_reference) => {
            visitor.pre_visit_link_reference(link_reference);
            for child in &link_reference.children {
                walk(visitor, child);
            }
            visitor.post_visit_link_reference(link_reference);
        }
        Node::Strong(strong) => {
            visitor.pre_visit_strong(strong);
            for child in &strong.children {
                walk(visitor, child);
            }
            visitor.post_visit_strong(strong);
        }
        Node::MdxJsxFlowElement(element) => {
            visitor.pre_visit_mdx_jsx_flow_element(element);
            for child in &element.children {
                walk(visitor, child);
            }
            visitor.post_visit_mdx_jsx_flow_element(element);
        }
        Node::MdxJsxTextElement(element) => {
            visitor.pre_visit_mdx_jsx_text_element(element);
            for child in &element.children {
                walk(visitor, child);
            }
            visitor.post_visit_mdx_jsx_text_element(element);
        }

        // Leaf nodes
        Node::Code(code) => visitor.visit_code(code),
        Node::Math(math) => visitor.visit_math(math),
        Node::ThematicBreak(thematic_break) => visitor.visit_thematic_break(thematic_break),
        Node::Definition(definition) => visitor.visit_definition(definition),
        Node::Break(r#break) => visitor.visit_break(r#break),
        Node::InlineCode(inline_code) => visitor.visit_inline_code(inline_code),
        Node::InlineMath(inline_math) => visitor.visit_inline_math(inline_math),
        Node::FootnoteReference(footnote_reference) => visitor.visit_footnote_reference(footnote_reference),
        Node::Html(html) => visitor.visit_html(html),
        Node::Image(image) => visitor.visit_image(image),
        Node::ImageReference(image_reference) => visitor.visit_image_reference(image_reference),
        Node::Text(text) => visitor.visit_text(text),
        Node::MdxjsEsm(mdxjs_esm) => visitor.visit_mdxjs_esm(mdxjs_esm),
        Node::MdxTextExpression(mdx_text_expression) => visitor.visit_mdx_text_expression(mdx_text_expression),
        Node::MdxFlowExpression(mdx_flow_expression) => visitor.visit_mdx_flow_expression(mdx_flow_expression),
        Node::Toml(toml) => visitor.visit_toml(toml),
        Node::Yaml(yaml) => visitor.visit_yaml(yaml),
    }
}