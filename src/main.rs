mod lexer;
mod ast;
mod parser;
mod html;

use std::env;
use std::fs;
use std::io::{self, Read};
use lexer::Lexer;
use parser::Parser;
use html::HtmlRenderer;

fn main() -> io::Result<()> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let mut custom_template = None;
    let mut monolithic = false;

    let mut i = 1;
    // Check for --template <path> or -t <path>
    while i < args.len() {
        if (args[i] == "--template" || args[i] == "-t") && i + 1 < args.len() {
            let template_path = &args[i + 1];
            match fs::read_to_string(template_path) {
                Ok(content) => {
                    custom_template = Some(content);
                }
                Err(e) => {
                    eprintln!("Error reading template file '{}': {}", template_path, e);
                    std::process::exit(1);
                }
            }
            i += 2;
        } else if args[i] == "--monolithic" || args[i] == "-m" {
            monolithic = true;
            i += 1;
        } else {
            i += 1;
        }
    }

    // Read the .qu source code from stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let lexer = Lexer::new(&input);
    let parser = Parser::new(lexer);
    let ast = parser.parse();

    // Initialize renderer with custom template if provided, otherwise default boilerplate
    let renderer = HtmlRenderer::new(custom_template, monolithic);
    let html_output = renderer.render(&ast);

    println!("{}", html_output);

    Ok(())
}