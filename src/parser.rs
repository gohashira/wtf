use std::fmt;

// Constants
const HEADING_CHAR: char = '#';
const BOLD_DELIM: &str = "**";
const ITALIC_DELIM: char = '*';
const NEWLINE_CHAR: char = '\n';
const LINK_OPEN: char = '[';
const LINK_CLOSE: char = ']';
const URL_OPEN: char = '(';
const URL_CLOSE: char = ')';
const IMAGE_PREFIX: char = '!';
const ESCAPE_CHAR: char = '\\';

const MIN_HEADING_LEVEL: u8 = 1;
const MAX_HEADING_LEVEL: u8 = 6;

// ============================================================================
// ERROR HANDLING
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedEndOfInput { context: String },
    UnclosedDelimiter { delimiter: String, position: usize },
    InvalidHeadingLevel { level: u8 },
    MalformedLink { position: usize },
    MalformedImage { position: usize },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedEndOfInput { context } => {
                write!(f, "Unexpected end of input while parsing {}", context)
            }
            ParseError::UnclosedDelimiter {
                delimiter,
                position,
            } => {
                write!(
                    f,
                    "Unclosed delimiter '{}' at position {}",
                    delimiter, position
                )
            }
            ParseError::InvalidHeadingLevel { level } => {
                write!(
                    f,
                    "Invalid heading level: {}. Must be between {} and {}",
                    level, MIN_HEADING_LEVEL, MAX_HEADING_LEVEL
                )
            }
            ParseError::MalformedLink { position } => {
                write!(f, "Malformed link syntax at position {}", position)
            }
            ParseError::MalformedImage { position } => {
                write!(f, "Malformed image syntax at position {}", position)
            }
        }
    }
}

impl std::error::Error for ParseError {}

// ============================================================================
// INLINE NODES (within paragraphs and headings)
// ============================================================================

#[derive(Debug, PartialEq, Clone)]
pub enum InlineNode {
    /// Plain text
    Text(String),

    /// Line break (single newline within paragraph)
    LineBreak,

    /// Bold text (can contain nested inline nodes)
    Bold(Vec<InlineNode>),

    /// Italic text (can contain nested inline nodes)
    Italic(Vec<InlineNode>),

    /// Link [text](url) - text can contain inline formatting
    Link { text: Vec<InlineNode>, url: String },
}

// ============================================================================
// BLOCK NODES (paragraphs, images, etc.)
// ============================================================================

#[derive(Debug, PartialEq, Clone)]
pub enum BlockNode {
    /// Paragraph containing inline elements
    Paragraph(Vec<InlineNode>),

    /// Image block: ![alt](url)
    Image { alt_text: String, url: String },
}

// ============================================================================
// SECTION (heading with content and subsections)
// ============================================================================

#[derive(Debug, PartialEq, Clone)]
pub struct Section {
    level: u8,
    title: Vec<InlineNode>,
    content: Vec<BlockNode>,
    subsections: Vec<Section>,
}

impl Section {
    pub fn new(level: u8, title: Vec<InlineNode>) -> Self {
        Self {
            level,
            title,
            content: Vec::new(),
            subsections: Vec::new(),
        }
    }

    pub fn level(&self) -> u8 {
        self.level
    }

    pub fn title(&self) -> &[InlineNode] {
        &self.title
    }

    pub fn content(&self) -> &[BlockNode] {
        &self.content
    }

    pub fn subsections(&self) -> &[Section] {
        &self.subsections
    }
}

// ============================================================================
// DOCUMENT (root of the parse tree)
// ============================================================================

#[derive(Debug, PartialEq, Clone)]
pub struct Document {
    /// Content before the first heading (preamble)
    content: Vec<BlockNode>,

    /// Top-level sections (H1, or highest level heading)
    sections: Vec<Section>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            content: Vec::new(),
            sections: Vec::new(),
        }
    }

    pub fn content(&self) -> &[BlockNode] {
        &self.content
    }

    pub fn sections(&self) -> &[Section] {
        &self.sections
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// PARSER
// ============================================================================

pub struct MarkdownParser {
    chars: Vec<char>,
    pos: usize,
}

impl MarkdownParser {
    pub fn new() -> Self {
        Self {
            chars: Vec::new(),
            pos: 0,
        }
    }

    /// Main parsing entry point
    pub fn parse(text: &str) -> Result<Document, ParseError> {
        let mut parser = Self::new();
        parser.chars = text.chars().collect();
        parser.pos = 0;
        parser.parse_document()
    }

    // ========================================================================
    // DOCUMENT-LEVEL PARSING
    // ========================================================================

    fn parse_document(&mut self) -> Result<Document, ParseError> {
        let mut document = Document::new();

        // Parse blocks until we hit a heading or EOF
        while !self.is_eof() {
            self.skip_empty_lines();

            if self.is_eof() {
                break;
            }

            // Check if this is a heading
            if self.is_heading() {
                // We hit a heading - parse all sections
                let sections = self.parse_all_sections()?;
                document.sections = sections;
                break;
            }

            // Parse a block for the preamble
            let block = self.parse_block()?;
            document.content.push(block);
        }

        Ok(document)
    }

    // ========================================================================
    // SECTION-LEVEL PARSING (hierarchical heading structure)
    // ========================================================================

    fn parse_all_sections(&mut self) -> Result<Vec<Section>, ParseError> {
        let mut sections = Vec::new();

        while !self.is_eof() {
            self.skip_empty_lines();

            if self.is_eof() {
                break;
            }

            if self.is_heading() {
                let section = self.parse_section_tree(None)?;
                sections.push(section);
            } else {
                // This shouldn't happen but handle gracefully
                break;
            }
        }

        Ok(sections)
    }

    /// Parse a section tree recursively
    /// parent_level: None for top-level, Some(level) for subsections
    fn parse_section_tree(&mut self, _parent_level: Option<u8>) -> Result<Section, ParseError> {
        let (level, title) = self.parse_heading_line()?;
        let mut section = Section::new(level, title);

        // Parse content until next heading or EOF
        loop {
            self.skip_empty_lines();

            if self.is_eof() {
                break;
            }

            if self.is_heading() {
                let next_level = self.peek_heading_level();

                // If next heading is deeper, it's a subsection
                if next_level > level {
                    let subsection = self.parse_section_tree(Some(level))?;
                    section.subsections.push(subsection);
                } else {
                    // Same or higher level - return to parent
                    break;
                }
            } else {
                // Parse content block
                let block = self.parse_block()?;
                section.content.push(block);
            }
        }

        Ok(section)
    }

    // ========================================================================
    // BLOCK-LEVEL PARSING (paragraphs, images)
    // ========================================================================

    fn parse_block(&mut self) -> Result<BlockNode, ParseError> {
        // Check for image block: ![alt](url)
        if self.peek() == Some(IMAGE_PREFIX) && self.peek_at(1) == Some(LINK_OPEN) {
            return self.parse_image_block();
        }

        // Otherwise, parse as paragraph
        self.parse_paragraph()
    }

    fn parse_paragraph(&mut self) -> Result<BlockNode, ParseError> {
        let mut inline_nodes = Vec::new();
        let mut first_line = true;

        loop {
            // Parse a line of inline content
            let line_content = self.parse_inline_content()?;

            if !line_content.is_empty() {
                // Add line break between lines (but not before first line)
                if !first_line {
                    inline_nodes.push(InlineNode::LineBreak);
                }
                inline_nodes.extend(line_content);
                first_line = false;
            }

            // Check what's next
            if self.peek() == Some(NEWLINE_CHAR) {
                self.advance(); // consume newline

                // Check for double newline (paragraph boundary)
                if self.peek() == Some(NEWLINE_CHAR) || self.is_eof() || self.is_heading() {
                    break;
                }
                // Single newline - continue with next line
            } else {
                // EOF or heading
                break;
            }
        }

        Ok(BlockNode::Paragraph(inline_nodes))
    }

    fn parse_image_block(&mut self) -> Result<BlockNode, ParseError> {
        let start_pos = self.pos;

        // Consume "!["
        self.advance(); // !
        self.advance(); // [

        // Parse alt text with escape support for brackets
        let mut alt_text = String::new();
        while let Some(ch) = self.peek() {
            if ch == ESCAPE_CHAR {
                self.advance();
                if let Some(next_ch) = self.peek() {
                    alt_text.push(next_ch);
                    self.advance();
                }
            } else if ch == LINK_CLOSE {
                self.advance();
                break;
            } else {
                alt_text.push(ch);
                self.advance();
            }
        }

        // Expect '('
        if self.peek() != Some(URL_OPEN) {
            return Err(ParseError::MalformedImage {
                position: start_pos,
            });
        }
        self.advance();

        // Parse URL
        let mut url = String::new();
        while let Some(ch) = self.peek() {
            if ch == URL_CLOSE {
                self.advance();
                break;
            }
            if ch == NEWLINE_CHAR {
                return Err(ParseError::MalformedImage {
                    position: start_pos,
                });
            }
            url.push(ch);
            self.advance();
        }

        // Trim whitespace from URL
        let url = url.trim().to_string();

        // Consume trailing newline if present
        if self.peek() == Some(NEWLINE_CHAR) {
            self.advance();
        }

        Ok(BlockNode::Image { alt_text, url })
    }

    // ========================================================================
    // INLINE-LEVEL PARSING (text, bold, italic, links, line breaks)
    // ========================================================================

    /// Parse inline content until newline or EOF
    fn parse_inline_content(&mut self) -> Result<Vec<InlineNode>, ParseError> {
        let mut nodes = Vec::new();

        while !self.is_eof() && self.peek() != Some(NEWLINE_CHAR) {
            // Check for inline elements
            if self.starts_with(BOLD_DELIM) {
                nodes.push(self.parse_bold()?);
            } else if self.peek() == Some(ITALIC_DELIM) && !self.starts_with(BOLD_DELIM) {
                nodes.push(self.parse_italic()?);
            } else if self.peek() == Some(LINK_OPEN) {
                nodes.push(self.parse_link()?);
            } else {
                // Parse plain text until next delimiter
                let text = self.parse_text_inline()?;
                if !text.is_empty() {
                    nodes.push(InlineNode::Text(text));
                }
            }
        }

        Ok(nodes)
    }

    fn parse_text_inline(&mut self) -> Result<String, ParseError> {
        let mut text = String::new();

        while let Some(ch) = self.peek() {
            // Stop at delimiters or newline
            if ch == NEWLINE_CHAR || ch == ITALIC_DELIM || ch == LINK_OPEN {
                break;
            }

            text.push(ch);
            self.advance();
        }

        Ok(text)
    }

    fn parse_bold(&mut self) -> Result<InlineNode, ParseError> {
        let start_pos = self.pos;

        // Consume opening "**"
        self.advance();
        self.advance();

        let mut children = Vec::new();

        // Parse until closing "**"
        while !self.is_eof() {
            if self.starts_with(BOLD_DELIM) {
                // Found closing delimiter
                self.advance();
                self.advance();
                return Ok(InlineNode::Bold(children));
            }

            // Stop at newline (unclosed bold)
            if self.peek() == Some(NEWLINE_CHAR) {
                return Err(ParseError::UnclosedDelimiter {
                    delimiter: BOLD_DELIM.to_string(),
                    position: start_pos,
                });
            }

            // Parse inline content (italic, link, text - but not nested bold)
            if self.peek() == Some(ITALIC_DELIM) && !self.starts_with(BOLD_DELIM) {
                children.push(self.parse_italic()?);
            } else if self.peek() == Some(LINK_OPEN) {
                children.push(self.parse_link()?);
            } else {
                let text = self.parse_text_until(&[BOLD_DELIM, "*", "[", "\n"])?;
                if !text.is_empty() {
                    children.push(InlineNode::Text(text));
                }
            }
        }

        Err(ParseError::UnclosedDelimiter {
            delimiter: BOLD_DELIM.to_string(),
            position: start_pos,
        })
    }

    fn parse_italic(&mut self) -> Result<InlineNode, ParseError> {
        let start_pos = self.pos;

        // Consume opening "*"
        self.advance();

        let mut children = Vec::new();

        // Parse until closing "*"
        while !self.is_eof() {
            if self.peek() == Some(ITALIC_DELIM) && !self.starts_with(BOLD_DELIM) {
                // Found closing delimiter (single *)
                self.advance();
                return Ok(InlineNode::Italic(children));
            }

            // Stop at newline (unclosed italic)
            if self.peek() == Some(NEWLINE_CHAR) {
                return Err(ParseError::UnclosedDelimiter {
                    delimiter: ITALIC_DELIM.to_string(),
                    position: start_pos,
                });
            }

            // Parse inline content (bold, link, text - but not nested italic)
            if self.starts_with(BOLD_DELIM) {
                children.push(self.parse_bold()?);
            } else if self.peek() == Some(LINK_OPEN) {
                children.push(self.parse_link()?);
            } else {
                let text = self.parse_text_until(&["*", "[", "\n"])?;
                if !text.is_empty() {
                    children.push(InlineNode::Text(text));
                }
            }
        }

        Err(ParseError::UnclosedDelimiter {
            delimiter: ITALIC_DELIM.to_string(),
            position: start_pos,
        })
    }

    fn parse_link(&mut self) -> Result<InlineNode, ParseError> {
        let start_pos = self.pos;

        // Consume '['
        self.advance();

        // Parse link text (can contain inline formatting)
        let mut text = Vec::new();
        while !self.is_eof() && self.peek() != Some(LINK_CLOSE) {
            if self.peek() == Some(NEWLINE_CHAR) {
                return Err(ParseError::MalformedLink {
                    position: start_pos,
                });
            }

            if self.starts_with(BOLD_DELIM) {
                text.push(self.parse_bold()?);
            } else if self.peek() == Some(ITALIC_DELIM) && !self.starts_with(BOLD_DELIM) {
                text.push(self.parse_italic()?);
            } else {
                let txt = self.parse_text_until(&[BOLD_DELIM, "*", "]", "\n"])?;
                if !txt.is_empty() {
                    text.push(InlineNode::Text(txt));
                }
            }
        }

        // Consume ']'
        if self.peek() != Some(LINK_CLOSE) {
            return Err(ParseError::MalformedLink {
                position: start_pos,
            });
        }
        self.advance();

        // Expect '('
        if self.peek() != Some(URL_OPEN) {
            return Err(ParseError::MalformedLink {
                position: start_pos,
            });
        }
        self.advance();

        // Parse URL (plain string until ')')
        let mut url = String::new();
        while !self.is_eof() && self.peek() != Some(URL_CLOSE) {
            if self.peek() == Some(NEWLINE_CHAR) {
                return Err(ParseError::MalformedLink {
                    position: start_pos,
                });
            }
            url.push(self.peek().unwrap());
            self.advance();
        }

        // Consume ')'
        if self.peek() != Some(URL_CLOSE) {
            return Err(ParseError::MalformedLink {
                position: start_pos,
            });
        }
        self.advance();

        // Trim whitespace from URL
        let url = url.trim().to_string();

        Ok(InlineNode::Link { text, url })
    }

    /// Parse text until one of the stop strings is encountered
    fn parse_text_until(&mut self, stop_strings: &[&str]) -> Result<String, ParseError> {
        let mut text = String::new();

        while let Some(ch) = self.peek() {
            // Check if we hit any stop string
            let mut should_stop = false;
            for stop_str in stop_strings {
                if self.starts_with(stop_str) {
                    should_stop = true;
                    break;
                }
            }

            if should_stop {
                break;
            }

            text.push(ch);
            self.advance();
        }

        Ok(text)
    }

    // ========================================================================
    // HEADING PARSING
    // ========================================================================

    fn parse_heading_line(&mut self) -> Result<(u8, Vec<InlineNode>), ParseError> {
        let level = self.parse_heading_level()?;

        // Skip space after #'s
        if self.peek() == Some(' ') {
            self.advance();
        }

        // Parse heading title (inline formatted)
        let title = self.parse_inline_content()?;

        // Consume newline
        if self.peek() == Some(NEWLINE_CHAR) {
            self.advance();
        }

        Ok((level, title))
    }

    fn parse_heading_level(&mut self) -> Result<u8, ParseError> {
        let mut level = 0u8;

        while self.peek() == Some(HEADING_CHAR) && level < MAX_HEADING_LEVEL {
            level += 1;
            self.advance();
        }

        if level < MIN_HEADING_LEVEL {
            return Err(ParseError::InvalidHeadingLevel { level });
        }

        Ok(level)
    }

    fn is_heading(&self) -> bool {
        self.peek() == Some(HEADING_CHAR)
    }

    fn peek_heading_level(&self) -> u8 {
        let mut level = 0u8;
        let mut temp_pos = self.pos;

        while temp_pos < self.chars.len()
            && self.chars[temp_pos] == HEADING_CHAR
            && level < MAX_HEADING_LEVEL
        {
            level += 1;
            temp_pos += 1;
        }

        level
    }

    // ========================================================================
    // UTILITY METHODS
    // ========================================================================

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    fn starts_with(&self, s: &str) -> bool {
        let s_chars: Vec<char> = s.chars().collect();
        if self.pos + s_chars.len() > self.chars.len() {
            return false;
        }
        &self.chars[self.pos..self.pos + s_chars.len()] == &s_chars[..]
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek();
        self.pos += 1;
        ch
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.chars.len()
    }

    fn skip_empty_lines(&mut self) {
        while self.peek() == Some(NEWLINE_CHAR) {
            self.advance();
        }
    }
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_text() {
        let doc = MarkdownParser::parse("Hello world").unwrap();
        assert_eq!(doc.content().len(), 1);
        assert_eq!(doc.sections().len(), 0);

        match &doc.content()[0] {
            BlockNode::Paragraph(inlines) => {
                assert_eq!(inlines.len(), 1);
                match &inlines[0] {
                    InlineNode::Text(text) => assert_eq!(text, "Hello world"),
                    _ => panic!("Expected Text node"),
                }
            }
            _ => panic!("Expected Paragraph"),
        }
    }

    #[test]
    fn test_bold_text() {
        let doc = MarkdownParser::parse("This is **bold** text").unwrap();
        match &doc.content()[0] {
            BlockNode::Paragraph(inlines) => {
                assert_eq!(inlines.len(), 3);
                assert!(matches!(inlines[0], InlineNode::Text(_)));
                assert!(matches!(inlines[1], InlineNode::Bold(_)));
                assert!(matches!(inlines[2], InlineNode::Text(_)));
            }
            _ => panic!("Expected Paragraph"),
        }
    }

    #[test]
    fn test_italic_text() {
        let doc = MarkdownParser::parse("This is *italic* text").unwrap();
        match &doc.content()[0] {
            BlockNode::Paragraph(inlines) => {
                assert_eq!(inlines.len(), 3);
                assert!(matches!(inlines[1], InlineNode::Italic(_)));
            }
            _ => panic!("Expected Paragraph"),
        }
    }

    #[test]
    fn test_line_break() {
        let doc = MarkdownParser::parse("Line one\nLine two").unwrap();
        match &doc.content()[0] {
            BlockNode::Paragraph(inlines) => {
                assert_eq!(inlines.len(), 3);
                assert!(matches!(inlines[0], InlineNode::Text(_)));
                assert!(matches!(inlines[1], InlineNode::LineBreak));
                assert!(matches!(inlines[2], InlineNode::Text(_)));
            }
            _ => panic!("Expected Paragraph"),
        }
    }

    #[test]
    fn test_multiple_paragraphs() {
        let doc = MarkdownParser::parse("Para 1\n\nPara 2").unwrap();
        assert_eq!(doc.content().len(), 2);
    }

    #[test]
    fn test_simple_heading() {
        let doc = MarkdownParser::parse("# Heading\nContent").unwrap();
        assert_eq!(doc.sections().len(), 1);
        assert_eq!(doc.sections()[0].level(), 1);
        assert_eq!(doc.sections()[0].content().len(), 1);
    }

    #[test]
    fn test_nested_sections() {
        let doc = MarkdownParser::parse("# H1\n## H2\n### H3").unwrap();
        assert_eq!(doc.sections().len(), 1);
        assert_eq!(doc.sections()[0].subsections().len(), 1);
        assert_eq!(doc.sections()[0].subsections()[0].subsections().len(), 1);
    }

    #[test]
    fn test_link() {
        let doc = MarkdownParser::parse("[text](url)").unwrap();
        match &doc.content()[0] {
            BlockNode::Paragraph(inlines) => match &inlines[0] {
                InlineNode::Link { text, url } => {
                    assert_eq!(url, "url");
                    assert_eq!(text.len(), 1);
                }
                _ => panic!("Expected Link"),
            },
            _ => panic!("Expected Paragraph"),
        }
    }

    #[test]
    fn test_image() {
        let doc = MarkdownParser::parse("![alt text](image.jpg)").unwrap();
        match &doc.content()[0] {
            BlockNode::Image { alt_text, url } => {
                assert_eq!(alt_text, "alt text");
                assert_eq!(url, "image.jpg");
            }
            _ => panic!("Expected Image"),
        }
    }
}
