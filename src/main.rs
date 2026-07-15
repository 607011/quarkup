use clap::Parser as ClapParser;
use quarkup::html::HtmlRenderer;
use quarkup::lexer::Lexer;
use quarkup::parser::Parser;
use std::fs;
use std::io::{self, Read};

#[derive(ClapParser, Debug)]
#[command(
    name = "quarkup",
    author = "Oliver",
    version = "1.1",
    about = "A subatomic document compiler that translates Quarkup markup into HTML",
    long_about = "Quarkup is a lightweight, highly efficient document compiler written in Rust."
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

    #[arg(short, long, help = "Print verbose compilation phase logs to stderr")]
    verbose: bool,

    #[arg(
        short,
        long = "define",
        value_name = "KEY=VALUE",
        help = "Define a variable for conditional rendering (e.g., -d target=web or -d draft=true)"
    )]
    defines: Vec<String>,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let defines_map = quarkup::parse_defines(&args.defines);

    if args.verbose {
        eprintln!("[Info] Starting Quarkup compiler...");
        if args.monolithic {
            eprintln!("[Info] Monolithic mode active: local images will be embedded as Base64.");
        }
        if !defines_map.is_empty() {
            eprintln!("[Info] Active CLI definitions: {:?}", defines_map);
        }
        if let Some(ref path) = args.output {
            eprintln!("[Info] Output will be written to: {}", path);
        } else {
            eprintln!("[Info] No output file specified. Outputting to stdout.");
        }
    }

    let mut custom_template = None;
    if let Some(ref template_path) = args.template {
        if args.verbose {
            eprintln!(
                "[Info] Reading custom HTML template from: {}",
                template_path
            );
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

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    if args.verbose {
        eprintln!("[Info] Tokenizing source with Lexer...");
    }
    let lexer = Lexer::new(&input);

    if args.verbose {
        eprintln!("[Info] Parsing tokens into Abstract Syntax Tree (AST)...");
    }
    let parser = Parser::new(lexer, defines_map);
    let ast = parser.parse();

    if args.verbose {
        eprintln!(
            "[Info] AST generation complete. Blocks found: {}",
            ast.blocks.len()
        );
        eprintln!("[Info] Rendering AST to HTML...");
    }

    let renderer = HtmlRenderer::new(custom_template, args.monolithic);
    let html_output = renderer.render(&ast);

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
