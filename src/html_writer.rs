use crate::parser::{BlockNode, Document, InlineNode, Section};
use std::fmt;

// HTML entity escape sequences
const ESCAPE_AMP: &str = "&amp;";
const ESCAPE_LT: &str = "&lt;";
const ESCAPE_GT: &str = "&gt;";
const ESCAPE_QUOT: &str = "&quot;";
const ESCAPE_APOS: &str = "&#39;";

// HTML tag constants - no string literals flying around
const TAG_H1_OPEN: &str = "<h1>";
const TAG_H1_CLOSE: &str = "</h1>";
const TAG_H2_OPEN: &str = "<h2>";
const TAG_H2_CLOSE: &str = "</h2>";
const TAG_H3_OPEN: &str = "<h3>";
const TAG_H3_CLOSE: &str = "</h3>";
const TAG_H4_OPEN: &str = "<h4>";
const TAG_H4_CLOSE: &str = "</h4>";
const TAG_H5_OPEN: &str = "<h5>";
const TAG_H5_CLOSE: &str = "</h5>";
const TAG_H6_OPEN: &str = "<h6>";
const TAG_H6_CLOSE: &str = "</h6>";
const TAG_P_OPEN: &str = "<p>";
const TAG_P_CLOSE: &str = "</p>";
const TAG_STRONG_OPEN: &str = "<strong>";
const TAG_STRONG_CLOSE: &str = "</strong>";
const TAG_EM_OPEN: &str = "<em>";
const TAG_EM_CLOSE: &str = "</em>";
const TAG_BR: &str = "<br>";
const TAG_A_OPEN: &str = "<a href=\"";
const TAG_A_MIDDLE: &str = "\">";
const TAG_A_CLOSE: &str = "</a>";
const TAG_IMG_OPEN: &str = "<img src=\"";
const TAG_IMG_MIDDLE: &str = "\" alt=\"";
const TAG_IMG_CLOSE: &str = "\">";

const MIN_HEADING_LEVEL: u8 = 1;
const MAX_HEADING_LEVEL: u8 = 6;

#[derive(Debug, Clone, PartialEq)]
pub enum HtmlError {
    InvalidHeadingLevel(u8),
    // Future errors can be added here as the package evolves
}

impl fmt::Display for HtmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HtmlError::InvalidHeadingLevel(level) => write!(
                f,
                "Invalid heading level: {}. Heading level must be between {} and {} (inclusive)",
                level, MIN_HEADING_LEVEL, MAX_HEADING_LEVEL
            ),
        }
    }
}

impl std::error::Error for HtmlError {}

/// Configuration for HTML writer
/// This is a placeholder for future blog-specific customization:
/// - CSS classes for elements
/// - Custom wrapper tags (article, section)
/// - ID generation for headings (anchor links, TOC)
/// - Custom attributes (data-*, aria-*)
/// - Metadata handling
#[derive(Debug, Clone)]
struct HtmlConfig {
    // Reserved for future blog customization
}

impl Default for HtmlConfig {
    fn default() -> Self {
        Self {}
    }
}

/// HTML writer for converting parsed markdown documents to HTML
///
/// This is a "dirty" package that will contain blog-specific logic
/// and customization in the future. Currently provides basic HTML conversion.
///
/// # Layer 1: Basic HTML Conversion (Current)
/// - Converts Document tree to semantic HTML tags
/// - Handles hierarchical sections recursively
/// - Supports inline formatting (bold, italic, links, line breaks)
/// - Escapes HTML entities for security
/// - Outputs minified HTML (no unnecessary whitespace)
///
/// # Layer 2: Blog Customization (Future)
/// - Custom CSS classes
/// - Wrapper elements
/// - ID generation for headings
/// - Custom attributes
pub struct HtmlWriter {
    _config: HtmlConfig,
}

impl HtmlWriter {
    pub fn new() -> Self {
        Self {
            _config: HtmlConfig::default(),
        }
    }

    /// Converts a parsed markdown document to minified HTML
    ///
    /// # Arguments
    /// * `document` - The parsed markdown document
    ///
    /// # Returns
    /// * `Ok(String)` - Minified HTML output
    /// * `Err(HtmlError)` - If conversion fails (e.g., invalid heading level)
    ///
    /// # Example
    /// ```no_run
    /// use wtf::parser::MarkdownParser;
    /// use wtf::html_writer::HtmlWriter;
    ///
    /// let doc = MarkdownParser::parse("# Hello World").unwrap();
    ///
    /// let writer = HtmlWriter::new();
    /// let html = writer.write_html(&doc).unwrap();
    /// assert_eq!(html, "<h1>Hello World</h1>");
    /// ```
    pub fn write_html(&self, document: &Document) -> Result<String, HtmlError> {
        let mut output = String::new();

        // Render preamble (content before first heading)
        for block in document.content() {
            output.push_str(&self.render_block(block)?);
        }

        // Render all sections
        for section in document.sections() {
            output.push_str(&self.render_section(section)?);
        }

        Ok(output)
    }

    /// Render a section and its subsections recursively
    fn render_section(&self, section: &Section) -> Result<String, HtmlError> {
        let mut output = String::new();

        // Render heading
        output.push_str(&self.render_heading(section.level(), section.title())?);

        // Render content blocks
        for block in section.content() {
            output.push_str(&self.render_block(block)?);
        }

        // Render subsections recursively
        for subsection in section.subsections() {
            output.push_str(&self.render_section(subsection)?);
        }

        Ok(output)
    }

    /// Render a block-level node (paragraph or image)
    fn render_block(&self, block: &BlockNode) -> Result<String, HtmlError> {
        match block {
            BlockNode::Paragraph(inlines) => {
                let content = self.render_inline_nodes(inlines)?;
                Ok(format!("{}{}{}", TAG_P_OPEN, content, TAG_P_CLOSE))
            }
            BlockNode::Image { alt_text, url } => {
                let escaped_alt = escape_html(alt_text);
                let escaped_url = escape_html(url);
                Ok(format!(
                    "{}{}{}{}{}",
                    TAG_IMG_OPEN, escaped_url, TAG_IMG_MIDDLE, escaped_alt, TAG_IMG_CLOSE
                ))
            }
        }
    }

    /// Render a slice of inline nodes
    fn render_inline_nodes(&self, nodes: &[InlineNode]) -> Result<String, HtmlError> {
        let mut output = String::new();
        for node in nodes {
            output.push_str(&self.render_inline(node)?);
        }
        Ok(output)
    }

    /// Render a single inline node
    fn render_inline(&self, node: &InlineNode) -> Result<String, HtmlError> {
        match node {
            InlineNode::Text(text) => Ok(escape_html(text)),
            InlineNode::LineBreak => Ok(TAG_BR.to_string()),
            InlineNode::Bold(children) => {
                let content = self.render_inline_nodes(children)?;
                Ok(format!(
                    "{}{}{}",
                    TAG_STRONG_OPEN, content, TAG_STRONG_CLOSE
                ))
            }
            InlineNode::Italic(children) => {
                let content = self.render_inline_nodes(children)?;
                Ok(format!("{}{}{}", TAG_EM_OPEN, content, TAG_EM_CLOSE))
            }
            InlineNode::Link { text, url } => {
                let content = self.render_inline_nodes(text)?;
                let escaped_url = escape_html(url);
                Ok(format!(
                    "{}{}{}{}{}",
                    TAG_A_OPEN, escaped_url, TAG_A_MIDDLE, content, TAG_A_CLOSE
                ))
            }
        }
    }

    /// Render a heading with inline-formatted title
    fn render_heading(&self, level: u8, title: &[InlineNode]) -> Result<String, HtmlError> {
        if !(MIN_HEADING_LEVEL..=MAX_HEADING_LEVEL).contains(&level) {
            return Err(HtmlError::InvalidHeadingLevel(level));
        }

        let content = self.render_inline_nodes(title)?;
        let (open_tag, close_tag) = match level {
            1 => (TAG_H1_OPEN, TAG_H1_CLOSE),
            2 => (TAG_H2_OPEN, TAG_H2_CLOSE),
            3 => (TAG_H3_OPEN, TAG_H3_CLOSE),
            4 => (TAG_H4_OPEN, TAG_H4_CLOSE),
            5 => (TAG_H5_OPEN, TAG_H5_CLOSE),
            6 => (TAG_H6_OPEN, TAG_H6_CLOSE),
            _ => unreachable!("Heading level already validated"),
        };

        Ok(format!("{}{}{}", open_tag, content, close_tag))
    }
}

impl Default for HtmlWriter {
    fn default() -> Self {
        Self::new()
    }
}

/// Escapes HTML entities in content for security
///
/// Escapes the following characters:
/// - `&` → `&amp;`
/// - `<` → `&lt;`
/// - `>` → `&gt;`
/// - `"` → `&quot;`
/// - `'` → `&#39;`
///
/// Note: & is escaped first to avoid double-escaping
fn escape_html(content: &str) -> String {
    content
        .replace('&', ESCAPE_AMP)
        .replace('<', ESCAPE_LT)
        .replace('>', ESCAPE_GT)
        .replace('"', ESCAPE_QUOT)
        .replace('\'', ESCAPE_APOS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::MarkdownParser;

    #[test]
    fn test_simple_paragraph() {
        let doc = MarkdownParser::parse("Hello world").unwrap();
        let writer = HtmlWriter::new();
        let html = writer.write_html(&doc).unwrap();
        assert_eq!(html, "<p>Hello world</p>");
    }

    #[test]
    fn test_bold_text() {
        let doc = MarkdownParser::parse("This is **bold** text").unwrap();
        let writer = HtmlWriter::new();
        let html = writer.write_html(&doc).unwrap();
        assert_eq!(html, "<p>This is <strong>bold</strong> text</p>");
    }

    #[test]
    fn test_italic_text() {
        let doc = MarkdownParser::parse("This is *italic* text").unwrap();
        let writer = HtmlWriter::new();
        let html = writer.write_html(&doc).unwrap();
        assert_eq!(html, "<p>This is <em>italic</em> text</p>");
    }

    #[test]
    fn test_line_break() {
        let doc = MarkdownParser::parse("Line one\nLine two").unwrap();
        let writer = HtmlWriter::new();
        let html = writer.write_html(&doc).unwrap();
        assert_eq!(html, "<p>Line one<br>Line two</p>");
    }

    #[test]
    fn test_multiple_paragraphs() {
        let doc = MarkdownParser::parse("Para 1\n\nPara 2").unwrap();
        let writer = HtmlWriter::new();
        let html = writer.write_html(&doc).unwrap();
        assert_eq!(html, "<p>Para 1</p><p>Para 2</p>");
    }

    #[test]
    fn test_simple_heading() {
        let doc = MarkdownParser::parse("# Heading").unwrap();
        let writer = HtmlWriter::new();
        let html = writer.write_html(&doc).unwrap();
        assert_eq!(html, "<h1>Heading</h1>");
    }

    #[test]
    fn test_heading_with_content() {
        let doc = MarkdownParser::parse("# Heading\nContent here").unwrap();
        let writer = HtmlWriter::new();
        let html = writer.write_html(&doc).unwrap();
        assert_eq!(html, "<h1>Heading</h1><p>Content here</p>");
    }

    #[test]
    fn test_nested_sections() {
        let doc = MarkdownParser::parse("# H1\n## H2\n### H3").unwrap();
        let writer = HtmlWriter::new();
        let html = writer.write_html(&doc).unwrap();
        assert_eq!(html, "<h1>H1</h1><h2>H2</h2><h3>H3</h3>");
    }

    #[test]
    fn test_link() {
        let doc = MarkdownParser::parse("[link text](https://example.com)").unwrap();
        let writer = HtmlWriter::new();
        let html = writer.write_html(&doc).unwrap();
        assert_eq!(html, "<p><a href=\"https://example.com\">link text</a></p>");
    }

    #[test]
    fn test_image() {
        let doc = MarkdownParser::parse("![alt text](image.jpg)").unwrap();
        let writer = HtmlWriter::new();
        let html = writer.write_html(&doc).unwrap();
        assert_eq!(html, "<img src=\"image.jpg\" alt=\"alt text\">");
    }

    #[test]
    fn test_html_entity_escaping() {
        let doc = MarkdownParser::parse("<script>alert(\"XSS\")</script>").unwrap();
        let writer = HtmlWriter::new();
        let html = writer.write_html(&doc).unwrap();
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("&quot;"));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn test_bold_with_italic_inside() {
        let doc = MarkdownParser::parse("**bold with *italic* inside**").unwrap();
        let writer = HtmlWriter::new();
        let html = writer.write_html(&doc).unwrap();
        assert_eq!(
            html,
            "<p><strong>bold with <em>italic</em> inside</strong></p>"
        );
    }

    #[test]
    fn test_link_with_formatting() {
        let doc = MarkdownParser::parse("[**bold** link](url)").unwrap();
        let writer = HtmlWriter::new();
        let html = writer.write_html(&doc).unwrap();
        assert_eq!(
            html,
            "<p><a href=\"url\"><strong>bold</strong> link</a></p>"
        );
    }

    #[test]
    fn test_preamble_and_sections() {
        let doc = MarkdownParser::parse("Intro text\n\n# Heading\nContent").unwrap();
        let writer = HtmlWriter::new();
        let html = writer.write_html(&doc).unwrap();
        assert_eq!(html, "<p>Intro text</p><h1>Heading</h1><p>Content</p>");
    }

    #[test]
    fn test_minified_output() {
        let doc = MarkdownParser::parse("# H1\n## H2").unwrap();
        let writer = HtmlWriter::new();
        let html = writer.write_html(&doc).unwrap();
        // Verify no newlines between tags
        assert!(!html.contains('\n'));
        assert_eq!(html, "<h1>H1</h1><h2>H2</h2>");
    }
}
