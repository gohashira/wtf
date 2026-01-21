# CLAUDE.md - AI Assistant Guide for WTF

## Project Overview

**WTF** is a directory-based markdown website server and renderer written in Rust. It serves markdown files from a directory structure as a live website with automatic HTML conversion, hierarchical navigation, and intelligent routing.

### Key Features
- **Directory-based routing**: Maps URL paths to markdown files using filesystem conventions
- **Live HTTP server**: Serves markdown content as HTML on-demand
- **Static rendering**: Can render individual markdown files to HTML via CLI
- **Hierarchical navigation**: Automatically generates sitemaps from directory structure
- **Custom 404 pages**: Supports hierarchical 404.md files with fallback logic
- **Security**: Path traversal protection and HTML entity escaping

### Binary Name
The project name is `wtf` but the compiled binary is named `gohashira_wtf` (see Cargo.toml and Taskfile.yaml).

---

## Architecture Overview

The codebase follows a clean, layered architecture with clear separation of concerns:

```
┌─────────────────────────────────────────────────────┐
│                     CLI Layer                        │
│                   (cli.rs, main.rs)                  │
└─────────────────┬───────────────────────────────────┘
                  │
       ┌──────────┴───────────┐
       │                      │
┌──────▼──────┐      ┌────────▼────────┐
│   Server    │      │     Render      │
│  (server.rs)│      │   (main.rs)     │
└──────┬──────┘      └────────┬────────┘
       │                      │
       │             ┌────────┴────────┐
       │             │                 │
┌──────▼──────┐ ┌───▼──────┐  ┌──────▼────────┐
│   Router    │ │  Parser  │  │  HtmlWriter   │
│ (router.rs) │ │(parser.rs)│ │(html_writer.rs)│
└─────────────┘ └──────────┘  └───────────────┘
```

### Layer Responsibilities

1. **CLI Layer** (`cli.rs`, `main.rs`)
   - Parses command-line arguments using clap
   - Dispatches to serve or render commands
   - Handles process lifecycle

2. **Server Layer** (`server.rs`)
   - HTTP server using tiny_http
   - Request routing and response generation
   - HTML document wrapping with `<head>`, `<body>`, etc.
   - Title extraction from H1 headings
   - Sitemap footer injection

3. **Router Layer** (`router.rs`)
   - URL-to-filesystem path resolution
   - Security validation (prevents directory traversal)
   - Hierarchical 404.md resolution
   - Sitemap generation from directory structure
   - HTML footer generation with "you're here" indicators

4. **Parser Layer** (`parser.rs`)
   - Markdown parsing to AST (Abstract Syntax Tree)
   - Supports: headings, paragraphs, bold, italic, links, images, line breaks
   - Hierarchical section tree construction
   - Error handling for malformed markdown

5. **HTML Writer Layer** (`html_writer.rs`)
   - Converts parsed markdown AST to HTML
   - HTML entity escaping for security
   - Minified output (no extra whitespace)
   - Semantic HTML tags

---

## Module Documentation

### `src/lib.rs`
Re-exports main types for library usage:
- `Router`, `ResolvedPath`, `RouterError`
- `Server`, `ServerConfig`, `ServerError`

### `src/main.rs`
Entry point for the CLI application:
- Parses CLI args using `Cli::parse()`
- Dispatches to `handle_serve()` or `handle_render()`
- **Constants**: Uses semantic constants for all messages (ERROR_PREFIX, STARTING_SERVER, etc.)
- **Error handling**: Writes errors to stderr, content to stdout

### `src/cli.rs`
CLI argument parsing using clap derive macros:
- **Commands**:
  - `serve [PATH] [--host HOST] [--port PORT]` - Start HTTP server
  - `render FILE` - Render markdown to HTML
- **Defaults**: host=0.0.0.0, port=8080
- **All help text defined as constants**

### `src/router.rs`
Directory-based routing with security:

**Key Types**:
- `Router` - Main routing logic
- `ResolvedPath` - Enum: `Found(PathBuf)` or `NotFound{attempted_paths}`
- `SitemapEntry` - Tree structure for navigation

**Routing Rules**:
```
URL              →  File Resolution Order
/                →  root.md
/home            →  home/home.md (priority) OR home.md
/home/about      →  home/about/about.md (priority) OR home/about.md
```

**Key Methods**:
- `resolve_path(url_path)` - Resolves URL to markdown file
- `resolve_404(url_path)` - Finds hierarchical 404.md (deepest first)
- `build_sitemap()` - Generates navigation tree
- `generate_sitemap_footer(entries, current_path)` - Creates HTML footer with links

**Security Features**:
- Rejects paths with `..` (parent directory)
- Rejects paths with `.` (current directory)
- Rejects trailing slashes (except root `/`)
- Rejects double slashes
- Validates content_root exists and is a directory
- Canonicalizes paths to prevent symlink attacks

**Special Behavior**:
- Directory index files take priority over standalone files
- Accessing `/home/home` is rejected (prevents duplicate route access)
- Sitemap excludes 404.md files and shadowed standalone files

### `src/parser.rs`
Markdown parser producing a hierarchical AST:

**AST Node Types**:
```rust
Document
├── content: Vec<BlockNode>       // Preamble before first heading
└── sections: Vec<Section>        // Top-level sections

Section
├── level: u8                     // 1-6
├── title: Vec<InlineNode>        // Can contain formatting
├── content: Vec<BlockNode>       // Content under this heading
└── subsections: Vec<Section>     // Nested sections

BlockNode
├── Paragraph(Vec<InlineNode>)
└── Image { alt_text, url }

InlineNode
├── Text(String)
├── LineBreak
├── Bold(Vec<InlineNode>)         // Can nest italic, links
├── Italic(Vec<InlineNode>)       // Can nest bold, links
└── Link { text: Vec<InlineNode>, url: String }
```

**Parsing Algorithm**:
- Recursive descent parser
- Character-by-character scanning
- Hierarchical section tree construction based on heading levels
- Line breaks within paragraphs preserved as `LineBreak` nodes
- Double newlines separate paragraphs

**Key Methods**:
- `MarkdownParser::parse(text)` - Main entry point (static method)
- Returns `Result<Document, ParseError>`

**Error Types**:
- `UnexpectedEndOfInput`
- `UnclosedDelimiter` (bold/italic not closed before newline)
- `InvalidHeadingLevel` (must be 1-6)
- `MalformedLink` / `MalformedImage` (missing brackets/parens)

### `src/html_writer.rs`
Converts parsed markdown AST to minified HTML:

**Key Features**:
- All HTML tags defined as constants (no magic strings)
- Recursive section rendering
- HTML entity escaping: `& < > " '`
- Minified output (no newlines or extra whitespace)

**HTML Mapping**:
```
Markdown           →  HTML
# Heading          →  <h1>Heading</h1>
**bold**           →  <strong>bold</strong>
*italic*           →  <em>italic</em>
[text](url)        →  <a href="url">text</a>
![alt](img.jpg)    →  <img src="img.jpg" alt="alt">
Line break         →  <br>
```

**Key Methods**:
- `write_html(document)` - Main entry point
- Returns `Result<String, HtmlError>`
- Renders preamble, then all sections recursively

**Design Notes**:
- `HtmlConfig` struct exists as placeholder for future blog-specific customization
- Designed to be extended with CSS classes, IDs, custom attributes

### `src/server.rs`
HTTP server using tiny_http:

**Key Types**:
- `Server` - Main server struct
- `ServerConfig` - Builder pattern config (content_root, host, port)
- `ServerError` - Error types for bind/router/IO failures

**Request Handling Flow**:
```
1. Receive HTTP request
2. Extract URL path
3. Router resolves path → ResolvedPath
4. If Found:
   - Read markdown file
   - Parse markdown → Document
   - Convert to HTML
   - Extract title from first H1
   - Generate sitemap footer
   - Wrap in HTML document structure
   - Return 200 OK
5. If NotFound:
   - Try resolve_404() for custom 404.md
   - If found: render 404.md with 404 status
   - Else: generic 404 page with footer
   - Return 404 NOT FOUND
6. On error: Return 500 INTERNAL SERVER ERROR
```

**HTML Document Structure**:
```html
<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>Page Title</title>
</head>
<body>
  [Markdown content as HTML]
  <hr><hr>
  <ul>
    [Sitemap navigation with "← you're here"]
  </ul>
</body>
</html>
```

**Key Methods**:
- `new(config)` - Creates server with router
- `run()` - Blocking call, runs until Ctrl+C
- `handle_request(url_path)` - Returns `(status_code, html)`
- `render_markdown_file(path, status, url_path)` - Full render pipeline

---

## Code Conventions

### Constants Over Magic Strings
**All literal strings must be defined as constants at module level.**

✅ Good:
```rust
const ERROR_PREFIX: &str = "Error:";
eprintln!("{} Failed to read file", ERROR_PREFIX);
```

❌ Bad:
```rust
eprintln!("Error: Failed to read file");
```

### Error Handling Patterns

1. **Custom error types for each module**:
   - `RouterError`, `ServerError`, `ParseError`, `HtmlError`
   - All implement `Display` and `std::error::Error`

2. **Result types throughout**:
   - Never panic in library code
   - Use `Result<T, CustomError>` for all fallible operations

3. **Error context**:
   - Errors include context (file paths, positions, etc.)
   - Use struct-style enum variants for rich error info

### Testing Strategy

**Location**: Tests are co-located with code using `#[cfg(test)] mod tests`

**Coverage**:
- Unit tests in each module (parser.rs, router.rs, html_writer.rs, server.rs)
- Integration tests in `tests/` directory
- Test edge cases, security (path traversal), error conditions

**Key Test Utilities**:
- `tempfile::tempdir()` for filesystem tests
- Helper functions like `create_test_content_root()` in router tests

### Code Organization Patterns

1. **Module structure**:
   ```rust
   // Constants at top
   const SOMETHING: &str = "value";

   // Error types
   pub enum ModuleError { ... }

   // Main types and implementations
   pub struct MainType { ... }

   // Helper functions (private)
   fn helper() { ... }

   // Tests at bottom
   #[cfg(test)]
   mod tests { ... }
   ```

2. **Public API minimal**:
   - Only expose what's needed
   - Use `pub(crate)` for internal APIs
   - Re-export key types in `lib.rs`

3. **Documentation comments**:
   - All public items have `///` doc comments
   - Include `# Arguments`, `# Returns`, `# Example` sections
   - Explain design decisions in regular comments

---

## Development Workflow

### Build Commands

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Build and install using Taskfile
task build
task install  # Installs to ~/.local/bin/
```

### Running the Server

```bash
# Serve current directory on default port (8080)
cargo run -- serve

# Serve specific directory
cargo run -- serve /path/to/content

# Custom host and port
cargo run -- serve --host 127.0.0.1 --port 3000

# Or use the example
cargo run --example serve_demo
```

### Rendering Files

```bash
# Render markdown to HTML (outputs to stdout)
cargo run -- render file.md > output.html
```

### Project Structure
```
wtf/
├── Cargo.toml              # Package manifest
├── Cargo.lock              # Dependency lock file
├── Taskfile.yaml           # Task automation (build, install)
├── .gitignore              # Git ignore patterns
├── .github/
│   └── workflows/
│       └── release.yml     # CI/CD: build on tag push
├── src/
│   ├── lib.rs              # Library entry point
│   ├── main.rs             # Binary entry point
│   ├── cli.rs              # CLI argument parsing
│   ├── router.rs           # Path resolution & routing
│   ├── parser.rs           # Markdown parsing
│   ├── html_writer.rs      # HTML generation
│   └── server.rs           # HTTP server
├── tests/
│   ├── integration_test.rs
│   ├── html_writer_integration_test.rs
│   └── server_integration_test.rs
├── examples/
│   └── serve_demo.rs       # Example usage
└── demo_site/              # Demo content for testing
    ├── root.md
    ├── 404.md
    └── home/
        ├── home.md
        └── about/
            ├── about.md
            └── team.md
```

### Dependencies

**Runtime**:
- `tiny_http = "0.12"` - Lightweight HTTP server
- `clap = { version = "4.5", features = ["derive"] }` - CLI parsing

**Dev Dependencies**:
- `tempfile = "3.8"` - Temporary directories for tests

**Edition**: `2024` (Rust 2024 Edition)

---

## Common Development Tasks

### Adding a New Markdown Feature

1. **Update Parser** (`parser.rs`):
   - Add new AST node type to `InlineNode` or `BlockNode`
   - Add parsing logic in appropriate parse method
   - Add unit tests

2. **Update HTML Writer** (`html_writer.rs`):
   - Add HTML constants for new tags
   - Add rendering logic in `render_block()` or `render_inline()`
   - Add unit tests

3. **Integration Test** (`tests/`):
   - Add end-to-end test parsing + rendering

### Modifying Routing Behavior

1. **Update Router** (`router.rs`):
   - Modify `resolve_path()` logic
   - Update relevant tests
   - Update documentation in this file

2. **Test Security Implications**:
   - Ensure path traversal protection still works
   - Test edge cases with test content structure

### Adding Server Features

1. **Update Server** (`server.rs`):
   - Modify request handling pipeline
   - Update HTML document structure if needed
   - Add configuration to `ServerConfig` if needed

2. **Update CLI** if exposing new options (`cli.rs`)

---

## Security Considerations

### Path Traversal Prevention
**The router validates all paths to prevent directory traversal attacks.**

**Blocked patterns**:
- `../` - Parent directory access
- `./` - Current directory reference
- `//` - Double slashes
- Trailing slashes (except root `/`)

**Implementation**: See `Router::sanitize_path()` in router.rs:167-358

### HTML Escaping
**All user content is escaped before HTML output.**

**Escaped characters**: `& < > " '`

**Implementation**:
- `html_writer::escape_html()` - Content escaping
- `router::escape_html_attr()` - URL attribute escaping
- `router::escape_html_text()` - Text content escaping
- `server::escape_html_title()` - Title tag escaping

### Content-Type Header
All responses include `Content-Type: text/html; charset=utf-8`

---

## Testing Guidelines

### Writing Tests

**Unit Tests**: Test individual functions in isolation
```rust
#[test]
fn test_specific_behavior() {
    let input = "...";
    let result = function(input);
    assert_eq!(result, expected);
}
```

**Integration Tests**: Test full pipelines
```rust
#[test]
fn test_parse_and_render() {
    let markdown = "# Hello";
    let doc = MarkdownParser::parse(markdown).unwrap();
    let html = HtmlWriter::new().write_html(&doc).unwrap();
    assert_eq!(html, "<h1>Hello</h1>");
}
```

**Filesystem Tests**: Use tempfile
```rust
#[test]
fn test_router() {
    let temp_dir = tempfile::tempdir().unwrap();
    fs::write(temp_dir.path().join("root.md"), "content").unwrap();
    let router = Router::new(temp_dir.path().to_path_buf()).unwrap();
    // ... test routing
}
```

### Running Tests
```bash
# All tests
cargo test

# Specific module
cargo test router::tests

# Specific test
cargo test test_resolve_path

# With output
cargo test -- --nocapture

# Integration tests only
cargo test --test '*'
```

---

## Release Process

Releases are automated via GitHub Actions:

1. **Trigger**: Push a git tag matching `v*` (e.g., `v1.0.0`)
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. **Workflow** (`.github/workflows/release.yml`):
   - Builds release binary on Ubuntu
   - Renames to `gohashira_wtf-linux-x86_64`
   - Creates GitHub release with auto-generated notes
   - Attaches binary to release

3. **Binary Name**: `gohashira_wtf-linux-x86_64`

---

## AI Assistant Guidelines

### When Modifying Code

1. **Always read files first** before making changes
2. **Follow constant patterns** - define all strings as constants
3. **Maintain error types** - add variants to existing error enums
4. **Write tests** - add tests for new functionality
5. **Update documentation** - keep this CLAUDE.md current
6. **Check security** - validate any path/URL handling changes

### When Adding Features

1. **Start with AST** - add node types to parser first
2. **Update all layers** - parser → HTML writer → server
3. **Test incrementally** - unit tests → integration tests
4. **Maintain conventions** - match existing code style

### When Debugging

1. **Check tests first** - run `cargo test` to see what breaks
2. **Use error context** - error types include helpful context
3. **Test with demo_site** - use existing test content structure
4. **Validate security** - ensure path validation still works

### Code Review Checklist

- [ ] All strings defined as constants
- [ ] Errors use custom error types with context
- [ ] Public items have documentation comments
- [ ] Tests added for new functionality
- [ ] No panics in library code (use `Result`)
- [ ] HTML entities escaped where user content is output
- [ ] Path validation for any filesystem operations
- [ ] Integration tests pass end-to-end

---

## Troubleshooting

### Common Issues

**Port already in use**:
```bash
# Change port
cargo run -- serve --port 8081
```

**Binary name confusion**:
- Source package name: `wtf`
- Binary name: `wtf` (in development)
- Installed binary: `gohashira_wtf`
- Use `task install` to install to `~/.local/bin/`

**Tests fail with path errors**:
- Ensure using `tempfile::tempdir()` for filesystem tests
- Check that paths are canonicalized in tests

**Build errors after updating Rust**:
- Edition is set to `2024` - requires recent Rust version
- Run `rustup update` if needed

---

## Future Enhancements

The codebase is designed for extensibility:

### Parser (`parser.rs`)
- Add support for code blocks with syntax highlighting
- Add list support (ordered/unordered)
- Add blockquote support
- Add horizontal rule support
- Add table support

### HTML Writer (`html_writer.rs`)
- CSS class injection via `HtmlConfig`
- ID generation for headings (anchor links)
- Table of contents generation
- Custom wrapper elements (article, section)
- Metadata handling (front matter)

### Router (`router.rs`)
- Sitemap XML generation
- RSS feed generation
- Search index building

### Server (`server.rs`)
- Static asset serving (CSS, images, JS)
- Live reload during development
- Custom CSS injection
- Template system

---

## Key Files Reference

| File | Lines | Purpose |
|------|-------|---------|
| `src/router.rs` | 1045 | Routing, security, sitemap |
| `src/parser.rs` | 839 | Markdown parsing to AST |
| `src/server.rs` | 469 | HTTP server, request handling |
| `src/html_writer.rs` | 398 | HTML generation from AST |
| `src/main.rs` | 142 | CLI entry point, command dispatch |
| `src/cli.rs` | 58 | CLI argument parsing |
| `src/lib.rs` | 10 | Library re-exports |

---

## Version Information

- **Rust Edition**: 2024
- **Package Version**: 0.1.0
- **Binary Name**: `gohashira_wtf`
- **Last Updated**: 2026-01-21

---

## Contact & Repository

For questions about this codebase, refer to:
- This CLAUDE.md file
- Inline documentation comments
- Unit tests for usage examples
- Integration tests for end-to-end examples
