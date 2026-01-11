# wtf - Directory-Based Markdown Server

A clean, fast command-line tool for serving directory-based markdown websites and rendering markdown files to HTML.

## Features

- 🚀 **Fast HTTP server** for markdown websites
- 📁 **Directory-based routing** - intuitive URL structure
- 🔄 **Hierarchical 404 pages** - custom errors per section
- 🎨 **Clean HTML output** - semantic, minified markup
- 🛡️ **Security built-in** - path validation, HTML escaping
- 📝 **CLI renderer** - convert markdown to HTML via command line
- ✅ **Well-tested** - 106 comprehensive tests

## Quick Start

### Build

```bash
cargo build --release
```

### Serve a Website

```bash
# Serve current directory
wtf serve

# Serve specific directory
wtf serve ./my_site

# Custom host and port
wtf serve --host 127.0.0.1 --port 3000
```

Then visit: http://localhost:8080

### Render Markdown to HTML

```bash
# Output to stdout
wtf render file.md

# Save to file
wtf render input.md > output.html
```

## Usage

### Commands

| Command | Description |
|---------|-------------|
| `wtf serve [PATH]` | Start web server for markdown site |
| `wtf render <FILE>` | Render markdown file to HTML |
| `wtf --help` | Show help message |
| `wtf --version` | Show version |

### `wtf serve` Options

| Option | Default | Description |
|--------|---------|-------------|
| `[PATH]` | current dir | Directory containing markdown files |
| `--host <HOST>` | `0.0.0.0` | Host address to bind to |
| `--port <PORT>` | `8080` | Port to listen on |

### `wtf render` Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<FILE>` | Yes | Markdown file to render |

Output goes to **stdout**, errors to **stderr** for clean piping.

## Directory Structure

The server uses a directory-based routing system:

```
my_site/
├── root.md              → http://localhost:8080/
├── 404.md               → Global 404 page
├── home.md              → http://localhost:8080/home
└── home/
    ├── home.md          → http://localhost:8080/home (directory index)
    ├── about.md         → http://localhost:8080/home/about
    └── about/
        ├── about.md     → http://localhost:8080/home/about
        ├── 404.md       → Custom 404 for /home/about/*
        └── team.md      → http://localhost:8080/home/about/team
```

### Routing Rules

1. **Root path** `/` → `root.md`
2. **Regular files** `/home` → `home.md`
3. **Directory index** `/home` → `home/home.md` (if home.md doesn't exist)
4. **Nested paths** `/home/about/team` → `home/about/team.md`

### 404 Fallback

404 pages use hierarchical fallback:

For `/home/about/missing`:
1. Try `home/about/missing/404.md`
2. Try `home/about/404.md`
3. Try `home/404.md`
4. Try `404.md` (root)
5. Generic 404 message

## Examples

### Local Development

```bash
cd my-blog
wtf serve
```

### Production-like Testing

```bash
wtf serve --host 0.0.0.0 --port 80  # Requires sudo on Unix
```

### Generate Static HTML

```bash
#!/bin/bash
for md in content/**/*.md; do
  html="dist/${md%.md}.html"
  mkdir -p "$(dirname "$html")"
  wtf render "$md" > "$html"
done
```

### Preview Markdown

```bash
wtf render draft.md | w3m -T text/html
```

## Architecture

```
src/
├── cli.rs         - Command-line interface (clap)
├── main.rs        - Entry point and command handlers
├── router.rs      - Path resolution logic
├── server.rs      - HTTP server (tiny_http)
├── parser.rs      - Markdown parser
└── html_writer.rs - HTML generator
```

### Design Principles

- **Clean separation** - Router handles paths, Server handles HTTP
- **Security first** - Path validation, HTML escaping
- **Type safety** - Custom error types, no panics
- **Idiomatic Rust** - No string literals, named constants
- **Well-tested** - Unit, integration, and doc tests

## Testing

Run all tests:

```bash
cargo test
```

Run specific test suite:

```bash
cargo test --test integration_test
cargo test --test server_integration_test
```

**Test Coverage:**
- ✅ 44 unit tests (router, server, parser, html_writer)
- ✅ 45 integration tests (end-to-end scenarios)
- ✅ 17 server integration tests (routing, 404 handling)
- ✅ Total: 106 tests, all passing

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `tiny_http` | 0.12 | Lightweight HTTP server |
| `clap` | 4.5 | Command-line argument parsing |
| `tempfile` | 3.8 | Test fixtures (dev only) |

## Security

- ✅ **Path traversal prevention** - Blocks `..` and `.` components
- ✅ **Path validation** - Ensures paths stay within content root
- ✅ **HTML escaping** - Prevents XSS attacks
- ✅ **Input sanitization** - Validates all user inputs
- ✅ **Error handling** - No panics, graceful error messages

## Documentation

- **[CLI.md](CLI.md)** - Complete CLI usage guide
- **[ROUTING.md](ROUTING.md)** - Routing system documentation
- **[SERVER_README.md](SERVER_README.md)** - Server architecture details

## Demo

A demo site is included in `demo_site/`:

```bash
wtf serve demo_site
```

Visit:
- http://localhost:8080/ - Homepage
- http://localhost:8080/home - Home section
- http://localhost:8080/home/about - About page
- http://localhost:8080/home/about/team - Team page
- http://localhost:8080/nonexistent - Custom 404

## Library Usage

You can also use `wtf` as a library:

```rust
use wtf::{Server, ServerConfig};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig::new(PathBuf::from("./content"))
        .with_host("0.0.0.0".to_string())
        .with_port(8080);
    
    let server = Server::new(config)?;
    server.run()?;
    
    Ok(())
}
```

## License

[Add your license here]

## Contributing

[Add contribution guidelines here]

## Roadmap

Possible future enhancements:

- [ ] CSS/static file serving
- [ ] Front matter metadata parsing
- [ ] Template system (layouts, partials)
- [ ] Live reload in development mode
- [ ] Static site generation mode
- [ ] Table of contents generation
- [ ] Syntax highlighting for code blocks
- [ ] Full HTML document wrapping in `render` command

## Acknowledgments

Built with Rust 🦀 for speed, safety, and reliability.
