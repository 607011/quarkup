use crate::html::HtmlRenderer;
use crate::lexer::Lexer;
use crate::parser::Parser;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
fn init() {
    console_error_panic_hook::set_once();
}

/// Compiles Quarkup source into a standalone HTML document.
///
/// - `template`: optional custom HTML template using the same `{{title}}` /
///   `{{content}}` placeholders as the CLI's `--template` flag. Pass `None`
///   (`undefined` in JS) to use the built-in default template.
/// - `monolithic`: embed local images as Base64 data URIs. Since the browser
///   sandbox has no filesystem access, this only affects images that are
///   already reachable as data URIs or absolute URLs; local paths are left
///   untouched with a console warning, same as a failed embed on the CLI.
/// - `defines`: newline- or semicolon-separated `KEY=VALUE` (or bare `KEY`)
///   entries, equivalent to repeating `-d KEY=VALUE` on the CLI.
#[wasm_bindgen]
pub fn compile(source: &str, template: Option<String>, monolithic: bool, defines: &str) -> String {
    let defines_map = crate::parse_defines(defines.split(['\n', ';']));

    let lexer = Lexer::new(source);
    let parser = Parser::new(lexer, defines_map);
    let ast = parser.parse();

    let renderer = HtmlRenderer::new(template, monolithic);
    renderer.render(&ast)
}
