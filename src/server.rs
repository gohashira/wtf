use crate::html_writer::HtmlWriter;
use crate::parser::{InlineNode, MarkdownParser};
use crate::router::{ResolvedPath, Router, RouterError};
use std::fmt;
use std::fs;
use std::path::PathBuf;
use tiny_http::{Response, Server as TinyServer};

// HTML document structure constants
const DOCTYPE: &str = "<!DOCTYPE html>";
const HTML_OPEN: &str = "<html>";
const HTML_CLOSE: &str = "</html>";
const HEAD_OPEN: &str = "<head>";
const HEAD_CLOSE: &str = "</head>";
const META_CHARSET: &str = "<meta charset=\"utf-8\">";
const TITLE_OPEN: &str = "<title>";
const TITLE_CLOSE: &str = "</title>";
const BODY_OPEN: &str = "<body>";
const BODY_CLOSE: &str = "</body>";

// HTTP status codes
const HTTP_STATUS_OK: u16 = 200;
const HTTP_STATUS_NOT_FOUND: u16 = 404;
const HTTP_STATUS_INTERNAL_ERROR: u16 = 500;

// Content type header
const CONTENT_TYPE_HTML: &str = "text/html; charset=utf-8";

// Generic error pages
const GENERIC_404_TITLE: &str = "404 Not Found";
const GENERIC_404_BODY: &str =
    "<h1>404 Not Found</h1><p>The requested page could not be found.</p>";
const GENERIC_500_TITLE: &str = "500 Internal Server Error";
const GENERIC_500_BODY: &str =
    "<h1>500 Internal Server Error</h1><p>An error occurred while processing your request.</p>";

// Default fallback title
const DEFAULT_TITLE: &str = "Page";

// Server info
const SERVER_START_MESSAGE: &str = "Server started successfully";
const SERVER_ADDRESS_PREFIX: &str = "Listening on";

// ============================================================================
// ERROR HANDLING
// ============================================================================

#[derive(Debug)]
pub enum ServerError {
    BindError { address: String, source: String },
    RouterError { source: RouterError },
    IoError { path: PathBuf, source: String },
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::BindError { address, source } => {
                write!(f, "Failed to bind to {}: {}", address, source)
            }
            ServerError::RouterError { source } => {
                write!(f, "Router error: {}", source)
            }
            ServerError::IoError { path, source } => {
                write!(f, "IO error reading {}: {}", path.display(), source)
            }
        }
    }
}

impl std::error::Error for ServerError {}

impl From<RouterError> for ServerError {
    fn from(err: RouterError) -> Self {
        ServerError::RouterError { source: err }
    }
}

// ============================================================================
// SERVER CONFIGURATION
// ============================================================================

/// Configuration for the web server
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Absolute path to the directory containing markdown files
    pub content_root: PathBuf,
    /// Host address to bind to (default: "0.0.0.0")
    pub host: String,
    /// Port to listen on (default: 8080)
    pub port: u16,
}

impl ServerConfig {
    /// Creates a new server configuration with the specified content root
    ///
    /// # Arguments
    /// * `content_root` - Path to the directory containing markdown files
    ///
    /// # Returns
    /// A new `ServerConfig` with default host (0.0.0.0) and port (8080)
    pub fn new(content_root: PathBuf) -> Self {
        Self {
            content_root,
            host: "0.0.0.0".to_string(),
            port: 8080,
        }
    }

    /// Sets the host address
    pub fn with_host(mut self, host: String) -> Self {
        self.host = host;
        self
    }

    /// Sets the port
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Returns the server address in "host:port" format
    fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

// ============================================================================
// SERVER
// ============================================================================

/// HTTP server for serving directory-based markdown websites
///
/// The server:
/// - Uses a `Router` to resolve URL paths to markdown files
/// - Parses markdown files using `MarkdownParser`
/// - Converts to HTML using `HtmlWriter`
/// - Wraps content in a complete HTML5 document structure
/// - Supports hierarchical 404.md error pages
pub struct Server {
    router: Router,
    config: ServerConfig,
}

impl Server {
    /// Creates a new server with the specified configuration
    ///
    /// # Arguments
    /// * `config` - Server configuration including content root, host, and port
    ///
    /// # Returns
    /// * `Ok(Server)` - Successfully created server
    /// * `Err(ServerError)` - If router initialization fails
    pub fn new(config: ServerConfig) -> Result<Self, ServerError> {
        let router = Router::new(config.content_root.clone())?;

        Ok(Self { router, config })
    }

    /// Starts the HTTP server and begins handling requests
    ///
    /// This is a blocking call that runs until interrupted (Ctrl+C)
    ///
    /// # Returns
    /// * `Ok(())` - Server shut down successfully
    /// * `Err(ServerError)` - If server fails to start or bind to address
    pub fn run(&self) -> Result<(), ServerError> {
        let address = self.config.address();

        let server = TinyServer::http(&address).map_err(|e| ServerError::BindError {
            address: address.clone(),
            source: e.to_string(),
        })?;

        println!("{}", SERVER_START_MESSAGE);
        println!("{} http://{}", SERVER_ADDRESS_PREFIX, address);

        for request in server.incoming_requests() {
            let url_path = request.url().to_string();

            match self.handle_request(&url_path) {
                Ok((status, html)) => {
                    let response = Response::from_string(html)
                        .with_status_code(status)
                        .with_header(
                            tiny_http::Header::from_bytes(
                                &b"Content-Type"[..],
                                &CONTENT_TYPE_HTML.as_bytes()[..],
                            )
                            .unwrap(),
                        );

                    if let Err(e) = request.respond(response) {
                        eprintln!("Error sending response: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Error handling request for {}: {}", url_path, e);

                    let html = wrap_html_document(GENERIC_500_TITLE, GENERIC_500_BODY);
                    let response = Response::from_string(html)
                        .with_status_code(HTTP_STATUS_INTERNAL_ERROR)
                        .with_header(
                            tiny_http::Header::from_bytes(
                                &b"Content-Type"[..],
                                &CONTENT_TYPE_HTML.as_bytes()[..],
                            )
                            .unwrap(),
                        );

                    if let Err(e) = request.respond(response) {
                        eprintln!("Error sending error response: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Handles a single HTTP request
    ///
    /// # Arguments
    /// * `url_path` - The URL path from the HTTP request
    ///
    /// # Returns
    /// * `Ok((status_code, html))` - HTTP status and rendered HTML
    /// * `Err(ServerError)` - If an error occurs during processing
    fn handle_request(&self, url_path: &str) -> Result<(u16, String), ServerError> {
        // Try to resolve the path
        let resolved = self.router.resolve_path(url_path)?;

        match resolved {
            ResolvedPath::Found(path) => {
                // Read and render the markdown file
                self.render_markdown_file(&path, HTTP_STATUS_OK, url_path)
            }
            ResolvedPath::NotFound { .. } => {
                // Try to find a 404.md file
                if let Some(not_found_path) = self.router.resolve_404(url_path) {
                    // Render the custom 404 page (footer already added in render_markdown_file)
                    self.render_markdown_file(&not_found_path, HTTP_STATUS_NOT_FOUND, url_path)
                } else {
                    // Use generic 404 page with footer
                    let sitemap = self.router.build_sitemap()?;
                    let footer_html =
                        crate::router::generate_sitemap_footer(&sitemap, Some(url_path));
                    let body_with_footer = format!("{}{}", GENERIC_404_BODY, footer_html);
                    let html = wrap_html_document(GENERIC_404_TITLE, &body_with_footer);
                    Ok((HTTP_STATUS_NOT_FOUND, html))
                }
            }
        }
    }

    /// Reads, parses, and renders a markdown file to HTML
    ///
    /// # Arguments
    /// * `path` - Path to the markdown file
    /// * `status_code` - HTTP status code to return
    /// * `url_path` - The URL path being requested (for "you're here" indicator)
    ///
    /// # Returns
    /// * `Ok((status_code, html))` - HTTP status and rendered HTML document
    /// * `Err(ServerError)` - If reading or parsing fails
    fn render_markdown_file(
        &self,
        path: &PathBuf,
        status_code: u16,
        url_path: &str,
    ) -> Result<(u16, String), ServerError> {
        // Read the markdown file
        let content = fs::read_to_string(path).map_err(|e| ServerError::IoError {
            path: path.clone(),
            source: e.to_string(),
        })?;

        // Parse the markdown
        let document = MarkdownParser::parse(&content).map_err(|e| ServerError::IoError {
            path: path.clone(),
            source: e.to_string(),
        })?;

        // Extract title from first heading (if available)
        let title = extract_title(&document);

        // Convert to HTML
        let writer = HtmlWriter::new();
        let body_html = writer
            .write_html(&document)
            .map_err(|e| ServerError::IoError {
                path: path.clone(),
                source: e.to_string(),
            })?;

        // Generate sitemap footer with current path indicator
        let sitemap = self.router.build_sitemap()?;
        let footer_html = crate::router::generate_sitemap_footer(&sitemap, Some(url_path));

        // Combine body and footer
        let complete_body = format!("{}{}", body_html, footer_html);

        // Wrap in HTML document structure
        let html = wrap_html_document(&title, &complete_body);

        Ok((status_code, html))
    }
}

// ============================================================================
// HTML DOCUMENT GENERATION
// ============================================================================

/// Wraps content in a complete HTML5 document structure
///
/// # Arguments
/// * `title` - Text for the <title> tag
/// * `content` - HTML content for the <body>
///
/// # Returns
/// A complete HTML document string
fn wrap_html_document(title: &str, content: &str) -> String {
    format!(
        "{}{}{}{}{}{}{}{}{}{}{}{}",
        DOCTYPE,
        HTML_OPEN,
        HEAD_OPEN,
        META_CHARSET,
        TITLE_OPEN,
        escape_html_title(title),
        TITLE_CLOSE,
        HEAD_CLOSE,
        BODY_OPEN,
        content,
        BODY_CLOSE,
        HTML_CLOSE
    )
}

/// Extracts the title from a document (first H1 heading text)
///
/// # Arguments
/// * `document` - Parsed markdown document
///
/// # Returns
/// Title string (from first H1, or default if none found)
fn extract_title(document: &crate::parser::Document) -> String {
    // Check if first section exists and is H1
    if let Some(section) = document.sections().first() {
        if section.level() == 1 {
            return inline_nodes_to_text(section.title());
        }
    }

    DEFAULT_TITLE.to_string()
}

/// Converts inline nodes to plain text (for title extraction)
///
/// # Arguments
/// * `nodes` - Slice of inline nodes
///
/// # Returns
/// Plain text representation (formatting removed)
fn inline_nodes_to_text(nodes: &[InlineNode]) -> String {
    let mut text = String::new();

    for node in nodes {
        match node {
            InlineNode::Text(t) => text.push_str(t),
            InlineNode::LineBreak => text.push(' '),
            InlineNode::Bold(children) => text.push_str(&inline_nodes_to_text(children)),
            InlineNode::Italic(children) => text.push_str(&inline_nodes_to_text(children)),
            InlineNode::Link {
                text: link_text, ..
            } => text.push_str(&inline_nodes_to_text(link_text)),
        }
    }

    text
}

/// Escapes HTML entities in title text
///
/// Note: This is simpler than the full escaping in html_writer.rs
/// since titles are plain text only
fn escape_html_title(title: &str) -> String {
    title
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_new() {
        let config = ServerConfig::new(PathBuf::from("/test"));
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert_eq!(config.address(), "0.0.0.0:8080");
    }

    #[test]
    fn test_server_config_with_host() {
        let config = ServerConfig::new(PathBuf::from("/test")).with_host("127.0.0.1".to_string());
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.address(), "127.0.0.1:8080");
    }

    #[test]
    fn test_server_config_with_port() {
        let config = ServerConfig::new(PathBuf::from("/test")).with_port(3000);
        assert_eq!(config.port, 3000);
        assert_eq!(config.address(), "0.0.0.0:3000");
    }

    #[test]
    fn test_wrap_html_document() {
        let html = wrap_html_document("Test Title", "<p>Content</p>");
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>Test Title</title>"));
        assert!(html.contains("<p>Content</p>"));
        assert!(html.contains("<meta charset=\"utf-8\">"));
    }

    #[test]
    fn test_escape_html_title() {
        let escaped = escape_html_title("<script>alert('XSS')</script>");
        assert_eq!(escaped, "&lt;script&gt;alert(&#39;XSS&#39;)&lt;/script&gt;");
    }

    #[test]
    fn test_inline_nodes_to_text() {
        let nodes = vec![
            InlineNode::Text("Hello ".to_string()),
            InlineNode::Bold(vec![InlineNode::Text("world".to_string())]),
            InlineNode::Text("!".to_string()),
        ];

        let text = inline_nodes_to_text(&nodes);
        assert_eq!(text, "Hello world!");
    }

    #[test]
    fn test_extract_title_with_h1() {
        let markdown = "# My Title\n\nContent here";
        let document = MarkdownParser::parse(markdown).unwrap();
        let title = extract_title(&document);
        assert_eq!(title, "My Title");
    }

    #[test]
    fn test_extract_title_no_h1() {
        let markdown = "Just a paragraph";
        let document = MarkdownParser::parse(markdown).unwrap();
        let title = extract_title(&document);
        assert_eq!(title, DEFAULT_TITLE);
    }
}
