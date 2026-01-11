use clap::{Parser, Subcommand};
use std::path::PathBuf;

// Constants for help text
const ABOUT_TEXT: &str = "Directory-based markdown website server and renderer";
const SERVE_ABOUT: &str = "Start the markdown website server";
const RENDER_ABOUT: &str = "Render a markdown file to HTML";
const PATH_HELP: &str = "Directory containing markdown files";
const HOST_HELP: &str = "Host address to bind to";
const PORT_HELP: &str = "Port to listen on";
const FILE_HELP: &str = "Markdown file to render";

const DEFAULT_HOST: &str = "0.0.0.0";
const DEFAULT_PORT: &str = "8080";

/// Directory-based markdown website server and renderer
#[derive(Parser, Debug)]
#[command(name = "wtf")]
#[command(about = ABOUT_TEXT, long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the markdown website server
    #[command(about = SERVE_ABOUT)]
    Serve(ServeArgs),

    /// Render a markdown file to HTML
    #[command(about = RENDER_ABOUT)]
    Render(RenderArgs),
}

#[derive(Parser, Debug)]
pub struct ServeArgs {
    /// Directory containing markdown files [default: current directory]
    #[arg(value_name = "PATH", help = PATH_HELP)]
    pub path: Option<PathBuf>,

    /// Host address to bind to
    #[arg(long, default_value = DEFAULT_HOST, help = HOST_HELP)]
    pub host: String,

    /// Port to listen on
    #[arg(long, short = 'p', default_value = DEFAULT_PORT, help = PORT_HELP)]
    pub port: u16,
}

#[derive(Parser, Debug)]
pub struct RenderArgs {
    /// Markdown file to render
    #[arg(value_name = "FILE", help = FILE_HELP)]
    pub file: PathBuf,
}
