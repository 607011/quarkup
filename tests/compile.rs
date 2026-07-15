use quarkup::html::HtmlRenderer;
use quarkup::lexer::Lexer;
use quarkup::parser::Parser;
use std::collections::HashMap;

fn compile(source: &str) -> String {
    compile_with_defines(source, HashMap::new())
}

fn compile_with_defines(source: &str, defines: HashMap<String, String>) -> String {
    let lexer = Lexer::new(source);
    let parser = Parser::new(lexer, defines);
    let ast = parser.parse();
    HtmlRenderer::new(None, false).render(&ast)
}

fn defines(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[test]
fn renders_heading_levels_by_stacking_quarks() {
    let html = compile(".u One\n.uu Two\n.uuu Three\n");
    assert!(html.contains("<h1>One</h1>"));
    assert!(html.contains("<h2>Two</h2>"));
    assert!(html.contains("<h3>Three</h3>"));
}

#[test]
fn dot_followed_by_letter_without_space_stays_dormant_punctuation() {
    // A dot is only an operator when followed by a letter *and* a space.
    // "e.g." and "file.txt" must survive untouched, per the README's promise
    // that no backslash-escaping is required for ordinary punctuation.
    let html = compile("See e.g. the file.txt for details.\n");
    assert!(html.contains("<p>See e.g. the file.txt for details.</p>"));
}

#[test]
fn renders_inline_formatting() {
    let html = compile("This is .c charming.. and .s inline_code...\n");
    assert!(html.contains("<em>charming</em>"));
    assert!(html.contains("<code>inline_code</code>"));
}

#[test]
fn renders_bold_via_muon() {
    let html = compile("This is .m bold text...\n");
    assert!(html.contains("<strong>bold text</strong>"));
}

#[test]
fn bold_and_italic_nest_in_both_directions() {
    let html = compile(".m bold with .c nested italic.. inside..\n");
    assert!(html.contains("<strong>bold with <em>nested italic</em> inside</strong>"));

    let html = compile(".c italic with .m nested bold.. inside..\n");
    assert!(html.contains("<em>italic with <strong>nested bold</strong> inside</em>"));
}

#[test]
fn inline_formatting_still_wraps_when_it_opens_the_paragraph() {
    // Regression test: parse_block used to consume a leading inline-only
    // quark (Charm, Muon, ...) before falling through to inline parsing,
    // silently dropping the InlineNode::Formatted wrapper whenever such a
    // quark was the very first token of a paragraph.
    let html = compile(".m Bold from the start..\n");
    assert!(html.contains("<strong>Bold from the start</strong>"));

    let html = compile(".c Italic from the start..\n");
    assert!(html.contains("<em>Italic from the start</em>"));
}

#[test]
fn renders_superscript_and_subscript() {
    let html = compile("a.u 2.. + b.u 2.. = c.u 2..\nH.d 2..O\n");
    assert!(html.contains("<sup>2</sup>"));
    assert!(html.contains("<sub>2</sub>"));
}

#[test]
fn nests_lists_by_stacking_lepton_letters() {
    let html = compile(".n One\n.nn Nested\n.n Back\n");
    // one open <ul> for the top level, another for the nested level
    assert_eq!(html.matches("<ul>").count(), 2);
    assert_eq!(html.matches("</ul>").count(), 2);
    assert!(html.contains("<li>One</li>"));
    assert!(html.contains("<li>Nested</li>"));
}

#[test]
fn renders_ordered_lists() {
    let html = compile(".e First\n.e Second\n");
    assert!(html.contains("<ol>"));
    assert!(html.contains("<li>First</li>"));
    assert!(html.contains("<li>Second</li>"));
}

#[test]
fn block_conditional_renders_only_the_matching_branch() {
    let src = ".b target=web\n.u Web only\n..\n";
    let with_web = compile_with_defines(src, defines(&[("target", "web")]));
    assert!(with_web.contains("Web only"));

    let without_web = compile_with_defines(src, HashMap::new());
    assert!(!without_web.contains("Web only"));
}

#[test]
fn inline_conditional_and_negation() {
    let src =
        "Click .b target=web|.l https://example.com|here.. .. .b !target=web|elsewhere.. now.\n";
    let web = compile_with_defines(src, defines(&[("target", "web")]));
    assert!(web.contains("href=\"https://example.com\""));
    assert!(!web.contains("elsewhere"));

    let cli = compile_with_defines(src, HashMap::new());
    assert!(!cli.contains("href="));
    assert!(cli.contains("elsewhere"));
}

#[test]
fn lattice_supports_colspan_and_rowspan_markers() {
    let src = ".l\nh: A ; B\nx ; _\ny ; z\n..\n";
    let html = compile(src);
    assert!(html.contains("rowspan=\"2\""));
}

#[test]
fn lattice_header_and_footer_rows_are_wrapped() {
    let src = ".l\nh: A ; B\nbody1 ; body2\nf: foot1 ; foot2\n..\n";
    let html = compile(src);
    assert!(html.contains("<thead>"));
    assert!(html.contains("</thead>"));
    assert!(html.contains("<tfoot>"));
    assert!(html.contains("</tfoot>"));
}

#[test]
fn code_blocks_capture_content_verbatim_without_parsing_it() {
    let src = ".s rust\nlet x = \"a.u 2..\";\n..\n";
    let html = compile(src);
    // the quarkup-looking snippet inside the code block must not be parsed
    assert!(html.contains("a.u 2.."));
    assert!(!html.contains("<sup>"));
}

#[test]
fn escapes_text_content() {
    let html = compile("Tom & Jerry <3 \"quotes\"\n");
    assert!(html.contains("Tom &amp; Jerry &lt;3 &quot;quotes&quot;"));
}

#[test]
fn escapes_link_href_and_image_attributes_against_injection() {
    // Regression test: href/src/caption used to be interpolated into HTML
    // attributes unescaped, allowing a crafted .qu document to break out of
    // the attribute and inject arbitrary markup into the rendered document.
    let html = compile("Click .l javascript:alert(1)\"><b>|here.. now.\n");
    assert!(!html.contains("\"><b>"));
    assert!(html.contains("&quot;&gt;&lt;b&gt;"));

    let html = compile(".g x.jpg \"><script>alert(1)</script>\n");
    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn escapes_metadata_substituted_into_the_default_template() {
    // Regression test: metadata (.t) values used to be substituted into the
    // <title>/<meta>/<html lang> placeholders unescaped.
    let html = compile(".t author Mallory\" onmouseover=\"alert(1)\n");
    assert!(!html.contains("onmouseover=\"alert(1)\""));
    assert!(html.contains("Mallory&quot; onmouseover=&quot;alert(1)"));
}

#[test]
fn comments_are_stripped() {
    let html = compile("Visible line\n# a comment\n// another comment\nStill visible\n");
    assert!(html.contains("Visible line"));
    assert!(html.contains("Still visible"));
    assert!(!html.contains("comment"));
}

#[test]
fn golden_file_example_demo_matches_snapshot() {
    let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/example/demo.qu"))
        .expect("example/demo.qu should exist");
    let actual = compile_with_defines(&source, defines(&[("target", "web")]));
    let snapshot_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/snapshots/example_demo.html"
    );
    let expected = std::fs::read_to_string(snapshot_path).unwrap_or_default();

    if std::env::var("BLESS").is_ok() {
        std::fs::write(snapshot_path, &actual).expect("failed to write snapshot");
        return;
    }

    assert_eq!(
        actual, expected,
        "rendered output for example/demo.qu no longer matches tests/snapshots/example_demo.html.\n\
         If this change is intentional, regenerate the snapshot with:\n\
         BLESS=1 cargo test golden_file_example_demo_matches_snapshot"
    );
}
