use crate::ast::{BlockNode, Document, InlineNode, LatticeRow, ListItem, RowType};
use crate::lexer::Flavor;
use base64::prelude::*;
use std::fs;
use std::path::Path;

use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::html::styled_line_to_highlighted_html;
use syntect::parsing::SyntaxSet;

const DEFAULT_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="{{lang}}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="author" content="{{author}}">
    <meta name="description" content="{{description}}">
    <meta name="keywords" content="{{keywords}}">
    <meta name="copyright" content="{{copyright}}">
    <title>{{title}}</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
            line-height: 1.6;
            max-width: 750px;
            margin: 3rem auto;
            padding: 0 1.5rem;
            color: #24292e;
            background-color: #fafbfc;
        }
        h1, h2, h3 { color: #111; margin-top: 2rem; }
        pre {
            background: #f6f8fa;
            padding: 1rem;
            border-radius: 6px;
            overflow-x: auto;
            border: 1px solid #e1e4e8;
            font-size: 13px;
            line-height: 1.45;
        }
        code {
            font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace;
            font-size: 85%;
        }
        .quarkup-math-block {
            padding: 1rem 0;
        }
        /* Style for the advanced tables (Lattices) */
        table.quarkup-lattice {
            width: 100%;
            border-collapse: collapse;
            margin: 2rem 0;
            font-size: 14px;
        }
        table.quarkup-lattice th, table.quarkup-lattice td {
            border: 1px solid #e1e4e8;
            padding: 10px 12px;
            text-align: left;
        }
        table.quarkup-lattice thead th {
            background-color: #8fb8cf;
            font-weight: 600;
        }
        table.quarkup-lattice tfoot td {
            background-color: #8fb8cf;
            font-style: italic;
            border-top: 2px double #e1e4e8;
        }
        table.quarkup-lattice tr.section-row td {
            background-color: #ebf8ff;
            font-weight: bold;
            color: #2b6cb0;
            text-align: center;
        }
    </style>
</head>
<body>
{{content}}
</body>
</html>"#;

pub struct HtmlRenderer {
    template: String,
    monolithic: bool,
    syntax_set: SyntaxSet,
    theme: Theme,
}

impl HtmlRenderer {
    pub fn new(custom_template: Option<String>, monolithic: bool) -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();

        let theme = theme_set
            .themes
            .get("InspiredGitHub")
            .cloned()
            .unwrap_or_else(|| theme_set.themes.values().next().unwrap().clone());

        Self {
            template: custom_template.unwrap_or_else(|| DEFAULT_TEMPLATE.to_string()),
            monolithic,
            syntax_set,
            theme,
        }
    }

    pub fn render(&self, doc: &Document) -> String {
        let mut content_html = String::new();
        let mut title = String::from("Untitled Quarkup Document");
        let mut author = String::from("Anonymous");
        let mut description = String::new();
        let mut keywords = String::new();
        let mut copyright = String::new();
        let mut lang = String::from("en");

        for block in &doc.blocks {
            if let BlockNode::Metadata { key, value } = block {
                match key.as_str() {
                    "title" => title = value.clone(),
                    "author" => author = value.clone(),
                    "lang" => lang = value.clone(),
                    "description" => description = value.clone(),
                    "keywords" => keywords = value.clone(),
                    "copyright" => copyright = value.clone(),
                    _ => {}
                }
            } else {
                content_html.push_str(&self.render_block(block));
                content_html.push('\n');
            }
        }

        self.template
            .replace("{{title}}", &title)
            .replace("{{author}}", &author)
            .replace("{{lang}}", &lang)
            .replace("{{description}}", &description)
            .replace("{{keywords}}", &keywords)
            .replace("{{copyright}}", &copyright)
            .replace("{{content}}", &content_html)
    }

    fn render_block(&self, block: &BlockNode) -> String {
        match block {
            BlockNode::Heading { level, content } => {
                let tag = format!("h{}", level);
                format!("<{}>{}</{}>", tag, self.render_inline_list(content), tag)
            }
            BlockNode::Paragraph(content) => {
                format!("<p>{}</p>", self.render_inline_list(content))
            }
            BlockNode::Image { path, caption } => {
                let resolved_src = if self.monolithic {
                    match get_image_data_url(path) {
                        Ok(data_url) => data_url,
                        Err(e) => {
                            eprintln!("Warning: Could not embed image '{}': {}", path, e);
                            path.clone()
                        }
                    }
                } else {
                    path.clone()
                };

                if let Some(cap) = caption {
                    format!(
                        "<figure><img src=\"{}\" alt=\"{}\"><figcaption>{}</figcaption></figure>",
                        resolved_src, cap, cap
                    )
                } else {
                    format!("<img src=\"{}\" alt=\"\">", resolved_src)
                }
            }
            BlockNode::CodeBlock { language, code } => {
                let syntax = language
                    .as_ref()
                    .and_then(|lang| self.syntax_set.find_syntax_by_token(lang))
                    .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

                let mut highlighter = HighlightLines::new(syntax, &self.theme);
                let mut highlighted_html = String::new();

                for line in code.lines() {
                    let line_with_nl = format!("{}\n", line);
                    if let Ok(regions) = highlighter.highlight_line(&line_with_nl, &self.syntax_set)
                    {
                        if let Ok(html_line) = styled_line_to_highlighted_html(
                            &regions,
                            syntect::html::IncludeBackground::No,
                        ) {
                            highlighted_html.push_str(&html_line);
                        } else {
                            highlighted_html.push_str(&html_escape(line));
                            highlighted_html.push('\n');
                        }
                    } else {
                        highlighted_html.push_str(&html_escape(line));
                        highlighted_html.push('\n');
                    }
                }

                format!("<pre><code>{}</code></pre>", highlighted_html)
            }
            BlockNode::MathBlock(latex) => {
                let svg = compile_latex_to_svg(latex, true);
                format!(
                    "<div class=\"quarkup-math-block\" style=\"text-align: center; margin: 1.5em 0;\">{}</div>",
                    svg
                )
            }
            BlockNode::List { ordered, items } => self.render_list(*ordered, items),
            BlockNode::Lattice(rows) => self.render_lattice(rows),
            BlockNode::Metadata { .. } => String::new(),
        }
    }

    fn render_lattice(&self, rows: &[LatticeRow]) -> String {
        let mut active_rows = rows.to_vec();

        // Step 1: Calculate colspans (right-to-left scan)
        for row in &mut active_rows {
            if row.row_type == RowType::Section {
                continue;
            }
            let mut col = 0;
            while col < row.cells.len() {
                if row.cells[col].is_colspan_marker {
                    if col > 0 {
                        let mut left_col = col - 1;
                        while left_col > 0 && row.cells[left_col].is_merged {
                            left_col -= 1;
                        }
                        row.cells[left_col].colspan += 1;
                        row.cells[col].is_merged = true;
                    }
                }
                col += 1;
            }
        }

        // Step 2: Calculate rowspans (bottom-up scan with strict bounds checking)
        for row_idx in 0..active_rows.len() {
            if active_rows[row_idx].row_type == RowType::Section {
                continue;
            }
            let num_cells = active_rows[row_idx].cells.len();
            for col_idx in 0..num_cells {
                if active_rows[row_idx].cells[col_idx].is_rowspan_marker {
                    if row_idx > 0 {
                        let mut top_row = row_idx - 1;
                        while top_row > 0 && active_rows[top_row].row_type == RowType::Section {
                            top_row -= 1;
                        }
                        // CRITICAL FIX: Guard against uneven column counts safely
                        if col_idx < active_rows[top_row].cells.len() {
                            let mut target_row = top_row;
                            while target_row > 0
                                && col_idx < active_rows[target_row].cells.len()
                                && active_rows[target_row].cells[col_idx].is_merged
                                && !active_rows[target_row].cells[col_idx].is_colspan_marker
                            {
                                target_row -= 1;
                            }
                            if col_idx < active_rows[target_row].cells.len() {
                                active_rows[target_row].cells[col_idx].rowspan += 1;
                                active_rows[row_idx].cells[col_idx].is_merged = true;
                            }
                        }
                    }
                }
            }
        }

        // Find the maximum column width of the grid safely
        let max_cols = active_rows
            .iter()
            .filter(|r| r.row_type != RowType::Section)
            .map(|r| r.cells.len())
            .max()
            .unwrap_or(0);

        let mut html = String::from("<table class=\"quarkup-lattice\">\n");
        let mut in_thead = false;
        let mut in_tfoot = false;

        for row in &active_rows {
            // Manage clean HTML section wrapping
            if row.row_type == RowType::Header && !in_thead {
                html.push_str("  <thead>\n");
                in_thead = true;
            } else if row.row_type != RowType::Header && in_thead {
                html.push_str("  </thead>\n");
                in_thead = false;
            }

            if row.row_type == RowType::Footer && !in_tfoot {
                html.push_str("  <tfoot>\n");
                in_tfoot = true;
            }

            if row.row_type == RowType::Section {
                html.push_str("  <tr class=\"section-row\">\n");
                let cell_content = if !row.cells.is_empty() {
                    self.render_inline_list(&row.cells[0].content)
                } else {
                    String::new()
                };
                html.push_str(&format!(
                    "    <td colspan=\"{}\">{}</td>\n",
                    max_cols, cell_content
                ));
                html.push_str("  </tr>\n");
            } else {
                html.push_str("  <tr>\n");
                for cell in &row.cells {
                    // If the cell was just a structural placeholder or empty gap, render it cleanly
                    if cell.is_merged {
                        continue;
                    }
                    let tag = if row.row_type == RowType::Header {
                        "th"
                    } else {
                        "td"
                    };
                    let mut attrs = String::new();
                    if cell.colspan > 1 {
                        attrs.push_str(&format!(" colspan=\"{}\"", cell.colspan));
                    }
                    if cell.rowspan > 1 {
                        attrs.push_str(&format!(" rowspan=\"{}\"", cell.rowspan));
                    }

                    let content = self.render_inline_list(&cell.content);
                    html.push_str(&format!("    <{}{} >{}</{}>\n", tag, attrs, content, tag));
                }
                html.push_str("  </tr>\n");
            }
        }

        if in_tfoot {
            html.push_str("  </tfoot>\n");
        }

        html.push_str("</table>\n");
        html
    }

    fn render_list(&self, ordered: bool, items: &[ListItem]) -> String {
        let mut html = String::new();
        let main_tag = if ordered { "ol" } else { "ul" };

        let mut current_level = 0;
        let mut tag_stack = Vec::new();

        for item in items {
            if item.level > current_level {
                while item.level > current_level {
                    html.push_str(&format!("<{}>\n", main_tag));
                    tag_stack.push(main_tag);
                    current_level += 1;
                }
            } else if item.level < current_level {
                while item.level < current_level {
                    if let Some(closed_tag) = tag_stack.pop() {
                        html.push_str(&format!("</{}>\n", closed_tag));
                    }
                    current_level -= 1;
                }
            }

            html.push_str(&format!(
                "<li>{}</li>\n",
                self.render_inline_list(&item.content).trim()
            ));
        }

        while let Some(closed_tag) = tag_stack.pop() {
            html.push_str(&format!("</{}>\n", closed_tag));
        }

        html
    }

    fn render_inline_list(&self, nodes: &[InlineNode]) -> String {
        nodes.iter().map(|node| self.render_inline(node)).collect()
    }

    fn render_inline(&self, node: &InlineNode) -> String {
        match node {
            InlineNode::Text(text) => html_escape(text),
            InlineNode::InlineMath(latex) => {
                let svg = compile_latex_to_svg(latex, false);
                format!(
                    "<span class=\"quarkup-math-inline\" style=\"display: inline-block; vertical-align: -0.15em;\">{}</span>",
                    svg
                )
            }
            InlineNode::Formatted { flavor, content } => {
                let rendered_content = self.render_inline_list(content);
                match flavor {
                    Flavor::Up => format!("<sup>{}</sup>", rendered_content),
                    Flavor::Down => format!("<sub>{}</sub>", rendered_content),
                    Flavor::Charm => format!("<em>{}</em>", rendered_content),
                    Flavor::Strange => format!("<code>{}</code>", rendered_content),
                    Flavor::Neutrino
                    | Flavor::Electron
                    | Flavor::Top
                    | Flavor::Bottom
                    | Flavor::Graphic
                    | Flavor::Lattice => rendered_content,
                }
            }
        }
    }
}

fn compile_latex_to_svg(latex: &str, is_block: bool) -> String {
    let options = mathjax_svg_rs::Options::default();

    match mathjax_svg_rs::render_tex(latex, &options) {
        Ok(mut svg) => {
            if !is_block {
                svg = svg.replace("width=\"", "style=\"height: 1.1em; width: auto;\" width=\"");
            }
            svg
        }
        Err(_) => {
            format!(
                "<span class=\"math-error\">[Math Error: {}]</span>",
                html_escape(latex)
            )
        }
    }
}

fn get_image_data_url(image_path: &str) -> Result<String, std::io::Error> {
    let bytes = fs::read(image_path)?;
    let base64_encoded = BASE64_STANDARD.encode(bytes);

    let path = Path::new(image_path);
    let mime_type = match path.extension().and_then(|ext| ext.to_str()) {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("webp") => "image/webp",
        _ => "application/octet-stream",
    };

    Ok(format!("data:{};base64,{}", mime_type, base64_encoded))
}

fn html_escape(input: &str) -> String {
    let mut escaped = String::new();
    for c in input.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#x27;"),
            _ => escaped.push(c),
        }
    }
    escaped
}
