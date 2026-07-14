use crate::lexer::Flavor;

#[derive(Debug, PartialEq)]
pub enum InlineNode {
    Text(String),
    Formatted {
        flavor: Flavor,
        content: Vec<InlineNode>,
    },
    InlineMath(String),
}

#[derive(Debug, PartialEq)]
pub struct ListItem {
    pub level: usize,
    pub content: Vec<InlineNode>,
}

#[derive(Debug, PartialEq)]
pub enum BlockNode {
    Heading {
        level: usize,
        content: Vec<InlineNode>,
    },
    Paragraph(Vec<InlineNode>),
    Metadata {
        key: String,
        value: String,
    },
    Image {
        path: String,
        caption: Option<String>,
    },
    CodeBlock {
        language: Option<String>,
        code: String,
    },
    MathBlock(String),
    List {
        ordered: bool,
        items: Vec<ListItem>,
    },
}

#[derive(Debug, PartialEq)]
pub struct Document {
    pub blocks: Vec<BlockNode>,
}