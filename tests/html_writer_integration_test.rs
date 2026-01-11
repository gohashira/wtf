use wtf::html_writer::HtmlWriter;
use wtf::parser::MarkdownParser;

#[test]
fn test_heading_with_inline_formatting() {
    let doc = MarkdownParser::parse("# Heading with **bold** and *italic*").unwrap();
    let writer = HtmlWriter::new();
    let html = writer.write_html(&doc).unwrap();
    assert_eq!(
        html,
        "<h1>Heading with <strong>bold</strong> and <em>italic</em></h1>"
    );
}

#[test]
fn test_complex_nested_inline() {
    let doc =
        MarkdownParser::parse("**Bold with *italic* inside** and *italic with **bold** inside*")
            .unwrap();
    let writer = HtmlWriter::new();
    let html = writer.write_html(&doc).unwrap();
    assert_eq!(
        html,
        "<p><strong>Bold with <em>italic</em> inside</strong> and <em>italic with <strong>bold</strong> inside</em></p>"
    );
}

#[test]
fn test_line_breaks_in_paragraph() {
    let doc = MarkdownParser::parse("Line one\nLine two\nLine three").unwrap();
    let writer = HtmlWriter::new();
    let html = writer.write_html(&doc).unwrap();
    assert_eq!(html, "<p>Line one<br>Line two<br>Line three</p>");
}

#[test]
fn test_image_as_block() {
    let doc = MarkdownParser::parse("Before\n\n![alt](url)\n\nAfter").unwrap();
    let writer = HtmlWriter::new();
    let html = writer.write_html(&doc).unwrap();
    assert_eq!(
        html,
        "<p>Before</p><img src=\"url\" alt=\"alt\"><p>After</p>"
    );
}

#[test]
fn test_image_with_escaped_brackets() {
    let doc = MarkdownParser::parse("![alt \\[2024\\]](url)").unwrap();
    let writer = HtmlWriter::new();
    let html = writer.write_html(&doc).unwrap();
    assert_eq!(html, "<img src=\"url\" alt=\"alt [2024]\">");
}

#[test]
fn test_link_with_nested_formatting() {
    let doc = MarkdownParser::parse("[Link with **bold** and *italic*](url)").unwrap();
    let writer = HtmlWriter::new();
    let html = writer.write_html(&doc).unwrap();
    assert_eq!(
        html,
        "<p><a href=\"url\">Link with <strong>bold</strong> and <em>italic</em></a></p>"
    );
}

#[test]
fn test_preamble_with_sections() {
    let doc = MarkdownParser::parse("Preamble 1\n\nPreamble 2\n\n# Heading\n\nContent").unwrap();
    let writer = HtmlWriter::new();
    let html = writer.write_html(&doc).unwrap();
    assert_eq!(
        html,
        "<p>Preamble 1</p><p>Preamble 2</p><h1>Heading</h1><p>Content</p>"
    );
}

#[test]
fn test_nested_sections_hierarchy() {
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
fn test_multiple_same_level_sections() {
    let doc = MarkdownParser::parse("# H1 First\n\n# H1 Second\n\n## H2\n\n# H1 Third").unwrap();
    let writer = HtmlWriter::new();
    let html = writer.write_html(&doc).unwrap();
    assert!(html.contains("<h1>H1 First</h1>"));
    assert!(html.contains("<h1>H1 Second</h1>"));
    assert!(html.contains("<h2>H2</h2>"));
    assert!(html.contains("<h1>H1 Third</h1>"));
}

#[test]
fn test_all_heading_levels() {
    let doc = MarkdownParser::parse("# H1\n## H2\n### H3\n#### H4\n##### H5\n###### H6").unwrap();
    let writer = HtmlWriter::new();
    let html = writer.write_html(&doc).unwrap();
    assert_eq!(
        html,
        "<h1>H1</h1><h2>H2</h2><h3>H3</h3><h4>H4</h4><h5>H5</h5><h6>H6</h6>"
    );
}

#[test]
fn test_html_entities_in_all_contexts() {
    let doc = MarkdownParser::parse("Text with <>&\"'\n\n# Heading with <>&\"'\n\n**Bold <>&\"'**\n\n*Italic <>&\"'*\n\n[Link <>&\"'](url)").unwrap();
    let writer = HtmlWriter::new();
    let html = writer.write_html(&doc).unwrap();

    // Check that all special characters are escaped
    assert!(html.contains("&lt;"));
    assert!(html.contains("&gt;"));
    assert!(html.contains("&amp;"));
    assert!(html.contains("&quot;"));
    assert!(html.contains("&#39;"));

    // Make sure no unescaped characters exist
    assert!(!html.contains("<>&\"'"));
}

#[test]
fn test_url_trimming() {
    let doc = MarkdownParser::parse("[link](  https://example.com  )").unwrap();
    let writer = HtmlWriter::new();
    let html = writer.write_html(&doc).unwrap();
    assert_eq!(html, "<p><a href=\"https://example.com\">link</a></p>");
}

#[test]
fn test_empty_paragraphs_preserved() {
    let doc = MarkdownParser::parse("Para 1\n\n\n\nPara 2").unwrap();
    let writer = HtmlWriter::new();
    let html = writer.write_html(&doc).unwrap();
    // Empty paragraphs between content should result in just 2 paragraphs
    assert_eq!(html, "<p>Para 1</p><p>Para 2</p>");
}

#[test]
fn test_minified_output_no_whitespace() {
    let doc = MarkdownParser::parse("# H1\n\n## H2\n\nText").unwrap();
    let writer = HtmlWriter::new();
    let html = writer.write_html(&doc).unwrap();

    // Verify no newlines in output
    assert!(!html.contains('\n'));
    assert!(!html.contains("  ")); // No double spaces
}

#[test]
fn test_complex_real_world_document() {
    let markdown = r#"Welcome to my blog!

This is a preamble paragraph with **bold** text.

# Chapter 1: Introduction

This chapter introduces the concept.
It has multiple lines
in the same paragraph.

## Section 1.1

Content with **bold *and italic* together**.

![Featured image](feature.jpg)

More content after image.

### Subsection 1.1.1

Deep content with [a **formatted** link](https://example.com).

## Section 1.2

Another section at H2 level.

# Chapter 2: Conclusion

Final thoughts with *italic containing **bold** inside*.

The end.
"#;

    let doc = MarkdownParser::parse(markdown).unwrap();
    let writer = HtmlWriter::new();
    let html = writer.write_html(&doc).unwrap();

    // Verify key elements are present
    assert!(html.starts_with("<p>Welcome to my blog!</p>"));
    assert!(html.contains("<h1>Chapter 1: Introduction</h1>"));
    assert!(html.contains("<h2>Section 1.1</h2>"));
    assert!(html.contains("<h3>Subsection 1.1.1</h3>"));
    assert!(html.contains("<h2>Section 1.2</h2>"));
    assert!(html.contains("<h1>Chapter 2: Conclusion</h1>"));
    assert!(html.contains("<br>")); // Line breaks preserved
    assert!(html.contains("<img src=\"feature.jpg\" alt=\"Featured image\">"));
    assert!(html.contains("<strong>bold <em>and italic</em> together</strong>"));
    assert!(html.contains("<em>italic containing <strong>bold</strong> inside</em>"));
    assert!(html.contains("<a href=\"https://example.com\">a <strong>formatted</strong> link</a>"));

    // Verify no newlines (minified)
    assert!(!html.contains('\n'));
}
