pub mod cli;
pub mod html_writer;
pub mod parser;
pub mod router;
pub mod server;

// Re-export main types for convenience
pub use router::{ResolvedPath, Router, RouterError};
pub use server::{Server, ServerConfig, ServerError};
