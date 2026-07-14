mod lexer;
mod ast;
mod parser;
mod html;

use std::fs;
use std::io::{self, Read};
use clap::Parser as ClapParser;
use lexer::Lexer;
use parser::Parser;
use html::HtmlRenderer;

#[derive(ClapParser, Debug)]
#[command(
    name = "quarkup",
    author = "Oliver Lau",
    version = "1.0",
    about = "A subatomic document compiler that translates Quarkup markup into HTML",
    long_about = "Quarkup is a lightweight, highly efficient document compiler written in Rust. It processes input from stdin and renders standalone, beautifully styled HTML documents."
)]
struct Args {
    #[arg(
        short,
        long,
        value_name = "FILE",
        help = "Path to a custom HTML standalone template"
    )]
    template: Option<String>,

    #[arg(
        short,
        long,
        value_name = "FILE",
        help = "Path where the rendered HTML output should be written. If omitted, writes to stdout."
    )]
    output: Option<String>,

    #[arg(
        short,
        long,
        help = "Embed local images as Base64 data URIs directly inside the HTML document to make it 100% self-contained"
    )]
    monolithic: bool,

    #[arg(
        short,
        long,
        help = "Print verbose compilation phase logs to stderr"
    )]
    verbose: bool,
}

fn main() -> io::Result<()> {
    // Parse command line arguments using clap.
    // This automatically handles -h, --help, -? and invalid arguments.
    let args = Args::parse();

    if args.verbose {
        eprintln!("[Info] Starting Quarkup compiler...");
        if args.monolithic {
            eprintln!("[Info] Monolithic mode active: local images will be embedded as Base64.");
        }
        if let Some(ref path) = args.output {
            eprintln!("[Info] Output will be written to: {}", path);
        } else {
            eprintln!("[Info] No output file specified. Outputting to stdout.");
        }
    }

    // Load custom template if specified
    let mut custom_template = None;
    if let Some(ref template_path) = args.template {
        if args.verbose {
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
    }

    if args.verbose {
        eprintln!("[Info] Reading source document from stdin...");
    }

    // Read the .qu source code from stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    if args.verbose {
        eprintln!("[Info] Tokenizing source with Lexer...");
    }
    let lexer = Lexer::new(&input);

    if args.verbose {
        eprintln!("[Info] Parsing tokens into Abstract Syntax Tree (AST)...");
    }
    let parser = Parser::new(lexer);
    let ast = parser.parse();

    if args.verbose {
        eprintln!("[Info] AST generation complete. Blocks found: {}", ast.blocks.len());
        eprintln!("[Info] Rendering AST to HTML...");
    }
    
    let renderer = HtmlRenderer::new(custom_template, args.monolithic);
    let html_output = renderer.render(&ast);

    // Handle output target
    if let Some(ref path) = args.output {
        if args.verbose {
            eprintln!("[Info] Writing rendered HTML to file: {}", path);
        }
        fs::write(path, html_output)?;
        if args.verbose {
            eprintln!("[Info] File successfully written.");
        }
    } else {
        if args.verbose {
            eprintln!("[Info] Writing rendered HTML to stdout.");
        }
        println!("{}", html_output);
    }

    if args.verbose {
        eprintln!("[Info] Process complete.");
    }

    Ok(())
}