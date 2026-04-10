use ego_tree::NodeId;
use markup5ever::{Attribute, QualName, ns};
use scraper::{Html, Node};

use crate::dom;

/// Standardize math content from various renderers (`KaTeX`,
/// `MathJax`, `MathML`, Temml) into a uniform format with
/// `data-latex` attributes.
///
/// Each recognized math element is replaced by a `<math>` element
/// carrying `data-latex`, `display`, and `xmlns` attributes.
pub fn standardize_math(html: &mut Html, main_content: NodeId) {
    standardize_katex(html, main_content);
    standardize_temml(html, main_content);
    standardize_mathjax_v2(html, main_content);
    standardize_mathjax_v3(html, main_content);
    standardize_mathjax_svg(html, main_content);
    standardize_mathml(html, main_content);
    standardize_wikipedia_math(html, main_content);
    cleanup_math_scripts(html, main_content);
}

/// `KaTeX`: `.katex` and `.katex-display` elements.
/// Extract LaTeX from `.katex-mathml annotation[encoding="..."]`.
fn standardize_katex(html: &mut Html, main_content: NodeId) {
    let selector = ".katex-display, .katex:not(.katex-display .katex)";
    let ids = select_within(html, main_content, selector);

    for id in ids {
        let is_block = has_class(html, id, "katex-display");
        let latex = find_katex_latex(html, id);
        let Some(latex) = latex else { continue };
        replace_with_math_element(html, id, &latex, is_block);
    }
}

/// Temml: `.temml` elements — similar structure to `KaTeX`.
fn standardize_temml(html: &mut Html, main_content: NodeId) {
    let ids = select_within(html, main_content, ".temml");

    for id in ids {
        let is_block = has_class(html, id, "temml-display");
        let latex = find_annotation_latex(html, id);
        let Some(latex) = latex else { continue };
        replace_with_math_element(html, id, &latex, is_block);
    }
}

/// `MathJax` v2: `.MathJax` and `.MathJax_Display` elements with
/// sibling `<script type="math/tex">`.
fn standardize_mathjax_v2(html: &mut Html, main_content: NodeId) {
    let ids = select_within(
        html,
        main_content,
        ".MathJax, .MathJax_Display, .MathJax_MathML",
    );

    for id in ids {
        let is_block = is_block_math(html, id);
        let latex = find_mathjax_v2_latex(html, id);
        let Some(latex) = latex else { continue };
        remove_mathjax_siblings(html, id);
        replace_with_math_element(html, id, &latex, is_block);
    }
}

/// `MathJax` v3: `mjx-container` elements.
/// Check `data-latex` attribute first, then inner scripts.
fn standardize_mathjax_v3(html: &mut Html, main_content: NodeId) {
    let ids = select_within(html, main_content, "mjx-container");

    for id in ids {
        let is_block = is_block_math(html, id);
        let latex = find_mathjax_v3_latex(html, id);
        let Some(latex) = latex else { continue };
        replace_with_math_element(html, id, &latex, is_block);
    }
}

/// `MathJax` SVG: `.MathJax_SVG` containers.
fn standardize_mathjax_svg(html: &mut Html, main_content: NodeId) {
    let ids = select_within(html, main_content, ".MathJax_SVG");

    for id in ids {
        let is_block = is_block_math(html, id);
        let latex = find_svg_latex(html, id);
        let Some(latex) = latex else { continue };
        replace_with_math_element(html, id, &latex, is_block);
    }
}

/// `MathML`: `<math>` elements. Add `data-latex` from `alttext` or
/// existing `data-latex`, preserving the `<math>` element.
fn standardize_mathml(html: &mut Html, main_content: NodeId) {
    let ids = select_within(html, main_content, "math:not([data-latex])");

    for id in ids {
        let latex = dom::get_attr(html, id, "alttext").or_else(|| find_annotation_latex(html, id));
        let Some(latex) = latex else { continue };
        set_attr(html, id, "data-latex", &latex);
        if dom::get_attr(html, id, "xmlns").is_none() {
            set_attr(html, id, "xmlns", "http://www.w3.org/1998/Math/MathML");
        }
    }
}

/// Wikipedia: `<math>` inside `.mwe-math-element` with `alttext`.
fn standardize_wikipedia_math(html: &mut Html, main_content: NodeId) {
    let ids = select_within(html, main_content, ".mwe-math-element math[alttext]");

    for id in ids {
        if dom::get_attr(html, id, "data-latex").is_some() {
            continue;
        }
        let Some(alttext) = dom::get_attr(html, id, "alttext") else {
            continue;
        };
        set_attr(html, id, "data-latex", &alttext);
    }
}

/// Remove leftover `<script type="math/tex">` and `MathJax` preview
/// elements that were not already cleaned up.
fn cleanup_math_scripts(html: &mut Html, main_content: NodeId) {
    let selectors = [
        r#"script[type="math/tex"]"#,
        r#"script[type="math/tex; mode=display"]"#,
        ".MathJax_Preview",
    ];
    for sel in &selectors {
        let ids = select_within(html, main_content, sel);
        for id in ids {
            dom::remove_node(html, id);
        }
    }
}

// --- LaTeX extraction helpers ---

/// Find LaTeX in `KaTeX`'s `.katex-mathml annotation` element.
fn find_katex_latex(html: &Html, node_id: NodeId) -> Option<String> {
    let sel = r#".katex-mathml annotation[encoding="application/x-tex"]"#;
    let ann_ids = select_within(html, node_id, sel);
    for ann_id in ann_ids {
        let text = dom::text_content(html, ann_id);
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    // Fallback to any annotation
    find_annotation_latex(html, node_id)
}

/// Find LaTeX from `annotation[encoding="application/x-tex"]`.
fn find_annotation_latex(html: &Html, node_id: NodeId) -> Option<String> {
    let sel = r#"annotation[encoding="application/x-tex"]"#;
    let ann_ids = select_within(html, node_id, sel);
    for ann_id in ann_ids {
        let text = dom::text_content(html, ann_id);
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

/// Find LaTeX for `MathJax` v2: check sibling
/// `<script type="math/tex">` or `data-mathml` annotation.
fn find_mathjax_v2_latex(html: &Html, node_id: NodeId) -> Option<String> {
    // Check data-latex attribute first
    if let Some(latex) = dom::get_attr(html, node_id, "data-latex")
        && !latex.is_empty()
    {
        return Some(latex);
    }

    // Look for sibling script[type="math/tex"]
    if let Some(parent_id) = dom::parent_element(html, node_id) {
        let scripts = select_within(
            html,
            parent_id,
            r#"script[type="math/tex"], script[type="math/tex; mode=display"]"#,
        );
        for script_id in scripts {
            let text = dom::text_content(html, script_id);
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    // Check assistive MathML annotation
    find_annotation_latex(html, node_id)
}

/// Find LaTeX for `MathJax` v3 `mjx-container`.
fn find_mathjax_v3_latex(html: &Html, node_id: NodeId) -> Option<String> {
    // Prefer data-latex attribute
    if let Some(latex) = dom::get_attr(html, node_id, "data-latex")
        && !latex.is_empty()
    {
        return Some(latex);
    }

    // Check inner script[type="math/tex"]
    let scripts = select_within(
        html,
        node_id,
        r#"script[type="math/tex"], script[type="math/tex; mode=display"]"#,
    );
    for script_id in scripts {
        let text = dom::text_content(html, script_id);
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    // Check assistive MathML annotation
    find_annotation_latex(html, node_id)
}

/// Find LaTeX in `MathJax` SVG output (from title or annotation).
fn find_svg_latex(html: &Html, node_id: NodeId) -> Option<String> {
    if let Some(latex) = dom::get_attr(html, node_id, "data-latex")
        && !latex.is_empty()
    {
        return Some(latex);
    }
    // Check title elements inside SVG
    for title_id in dom::descendant_elements_by_tag(html, node_id, "title") {
        let text = dom::text_content(html, title_id);
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    find_annotation_latex(html, node_id)
}

// --- DOM manipulation helpers ---

/// Replace an element with a `<math>` element carrying `data-latex`.
/// Creates a new orphan `<math>` node, inserts it before the target,
/// then removes the target.
fn replace_with_math_element(html: &mut Html, node_id: NodeId, latex: &str, is_block: bool) {
    let display = if is_block { "block" } else { "inline" };
    let math_el = create_math_element(latex, display);
    let new_id = html.tree.orphan(Node::Element(math_el)).id();

    // Insert replacement before the original, then remove original
    let Some(mut node_mut) = html.tree.get_mut(node_id) else {
        return;
    };
    node_mut.insert_id_before(new_id);
    node_mut.detach();
}

/// Build a scraper `Element` for `<math xmlns="..." display="..."
/// data-latex="...">`.
fn create_math_element(latex: &str, display: &str) -> scraper::node::Element {
    let name = QualName::new(None, ns!(), markup5ever::LocalName::from("math"));
    let attrs = vec![
        Attribute {
            name: QualName::new(None, ns!(), markup5ever::LocalName::from("xmlns")),
            value: "http://www.w3.org/1998/Math/MathML".into(),
        },
        Attribute {
            name: QualName::new(None, ns!(), markup5ever::LocalName::from("display")),
            value: display.into(),
        },
        Attribute {
            name: QualName::new(None, ns!(), markup5ever::LocalName::from("data-latex")),
            value: latex.into(),
        },
    ];
    scraper::node::Element::new(name, attrs)
}

/// Set an attribute on an element node.
fn set_attr(html: &mut Html, node_id: NodeId, name: &str, value: &str) {
    let Some(mut node_mut) = html.tree.get_mut(node_id) else {
        return;
    };
    let Node::Element(el) = node_mut.value() else {
        return;
    };
    let qn = QualName::new(None, ns!(), markup5ever::LocalName::from(name));
    el.attrs.retain(|(n, _)| n != &qn);
    el.attrs
        .push((qn, markup5ever::tendril::StrTendril::from(value)));
}

/// Check if an element has a specific CSS class.
fn has_class(html: &Html, node_id: NodeId, class: &str) -> bool {
    let Some(node_ref) = html.tree.get(node_id) else {
        return false;
    };
    let Node::Element(el) = node_ref.value() else {
        return false;
    };
    el.attr("class")
        .is_some_and(|c| c.split_whitespace().any(|cls| cls == class))
}

/// Determine if a math element is block-level display.
fn is_block_math(html: &Html, node_id: NodeId) -> bool {
    // Check display attribute
    if let Some(display) = dom::get_attr(html, node_id, "display")
        && (display == "block" || display == "true")
    {
        return true;
    }

    // Check class names for display/block indicators
    if has_class(html, node_id, "MathJax_Display") {
        return true;
    }

    // Check parent for display class
    if let Some(parent_id) = dom::parent_element(html, node_id)
        && (has_class(html, parent_id, "MathJax_Display")
            || has_class(html, parent_id, "katex-display"))
    {
        return true;
    }

    false
}

/// Select elements matching a CSS selector that are descendants of
/// `ancestor_id`.
fn select_within(html: &Html, ancestor_id: NodeId, selector: &str) -> Vec<NodeId> {
    dom::select_ids(html, selector)
        .into_iter()
        .filter(|&id| id == ancestor_id || dom::is_ancestor(html, id, ancestor_id))
        .collect()
}

/// Remove `MathJax` preview and script siblings.
fn remove_mathjax_siblings(html: &mut Html, node_id: NodeId) {
    let Some(parent_id) = dom::parent_element(html, node_id) else {
        return;
    };
    let scripts = select_within(
        html,
        parent_id,
        r#"script[type="math/tex"], script[type="math/tex; mode=display"], .MathJax_Preview"#,
    );
    for script_id in scripts {
        dom::remove_node(html, script_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_and_standardize(html_str: &str) -> String {
        let mut html = Html::parse_document(html_str);
        let root = html.tree.root().id();
        standardize_math(&mut html, root);
        dom::outer_html(&html, root)
    }

    #[test]
    fn katex_inline() {
        let input = r#"<html><body><span class="katex">
            <span class="katex-mathml">
                <math><semantics><annotation encoding="application/x-tex">x^2</annotation></semantics></math>
            </span>
            <span class="katex-html">rendered</span>
        </span></body></html>"#;
        let output = parse_and_standardize(input);
        assert!(output.contains(r#"data-latex="x^2""#));
        assert!(output.contains(r#"display="inline""#));
        assert!(!output.contains("katex-html"));
    }

    #[test]
    fn katex_display() {
        let input = r#"<html><body><span class="katex-display"><span class="katex">
            <span class="katex-mathml">
                <math><semantics><annotation encoding="application/x-tex">\sum_{i=0}^n</annotation></semantics></math>
            </span>
            <span class="katex-html">rendered</span>
        </span></span></body></html>"#;
        let output = parse_and_standardize(input);
        assert!(output.contains(r#"data-latex="\sum_{i=0}^n""#));
        assert!(output.contains(r#"display="block""#));
    }

    #[test]
    fn mathjax_v3_data_latex() {
        let input = r#"<html><body>
            <mjx-container data-latex="E=mc^2" display="true">
                <svg>...</svg>
            </mjx-container>
        </body></html>"#;
        let output = parse_and_standardize(input);
        assert!(output.contains(r#"data-latex="E=mc^2""#));
        assert!(output.contains("<math"));
    }

    #[test]
    fn mathml_alttext() {
        let input = r#"<html><body>
            <math alttext="x + y">
                <mi>x</mi><mo>+</mo><mi>y</mi>
            </math>
        </body></html>"#;
        let output = parse_and_standardize(input);
        assert!(output.contains(r#"data-latex="x + y""#));
        assert!(output.contains("<math"));
    }

    #[test]
    fn wikipedia_math() {
        let input = r#"<html><body>
            <span class="mwe-math-element">
                <math alttext="\alpha + \beta" xmlns="http://www.w3.org/1998/Math/MathML">
                    <mi>α</mi><mo>+</mo><mi>β</mi>
                </math>
            </span>
        </body></html>"#;
        let output = parse_and_standardize(input);
        assert!(
            output.contains(r#"data-latex="\alpha + \beta""#),
            "output: {output}"
        );
    }

    #[test]
    fn mathjax_v2_with_script() {
        let input = r#"<html><body>
            <span class="MathJax">rendered</span>
            <script type="math/tex">f(x) = x^2</script>
        </body></html>"#;
        let output = parse_and_standardize(input);
        assert!(output.contains(r#"data-latex="f(x) = x^2""#));
        // Script should be cleaned up
        assert!(!output.contains(r#"type="math/tex""#));
    }

    #[test]
    fn mathjax_svg() {
        let input = r#"<html><body>
            <div class="MathJax_SVG">
                <svg><title>E=mc^2</title></svg>
            </div>
        </body></html>"#;
        let output = parse_and_standardize(input);
        assert!(output.contains(r#"data-latex="E=mc^2""#));
    }

    #[test]
    fn temml_inline() {
        let input = r#"<html><body><span class="temml">
            <math><semantics>
                <annotation encoding="application/x-tex">a + b</annotation>
            </semantics></math>
        </span></body></html>"#;
        let output = parse_and_standardize(input);
        assert!(output.contains(r#"data-latex="a + b""#));
    }

    #[test]
    fn no_math_is_noop() {
        let input = "<html><body><p>No math here</p></body></html>";
        let output = parse_and_standardize(input);
        assert!(output.contains("No math here"));
        assert!(!output.contains("data-latex"));
    }

    #[test]
    fn already_has_data_latex_on_math() {
        let input = r#"<html><body>
            <math data-latex="y = mx + b" display="inline">
                <mi>y</mi>
            </math>
        </body></html>"#;
        let output = parse_and_standardize(input);
        // Should not duplicate or remove the existing attribute
        assert!(output.contains(r#"data-latex="y = mx + b""#));
    }
}
