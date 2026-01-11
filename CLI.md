# CLI Usage Guide

`wtf` is a command-line tool for serving directory-based markdown websites and rendering markdown files to HTML.

## Installation

Build from source:
```bash
cargo build --release
```

The binary will be at `target/release/wtf`.

## Commands

### `wtf serve` - Start Web Server

Start a web server that serves a directory-based markdown website.

#### Basic Usage

```bash
# Serve current directory on 0.0.0.0:8080
wtf serve

# Serve specific directory
wtf serve ./my_site

# Custom host and port
wtf serve --host 127.0.0.1 --port 3000

# Combined options
wtf serve ./demo_site --host 0.0.0.0 --port 8000
```

#### Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--host <HOST>` | | Host address to bind to | `0.0.0.0` |
| `--port <PORT>` | `-p` | Port to listen on | `8080` |
| `--help` | `-h` | Show help message | |

#### Arguments

- `[PATH]` - Optional directory containing markdown files (default: current directory)

#### Examples

**Serve current directory:**
```bash
wtf serve
```

Output (to stderr):
```
Starting markdown server...
Content root: /Users/name/current/directory
Listening on: http://0.0.0.0:8080

Press Ctrl+C to stop the server
```

**Serve on localhost only:**
```bash
wtf serve --host 127.0.0.1
```

**Serve on custom port:**
```bash
wtf serve -p 3000
```

**Serve specific directory:**
```bash
cd /path/to/parent
wtf serve ./my_blog --port 8080
```

#### Error Handling

**Directory doesn't exist:**
```bash
$ wtf serve /nonexistent
Error: Content root does not exist: /nonexistent
```

**Path is not a directory:**
```bash
$ wtf serve file.md
Error: Content root is not a directory: file.md
```

**Port already in use:**
```bash
$ wtf serve --port 8080
# (if port 8080 is already taken)
Error: Failed to bind to 0.0.0.0:8080: address already in use
```

---

### `wtf render` - Render Markdown to HTML

Render a markdown file to HTML and output to stdout.

#### Basic Usage

```bash
# Render file to stdout
wtf render file.md

# Redirect to file
wtf render input.md > output.html

# Pipe to other commands
wtf render doc.md | less
```

#### Arguments

- `<FILE>` - Required markdown file to render

#### Options

| Option | Short | Description |
|--------|-------|-------------|
| `--help` | `-h` | Show help message |

#### Examples

**Render to stdout:**
```bash
$ wtf render demo_site/root.md
<h1>Welcome to My Site</h1><p>This is the homepage...</p>
```

**Save to file:**
```bash
wtf render input.md > output.html
```

**Render multiple files:**
```bash
for file in docs/*.md; do
  wtf render "$file" > "html/$(basename "$file" .md).html"
done
```

#### Output

The command outputs:
- **Stdout**: Minified HTML (no newlines, no DOCTYPE, just body content)
- **Stderr**: Error messages (if any)

This allows clean piping to other commands without mixing output and errors.

#### Error Handling

**File doesn't exist:**
```bash
$ wtf render nonexistent.md
Error: File does not exist: nonexistent.md
```
(Exit code: 1)

**Path is not a file:**
```bash
$ wtf render ./directory
Error: Path is not a file: ./directory
```
(Exit code: 1)

**Parse error:**
```bash
$ wtf render malformed.md
Error: Parse error: Unclosed delimiter '**' at position 42
```
(Exit code: 1)

---

## Global Options

| Option | Short | Description |
|--------|-------|-------------|
| `--help` | `-h` | Print help |
| `--version` | `-V` | Print version |

### Examples

**Show help:**
```bash
wtf --help
wtf serve --help
wtf render --help
```

**Show version:**
```bash
$ wtf --version
wtf 0.1.0
```

---

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | Error (file not found, parse error, etc.) |
| `101` | Panic (unexpected error) |

---

## Environment

### Current Directory

The `serve` command uses the current working directory as the default content root:

```bash
# These are equivalent:
cd /path/to/site && wtf serve
wtf serve /path/to/site
```

### Output Redirection

**Separate stdout and stderr:**
```bash
# Only HTML to file, errors to terminal
wtf render input.md > output.html

# Errors to file, HTML to terminal
wtf render input.md 2> errors.log

# Both to separate files
wtf render input.md > output.html 2> errors.log

# Silence errors
wtf render input.md 2>/dev/null
```

---

## Common Use Cases

### Local Development Server

```bash
cd my-website
wtf serve
# Visit http://localhost:8080
```

### Build Static Site

```bash
#!/bin/bash
# Generate HTML from all markdown files
mkdir -p dist
for md in content/**/*.md; do
  html="dist/${md%.md}.html"
  mkdir -p "$(dirname "$html")"
  wtf render "$md" > "$html"
done
```

### Preview Single File

```bash
wtf render draft.md | w3m -T text/html
```

### Integration with Other Tools

```bash
# With pandoc for additional processing
wtf render input.md | pandoc -f html -t docx -o output.docx

# With tidy for formatted output
wtf render input.md | tidy -indent -wrap 80

# With grep for content search
wtf render *.md | grep "search term"
```

---

## Directory Structure for `serve`

The server expects a directory-based structure. See [ROUTING.md](ROUTING.md) for details.

**Example:**
```
my_site/
├── root.md              → http://localhost:8080/
├── 404.md               → Global 404 page
└── home/
    ├── home.md          → http://localhost:8080/home
    └── about/
        ├── about.md     → http://localhost:8080/home/about
        └── team.md      → http://localhost:8080/home/about/team
```

---

## Troubleshooting

### Server won't start

**Problem:** `Failed to bind to 0.0.0.0:8080`
- **Cause:** Port already in use
- **Solution:** Use different port: `wtf serve -p 8081`

**Problem:** `Permission denied`
- **Cause:** Ports < 1024 require root on Unix
- **Solution:** Use port ≥ 1024: `wtf serve -p 8080`

### Can't access server from other devices

**Problem:** Server running on `127.0.0.1` not accessible from network
- **Solution:** Use `0.0.0.0`: `wtf serve --host 0.0.0.0`

### Render produces no output

**Problem:** Command succeeds but no output
- **Check:** Is the markdown file empty?
- **Check:** Are you redirecting stderr to stdout accidentally?

### HTML looks wrong

**Problem:** Missing styles
- **Note:** The render command outputs plain HTML without CSS
- **Solution:** Add your own stylesheet or use a framework

---

## Tips

1. **Use absolute paths** when serving from different directories:
   ```bash
   wtf serve ~/websites/blog
   ```

2. **Bind to localhost** for security when developing:
   ```bash
   wtf serve --host 127.0.0.1
   ```

3. **Check rendered output** before deploying:
   ```bash
   wtf render page.md | less
   ```

4. **Script automation** with proper error handling:
   ```bash
   if wtf render input.md > output.html 2>errors.log; then
     echo "Success!"
   else
     cat errors.log
     exit 1
   fi
   ```

---

## See Also

- [ROUTING.md](ROUTING.md) - Directory-based routing documentation
- [SERVER_README.md](SERVER_README.md) - Server architecture and features
