use wtf::html_writer::HtmlWriter;
use wtf::parser::MarkdownParser;

#[test]
fn test_simple_text() {
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
fn test_nested_headings() {
    let doc =
        MarkdownParser::parse("# H1\nContent 1\n## H2\nContent 2\n### H3\nContent 3").unwrap();
    let writer = HtmlWriter::new();
    let html = writer.write_html(&doc).unwrap();
    assert_eq!(
        html,
        "<h1>H1</h1><p>Content 1</p><h2>H2</h2><p>Content 2</p><h3>H3</h3><p>Content 3</p>"
    );
}

#[test]
fn test_skipped_heading_levels() {
    let doc = MarkdownParser::parse("# H1\n#### H4\nContent").unwrap();
    let writer = HtmlWriter::new();
    let html = writer.write_html(&doc).unwrap();
    assert_eq!(html, "<h1>H1</h1><h4>H4</h4><p>Content</p>");
}

#[test]
fn test_preamble() {
    let doc =
        MarkdownParser::parse("Intro text\n\nMore intro\n\n# First Heading\nContent").unwrap();
    let writer = HtmlWriter::new();
    let html = writer.write_html(&doc).unwrap();
    assert_eq!(
        html,
        "<p>Intro text</p><p>More intro</p><h1>First Heading</h1><p>Content</p>"
    );
}

#[test]
fn test_link() {
    let doc = MarkdownParser::parse("[link text](https://example.com)").unwrap();
    let writer = HtmlWriter::new();
    let html = writer.write_html(&doc).unwrap();
    assert_eq!(html, "<p><a href=\"https://example.com\">link text</a></p>");
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
fn test_image() {
    let doc = MarkdownParser::parse("![alt text](image.jpg)").unwrap();
    let writer = HtmlWriter::new();
    let html = writer.write_html(&doc).unwrap();
    assert_eq!(html, "<img src=\"image.jpg\" alt=\"alt text\">");
}

#[test]
fn test_image_with_escaped_bracket() {
    let doc = MarkdownParser::parse("![alt \\]text](image.jpg)").unwrap();
    let writer = HtmlWriter::new();
    let html = writer.write_html(&doc).unwrap();
    assert_eq!(html, "<img src=\"image.jpg\" alt=\"alt ]text\">");
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
fn test_italic_with_bold_inside() {
    let doc = MarkdownParser::parse("*italic with **bold** inside*").unwrap();
    let writer = HtmlWriter::new();
    let html = writer.write_html(&doc).unwrap();
    assert_eq!(
        html,
        "<p><em>italic with <strong>bold</strong> inside</em></p>"
    );
}

#[test]
fn test_html_escaping() {
    let doc = MarkdownParser::parse("<script>alert(\"XSS\")</script>").unwrap();
    let writer = HtmlWriter::new();
    let html = writer.write_html(&doc).unwrap();
    assert!(html.contains("&lt;script&gt;"));
    assert!(html.contains("&quot;"));
    assert!(!html.contains("<script>"));
}

#[test]
fn test_url_whitespace_trimming() {
    let doc = MarkdownParser::parse("[link](  https://example.com  )").unwrap();
    let writer = HtmlWriter::new();
    let html = writer.write_html(&doc).unwrap();
    assert_eq!(html, "<p><a href=\"https://example.com\">link</a></p>");
}

#[test]
fn test_heading_with_formatting() {
    let doc = MarkdownParser::parse("# Heading with **bold** text").unwrap();
    let writer = HtmlWriter::new();
    let html = writer.write_html(&doc).unwrap();
    assert_eq!(html, "<h1>Heading with <strong>bold</strong> text</h1>");
}

#[test]
fn test_multiple_same_level_headings() {
    let doc = MarkdownParser::parse("# H1 First\n\n# H1 Second\n\n# H1 Third").unwrap();
    let writer = HtmlWriter::new();
    let html = writer.write_html(&doc).unwrap();
    assert_eq!(html, "<h1>H1 First</h1><h1>H1 Second</h1><h1>H1 Third</h1>");
}

#[test]
fn test_complex_hierarchy() {
    let md = r#"# Chapter 1
Intro to chapter 1.

## Section 1.1
Content for 1.1

### Subsection 1.1.1
Deep content

## Section 1.2
Content for 1.2

# Chapter 2
New chapter"#;

    let doc = MarkdownParser::parse(md).unwrap();
    let writer = HtmlWriter::new();
    let html = writer.write_html(&doc).unwrap();

    assert!(html.contains("<h1>Chapter 1</h1>"));
    assert!(html.contains("<h2>Section 1.1</h2>"));
    assert!(html.contains("<h3>Subsection 1.1.1</h3>"));
    assert!(html.contains("<h2>Section 1.2</h2>"));
    assert!(html.contains("<h1>Chapter 2</h1>"));
}

#[test]
fn test_empty_document() {
    let doc = MarkdownParser::parse("").unwrap();
    let writer = HtmlWriter::new();
    let html = writer.write_html(&doc).unwrap();
    assert_eq!(html, "");
}

#[test]
fn test_only_whitespace() {
    let doc = MarkdownParser::parse("\n\n\n").unwrap();
    let writer = HtmlWriter::new();
    let html = writer.write_html(&doc).unwrap();
    assert_eq!(html, "");
}

#[test]
fn test_all_heading_levels() {
    let content = "# H1\n## H2\n### H3\n#### H4\n##### H5\n###### H6";
    let doc = MarkdownParser::parse(content).unwrap();
    let writer = HtmlWriter::new();
    let html = writer.write_html(&doc).unwrap();

    // Verify all heading levels are present
    assert!(html.contains("<h1>H1</h1>"));
    assert!(html.contains("<h2>H2</h2>"));
    assert!(html.contains("<h3>H3</h3>"));
    assert!(html.contains("<h4>H4</h4>"));
    assert!(html.contains("<h5>H5</h5>"));
    assert!(html.contains("<h6>H6</h6>"));
}

#[test]
fn test_unclosed_bold() {
    let result = MarkdownParser::parse("**unclosed bold");
    assert!(result.is_err());
}

#[test]
fn test_unclosed_italic() {
    let result = MarkdownParser::parse("*unclosed italic");
    assert!(result.is_err());
}

#[test]
fn test_malformed_link_no_closing_bracket() {
    let result = MarkdownParser::parse("[link text(url)");
    assert!(result.is_err());
}

#[test]
fn test_malformed_link_no_parens() {
    // Without (), this is a malformed link
    let result = MarkdownParser::parse("[link text]");
    assert!(result.is_err());
}
