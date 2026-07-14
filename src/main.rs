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
    let args: Vec<String> = env::args().collect();
    let mut custom_template = None;
    let mut monolithic = false;
    let mut verbose = false;
    let mut output_path = None;

    // Parse command line arguments
    let mut i = 1;
    while i < args.len() {
        if (args[i] == "--template" || args[i] == "-t") && i + 1 < args.len() {
            let template_path = &args[i + 1];
            if verbose {
                eprintln!("[Info] Reading custom HTML template from: {}", template_path);
            }
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
        } else if (args[i] == "--output" || args[i] == "-o") && i + 1 < args.len() {
            output_path = Some(args[i + 1].clone());
            i += 2;
        } else if args[i] == "--monolithic" || args[i] == "-m" {
            monolithic = true;
            i += 1;
        } else if args[i] == "-v" || args[i] == "--verbose" {
            verbose = true;
            i += 1;
        } else {
            i += 1;
        }
    }

    if verbose {
        eprintln!("[Info] Starting Quarkup compiler...");
        if monolithic {
            eprintln!("[Info] Monolithic mode active: local images will be embedded as Base64.");
        }
        if let Some(ref path) = output_path {
            eprintln!("[Info] Output will be written to: {}", path);
        } else {
            eprintln!("[Info] No output file specified. Outputting to stdout.");
        }
        eprintln!("[Info] Reading source document from stdin...");
    }

    // Read the .qu source code from stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    if verbose {
        eprintln!("[Info] Tokenizing source with Lexer...");
    }
    let lexer = Lexer::new(&input);

    if verbose {
        eprintln!("[Info] Parsing tokens into Abstract Syntax Tree (AST)...");
    }
    let parser = Parser::new(lexer);
    let ast = parser.parse();

    if verbose {
        eprintln!("[Info] AST generation complete. Blocks found: {}", ast.blocks.len());
        eprintln!("[Info] Rendering AST to HTML...");
    }
    
    let renderer = HtmlRenderer::new(custom_template, monolithic);
    let html_output = renderer.render(&ast);

    // Handle output target
    if let Some(path) = output_path {
        if verbose {
            eprintln!("[Info] Writing rendered HTML to file: {}", path);
        }
        fs::write(&path, html_output)?;
        if verbose {
            eprintln!("[Info] File successfully written.");
        }
    } else {
        if verbose {
            eprintln!("[Info] Writing rendered HTML to stdout.");
        }
        println!("{}", html_output);
    }

    if verbose {
        eprintln!("[Info] Process complete.");
    }

    Ok(())
}