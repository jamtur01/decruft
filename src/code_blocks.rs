use ego_tree::NodeId;
use markup5ever::{QualName, ns};
use scraper::{Html, Node};

use crate::dom;

/// Highlighter class patterns that capture a language name.
/// Each pattern is tried against individual CSS class names.
const HIGHLIGHTER_PREFIXES: &[&str] = &[
    "language-",
    "lang-",
    "syntax-",
    "highlight-",
    "code-",
    "cm-s-", // CodeMirror theme prefix (skip, not a language)
];

/// Suffixes that indicate a language class.
const HIGHLIGHTER_SUFFIXES: &[&str] = &["-code", "-snippet"];

/// Class patterns to strip as code-block line numbers.
const LINE_NUMBER_CLASSES: &[&str] = &[
    "linenumber",
    "line-number",
    "line-numbers",
    "lineno",
    "linenos",
    "rouge-gutter",
    "hljs-ln-numbers",
    "hljs-ln-line",
    "lnt",
    "react-syntax-highlighter-line-number",
    "code-line-number",
    "gutter",
];

/// Classes on elements that are copy/run buttons to remove.
const BUTTON_CLASSES: &[&str] = &[
    "copy-button",
    "copy-code-button",
    "copybutton",
    "code-copy",
    "btn-copy",
    "copy",
];

/// Data attributes that mark copy buttons.
const BUTTON_DATA_ATTRS: &[&str] = &["data-copy", "data-clipboard-text"];

/// Known programming language names and aliases.
const LANGUAGES: &[&str] = &[
    "actionscript",
    "ada",
    "agda",
    "apache",
    "applescript",
    "arduino",
    "asm",
    "assembly",
    "astro",
    "awk",
    "bash",
    "basic",
    "batch",
    "brainfuck",
    "bsl",
    "c",
    "ceylon",
    "clojure",
    "cmake",
    "cobol",
    "coffeescript",
    "cpp",
    "crystal",
    "csharp",
    "css",
    "csv",
    "cuda",
    "d",
    "dart",
    "delphi",
    "diff",
    "django",
    "dockerfile",
    "dotnet",
    "eiffel",
    "elixir",
    "elm",
    "emacs",
    "erb",
    "erlang",
    "excel",
    "fish",
    "fortran",
    "fsharp",
    "gdscript",
    "gherkin",
    "glsl",
    "go",
    "graphql",
    "groovy",
    "haml",
    "handlebars",
    "haskell",
    "haxe",
    "hlsl",
    "html",
    "http",
    "ini",
    "io",
    "java",
    "javascript",
    "jinja",
    "json",
    "jsonnet",
    "jsx",
    "julia",
    "jupyter",
    "kotlin",
    "latex",
    "lean",
    "less",
    "liquid",
    "lisp",
    "livescript",
    "llvm",
    "lua",
    "makefile",
    "markdown",
    "markup",
    "mathematica",
    "matlab",
    "mdx",
    "mermaid",
    "mips",
    "moonscript",
    "nginx",
    "nim",
    "nix",
    "nushell",
    "objc",
    "objective-c",
    "ocaml",
    "openscad",
    "pascal",
    "perl",
    "php",
    "plaintext",
    "plsql",
    "postcss",
    "powershell",
    "prisma",
    "processing",
    "prolog",
    "protobuf",
    "puppet",
    "purescript",
    "python",
    "r",
    "racket",
    "razor",
    "regex",
    "rescript",
    "rest",
    "ruby",
    "rust",
    "sass",
    "scala",
    "scheme",
    "scss",
    "sh",
    "shell",
    "smalltalk",
    "smarty",
    "solidity",
    "sparql",
    "sql",
    "stan",
    "stylus",
    "svelte",
    "svg",
    "swift",
    "tcl",
    "terraform",
    "tex",
    "text",
    "toml",
    "tsx",
    "twig",
    "typescript",
    "v",
    "vala",
    "vb",
    "vbnet",
    "verilog",
    "vhdl",
    "vim",
    "vue",
    "wasm",
    "wolfram",
    "xml",
    "xquery",
    "xslt",
    "yaml",
    "zig",
    "zsh",
    // Common aliases
    "js",
    "ts",
    "rb",
    "py",
    "rs",
    "cs",
    "hs",
    "yml",
    "sh",
    "md",
];

/// Standardize code blocks within the content subtree.
///
/// Normalizes various syntax highlighter markup into clean
/// `<pre><code data-lang="...">` elements.
pub fn standardize_code_blocks(html: &mut Html, main_content: NodeId) {
    remove_line_numbers(html, main_content);
    remove_copy_buttons(html, main_content);
    normalize_pre_code_blocks(html, main_content);
}

/// Remove line number elements from code blocks.
fn remove_line_numbers(html: &mut Html, main_content: NodeId) {
    let mut to_remove = Vec::new();

    let descendants = dom::all_descendant_elements(html, main_content);
    for node_id in descendants {
        if should_remove_as_line_number(html, node_id) {
            to_remove.push(node_id);
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

/// Check if an element should be removed as a line number.
fn should_remove_as_line_number(html: &Html, node_id: NodeId) -> bool {
    let Some(node_ref) = html.tree.get(node_id) else {
        return false;
    };
    let Node::Element(el) = node_ref.value() else {
        return false;
    };

    let classes = el.attr("class").unwrap_or("");
    for class in classes.split_whitespace() {
        let lower = class.to_ascii_lowercase();
        if LINE_NUMBER_CLASSES.iter().any(|&c| c == lower) {
            return true;
        }
    }

    // <td> cells used for line numbers in table-based highlighters
    let tag = el.name.local.as_ref();
    if tag == "td" && classes.contains("hljs-ln-numbers") {
        return true;
    }

    false
}

/// Remove copy/clipboard buttons from code blocks.
fn remove_copy_buttons(html: &mut Html, main_content: NodeId) {
    let mut to_remove = Vec::new();

    let descendants = dom::all_descendant_elements(html, main_content);
    for node_id in descendants {
        if is_copy_button(html, node_id) {
            to_remove.push(node_id);
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

/// Check if an element is a copy button.
fn is_copy_button(html: &Html, node_id: NodeId) -> bool {
    let Some(node_ref) = html.tree.get(node_id) else {
        return false;
    };
    let Node::Element(el) = node_ref.value() else {
        return false;
    };

    // Check data attributes
    for attr_name in BUTTON_DATA_ATTRS {
        if el.attr(attr_name).is_some() {
            return true;
        }
    }

    // Check class names
    let classes = el.attr("class").unwrap_or("");
    for class in classes.split_whitespace() {
        let lower = class.to_ascii_lowercase();
        if BUTTON_CLASSES.iter().any(|&c| c == lower) {
            return true;
        }
    }

    false
}

/// Find all `<pre>` elements, detect language, unwrap inner spans,
/// and normalize to `<pre><code data-lang="...">`.
fn normalize_pre_code_blocks(html: &mut Html, main_content: NodeId) {
    let pre_ids = dom::descendant_elements_by_tag(html, main_content, "pre");

    for pre_id in pre_ids {
        normalize_single_pre(html, pre_id);
    }
}

/// Normalize a single `<pre>` element into `<pre><code>` form.
fn normalize_single_pre(html: &mut Html, pre_id: NodeId) {
    let language = detect_language(html, pre_id);
    let code_text = extract_code_text(html, pre_id);

    // Remove all existing children of <pre>
    let child_ids: Vec<NodeId> = {
        let Some(node_ref) = html.tree.get(pre_id) else {
            return;
        };
        node_ref.children().map(|c| c.id()).collect()
    };
    for child_id in child_ids {
        dom::remove_node(html, child_id);
    }

    // Create <code> element with data-lang
    let code_el = build_code_element(&language);
    let code_id = html.tree.orphan(Node::Element(code_el)).id();

    // Create text node with the code content
    let text_node = Node::Text(scraper::node::Text {
        text: code_text.into(),
    });
    let text_id = html.tree.orphan(text_node).id();

    // Append text into <code>, then <code> into <pre>
    {
        let Some(mut code_mut) = html.tree.get_mut(code_id) else {
            return;
        };
        code_mut.append_id(text_id);
    }
    {
        let Some(mut pre_mut) = html.tree.get_mut(pre_id) else {
            return;
        };
        pre_mut.append_id(code_id);
    }

    // Strip all attributes from <pre> itself
    let Some(mut pre_mut) = html.tree.get_mut(pre_id) else {
        return;
    };
    let Node::Element(el) = pre_mut.value() else {
        return;
    };
    el.attrs.clear();
}

/// Build a `<code>` element, optionally with `data-lang` attribute.
fn build_code_element(language: &str) -> scraper::node::Element {
    let name = QualName::new(None, ns!(html), markup5ever::local_name!("code"));
    let mut attributes = Vec::new();

    if !language.is_empty() {
        attributes.push(markup5ever::Attribute {
            name: QualName::new(None, ns!(), markup5ever::LocalName::from("data-lang")),
            value: language.into(),
        });
    }

    scraper::node::Element::new(name, attributes)
}

/// Detect the programming language from a `<pre>` and its children.
fn detect_language(html: &Html, pre_id: NodeId) -> String {
    // Check <pre> itself
    if let Some(lang) = detect_language_from_node(html, pre_id) {
        return lang;
    }

    // Check child <code> elements
    let code_children = dom::descendant_elements_by_tag(html, pre_id, "code");
    for code_id in code_children {
        if let Some(lang) = detect_language_from_node(html, code_id) {
            return lang;
        }
    }

    // Check parent elements (wrapper divs with language classes)
    let mut current = pre_id;
    for _ in 0..3 {
        let Some(parent_id) = dom::parent_element(html, current) else {
            break;
        };
        if let Some(lang) = detect_language_from_node(html, parent_id) {
            return lang;
        }
        current = parent_id;
    }

    String::new()
}

/// Try to detect language from a single element's attributes.
fn detect_language_from_node(html: &Html, node_id: NodeId) -> Option<String> {
    let node_ref = html.tree.get(node_id)?;
    let Node::Element(el) = node_ref.value() else {
        return None;
    };

    // Check data-language / data-lang attributes (Shiki, rehype)
    for attr_name in &["data-language", "data-lang", "language"] {
        if let Some(val) = el.attr(attr_name) {
            let lower = val.to_ascii_lowercase();
            if is_known_language(&lower) {
                return Some(lower);
            }
        }
    }

    // Check class names
    let classes = el.attr("class").unwrap_or("");
    detect_language_from_classes(classes)
}

/// Extract language from CSS class names using highlighter patterns.
fn detect_language_from_classes(classes: &str) -> Option<String> {
    for class in classes.split_whitespace() {
        let lower = class.to_ascii_lowercase();

        // Check prefix patterns: language-X, lang-X, etc.
        for prefix in HIGHLIGHTER_PREFIXES {
            if let Some(lang) = lower.strip_prefix(prefix)
                && is_known_language(lang)
            {
                return Some(lang.to_string());
            }
        }

        // Check suffix patterns: X-code, X-snippet
        for suffix in HIGHLIGHTER_SUFFIXES {
            if let Some(lang) = lower.strip_suffix(suffix)
                && is_known_language(lang)
            {
                return Some(lang.to_string());
            }
        }

        // Bare language name as class
        if is_known_language(&lower) {
            return Some(lower);
        }
    }
    None
}

/// Check if a string is a known programming language.
fn is_known_language(name: &str) -> bool {
    LANGUAGES.contains(&name)
}

/// Extract plain text content from a code block, stripping all
/// markup but preserving whitespace structure.
fn extract_code_text(html: &Html, node_id: NodeId) -> String {
    let mut buf = String::new();
    collect_code_text(html, node_id, &mut buf);

    // Clean up: normalize tabs, NBSP, trim
    let cleaned = buf.replace('\t', "    ").replace('\u{00a0}', " ");

    // Trim leading/trailing whitespace, normalize multiple blank lines
    let trimmed = cleaned.trim();
    collapse_blank_lines(trimmed)
}

/// Recursively collect text from a node, skipping line number
/// and button elements.
fn collect_code_text(html: &Html, node_id: NodeId, buf: &mut String) {
    let Some(node_ref) = html.tree.get(node_id) else {
        return;
    };
    match node_ref.value() {
        Node::Text(t) => buf.push_str(t),
        Node::Element(el) => {
            let tag = el.name.local.as_ref();

            // Handle <br> as newline
            if tag == "br" {
                buf.push('\n');
                return;
            }

            // Skip buttons and styles
            if tag == "button" || tag == "style" {
                return;
            }

            for child in node_ref.children() {
                collect_code_text(html, child.id(), buf);
            }
        }
        _ => {}
    }
}

/// Collapse runs of 3+ newlines into 2 newlines.
fn collapse_blank_lines(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut newline_count = 0u32;

    for ch in s.chars() {
        if ch == '\n' {
            newline_count += 1;
            if newline_count <= 2 {
                result.push(ch);
            }
        } else {
            newline_count = 0;
            result.push(ch);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_and_standardize(html_str: &str) -> String {
        let mut doc = Html::parse_document(html_str);
        let root = doc.tree.root().id();
        standardize_code_blocks(&mut doc, root);
        dom::outer_html(&doc, root)
    }

    #[test]
    fn basic_pre_code_passthrough() {
        let html = r"<html><body><pre><code>fn main() {}</code></pre></body></html>";
        let result = parse_and_standardize(html);
        assert!(result.contains("<pre><code>fn main() {}</code></pre>"));
    }

    #[test]
    fn detects_language_from_class() {
        let html = r#"<html><body>
            <pre><code class="language-rust">fn main() {}</code></pre>
        </body></html>"#;
        let result = parse_and_standardize(html);
        assert!(result.contains(r#"data-lang="rust""#));
        assert!(result.contains("fn main() {}"));
    }

    #[test]
    fn detects_prismjs_lang_prefix() {
        let html = r#"<html><body>
            <pre><code class="lang-python">print("hi")</code></pre>
        </body></html>"#;
        let result = parse_and_standardize(html);
        assert!(result.contains(r#"data-lang="python""#));
    }

    #[test]
    fn detects_highlightjs_hljs_class() {
        let html = r#"<html><body>
            <pre><code class="hljs language-javascript">let x = 1;</code></pre>
        </body></html>"#;
        let result = parse_and_standardize(html);
        assert!(result.contains(r#"data-lang="javascript""#));
    }

    #[test]
    fn detects_data_language_attribute() {
        let html = r#"<html><body>
            <pre data-language="go"><code>fmt.Println("hi")</code></pre>
        </body></html>"#;
        let result = parse_and_standardize(html);
        assert!(result.contains(r#"data-lang="go""#));
    }

    #[test]
    fn strips_syntax_highlighting_spans() {
        let html = r#"<html><body>
            <pre><code class="language-rust">
                <span class="keyword">fn</span> <span class="function">main</span>() {}
            </code></pre>
        </body></html>"#;
        let result = parse_and_standardize(html);
        assert!(result.contains(r#"data-lang="rust""#));
        assert!(!result.contains("<span"));
        assert!(result.contains("fn"));
        assert!(result.contains("main"));
    }

    #[test]
    fn removes_line_numbers() {
        let html = r#"<html><body>
            <pre><code class="language-python">
                <span class="lineno">1</span>print("hello")
                <span class="lineno">2</span>print("world")
            </code></pre>
        </body></html>"#;
        let result = parse_and_standardize(html);
        assert!(!result.contains("lineno"));
        assert!(result.contains("print"));
    }

    #[test]
    fn removes_copy_buttons() {
        let html = r#"<html><body>
            <pre>
                <button class="copy-button">Copy</button>
                <code class="language-rust">let x = 1;</code>
            </pre>
        </body></html>"#;
        let result = parse_and_standardize(html);
        assert!(!result.contains("copy-button"));
        assert!(!result.contains("Copy"));
        assert!(result.contains("let x = 1;"));
    }

    #[test]
    fn removes_data_copy_buttons() {
        let html = r#"<html><body>
            <pre>
                <button data-copy="true">Copy</button>
                <code>x = 1</code>
            </pre>
        </body></html>"#;
        let result = parse_and_standardize(html);
        assert!(!result.contains("button"));
        assert!(result.contains("x = 1"));
    }

    #[test]
    fn detects_language_from_parent_div() {
        let html = r#"<html><body>
            <div class="language-typescript">
                <pre><code>const x: number = 1;</code></pre>
            </div>
        </body></html>"#;
        let result = parse_and_standardize(html);
        assert!(result.contains(r#"data-lang="typescript""#));
    }

    #[test]
    fn handles_bare_language_class() {
        let html = r#"<html><body>
            <pre><code class="python">x = 1</code></pre>
        </body></html>"#;
        let result = parse_and_standardize(html);
        assert!(result.contains(r#"data-lang="python""#));
    }

    #[test]
    fn no_language_when_unknown() {
        let html = r#"<html><body>
            <pre><code class="someRandomClass">stuff</code></pre>
        </body></html>"#;
        let result = parse_and_standardize(html);
        assert!(!result.contains("data-lang"));
        assert!(result.contains("stuff"));
    }

    #[test]
    fn normalizes_tabs_and_nbsp() {
        let html = "<html><body><pre><code>x\tfoo\u{00a0}bar</code></pre></body></html>";
        let result = parse_and_standardize(html);
        // Tabs become 4 spaces, NBSP becomes regular space
        assert!(
            result.contains("x    foo bar"),
            "Expected normalized whitespace in: {result:?}"
        );
    }

    #[test]
    fn removes_rouge_gutter() {
        let html = r#"<html><body>
            <pre><code class="language-ruby">
                <td class="rouge-gutter">1</td>
                <td>puts "hi"</td>
            </code></pre>
        </body></html>"#;
        let result = parse_and_standardize(html);
        assert!(!result.contains("rouge-gutter"));
    }

    #[test]
    fn handles_pre_without_code() {
        let html = r"<html><body>
            <pre>plain preformatted text</pre>
        </body></html>";
        let result = parse_and_standardize(html);
        assert!(result.contains("<pre><code>plain preformatted text</code></pre>"));
    }

    #[test]
    fn preserves_br_as_newlines() {
        let html = r"<html><body>
            <pre><code>line1<br>line2<br>line3</code></pre>
        </body></html>";
        let result = parse_and_standardize(html);
        assert!(result.contains("line1\nline2\nline3"));
    }

    #[test]
    fn collapses_excess_blank_lines() {
        let input = "a\n\n\n\n\nb";
        let result = collapse_blank_lines(input);
        assert_eq!(result, "a\n\nb");
    }

    #[test]
    fn is_known_language_works() {
        assert!(is_known_language("rust"));
        assert!(is_known_language("javascript"));
        assert!(is_known_language("typescript"));
        assert!(is_known_language("python"));
        assert!(is_known_language("go"));
        assert!(is_known_language("nix"));
        assert!(!is_known_language("notareallang"));
    }

    #[test]
    fn detects_code_suffix_pattern() {
        assert_eq!(
            detect_language_from_classes("javascript-code"),
            Some("javascript".to_string())
        );
    }

    #[test]
    fn detects_snippet_suffix_pattern() {
        assert_eq!(
            detect_language_from_classes("python-snippet"),
            Some("python".to_string())
        );
    }

    #[test]
    fn pre_attributes_are_stripped() {
        let html = r#"<html><body>
            <pre class="highlight" data-lang="rust"><code>let x = 1;</code></pre>
        </body></html>"#;
        let result = parse_and_standardize(html);
        // <pre> should have no attributes; data-lang goes on <code>
        assert!(result.contains(r#"<pre><code data-lang="rust">"#));
    }
}
