use crate::lexer::Flavor;

#[derive(Debug, PartialEq, Clone)]
pub enum InlineNode {
    Text(String),
    Formatted {
        flavor: Flavor,
        content: Vec<InlineNode>,
    },
    InlineMath(String),
    Link {
        url: String,
        text: Vec<InlineNode>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct ListItem {
    pub level: usize,
    pub content: Vec<InlineNode>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum RowType {
    Header,
    Body,
    Footer,
    Section,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CellAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, PartialEq, Clone)]
pub struct LatticeCell {
    pub content: Vec<InlineNode>,
    pub colspan: usize,
    pub rowspan: usize,
    pub is_merged: bool,
    pub is_colspan_marker: bool,
    pub is_rowspan_marker: bool,
    pub align: CellAlign,
}

#[derive(Debug, PartialEq, Clone)]
pub struct LatticeRow {
    pub row_type: RowType,
    pub cells: Vec<LatticeCell>,
}

#[derive(Debug, PartialEq, Clone)]
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
    Lattice(Vec<LatticeRow>),
    Conditional(Vec<BlockNode>), // Dynamic wrapper for active branches
}

#[derive(Debug, PartialEq, Clone)]
pub struct Document {
    pub blocks: Vec<BlockNode>,
}
