mod cli;
mod html_writer;
mod parser;
mod router;
mod server;

use clap::Parser;
use cli::{Cli, Commands};
use html_writer::HtmlWriter;
use parser::MarkdownParser;
use server::{Server, ServerConfig};
use std::env;
use std::fs;
use std::io::{self, Write};

// Constants for messages
const ERROR_PREFIX: &str = "Error:";
const STARTING_SERVER: &str = "Starting markdown server...";
const CONTENT_ROOT_PREFIX: &str = "Content root:";
const LISTENING_PREFIX: &str = "Listening on:";
const STOP_MESSAGE: &str = "Press Ctrl+C to stop the server";
const HTTP_PREFIX: &str = "http://";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Serve(args) => handle_serve(args),
        Commands::Render(args) => handle_render(args),
    }
}

/// Handle the 'serve' subcommand
fn handle_serve(args: cli::ServeArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Determine content root: use provided path or current directory
    let content_root = if let Some(path) = args.path {
        path
    } else {
        env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?
    };

    // Validate content root exists
    if !content_root.exists() {
        return Err(format!("Content root does not exist: {}", content_root.display()).into());
    }

    if !content_root.is_dir() {
        return Err(format!(
            "Content root is not a directory: {}",
            content_root.display()
        )
        .into());
    }

    // Create server configuration
    let config = ServerConfig::new(content_root.clone())
        .with_host(args.host.clone())
        .with_port(args.port);

    // Print startup information to stderr
    eprintln!("{}", STARTING_SERVER);
    eprintln!(
        "{} {}",
        CONTENT_ROOT_PREFIX,
        content_root
            .canonicalize()
            .unwrap_or(content_root)
            .display()
    );
    eprintln!(
        "{} {}{}:{}",
        LISTENING_PREFIX, HTTP_PREFIX, args.host, args.port
    );
    eprintln!();
    eprintln!("{}", STOP_MESSAGE);
    eprintln!();

    // Create and run the server
    let server = Server::new(config)?;
    server.run()?;

    Ok(())
}

/// Handle the 'render' subcommand
fn handle_render(args: cli::RenderArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Validate file exists
    if !args.file.exists() {
        // Write error to stderr
        writeln!(
            io::stderr(),
            "{} File does not exist: {}",
            ERROR_PREFIX,
            args.file.display()
        )?;
        std::process::exit(1);
    }

    if !args.file.is_file() {
        // Write error to stderr
        writeln!(
            io::stderr(),
            "{} Path is not a file: {}",
            ERROR_PREFIX,
            args.file.display()
        )?;
        std::process::exit(1);
    }

    // Read the markdown file
    let content = fs::read_to_string(&args.file).map_err(|e| {
        // Write error to stderr
        let _ = writeln!(io::stderr(), "{} Failed to read file: {}", ERROR_PREFIX, e);
        e
    })?;

    // Parse the markdown
    let document = MarkdownParser::parse(&content).map_err(|e| {
        // Write error to stderr
        let _ = writeln!(io::stderr(), "{} Parse error: {}", ERROR_PREFIX, e);
        e
    })?;

    // Convert to HTML
    let writer = HtmlWriter::new();
    let html = writer.write_html(&document).map_err(|e| {
        // Write error to stderr
        let _ = writeln!(
            io::stderr(),
            "{} HTML generation error: {}",
            ERROR_PREFIX,
            e
        );
        e
    })?;

    // Output HTML to stdout
    println!("{}", html);

    Ok(())
}
