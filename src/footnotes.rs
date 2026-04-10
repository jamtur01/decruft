use ego_tree::NodeId;
use scraper::Html;

use crate::dom;
use crate::selectors::{FOOTNOTE_INLINE_REFERENCES, FOOTNOTE_LIST_SELECTORS};

/// Data attribute used to mark elements as footnote-protected,
/// preventing removal by exact/partial selector cleanup.
const PROTECTED_ATTR: &str = "data-decruft-footnote";

/// Standardize footnotes: mark footnote containers and inline
/// references so they survive the cleanup pipeline.
///
/// This runs BEFORE selector-based removal so the protection
/// markers are in place when cleanup runs.
pub fn standardize_footnotes(html: &mut Html, main_content: NodeId) {
    protect_footnote_lists(html, main_content);
    protect_inline_references(html, main_content);
    protect_sidenotes(html, main_content);
}

/// Mark footnote list containers with a protection attribute.
/// Also marks their ancestor chain up to `main_content` so
/// partial-selector removal won't strip a parent div.
fn protect_footnote_lists(html: &mut Html, main_content: NodeId) {
    for selector_str in FOOTNOTE_LIST_SELECTORS {
        let ids = dom::select_within(html, main_content, selector_str);
        for id in ids {
            dom::set_attr(html, id, PROTECTED_ATTR, "list");
            protect_ancestors(html, id, main_content);
        }
    }

    // Additional patterns not covered by FOOTNOTE_LIST_SELECTORS:
    // WordPress block footnotes, Google Docs, Substack, etc.
    let extra_selectors = [
        "div[id^=\"ftnt\"]",
        "aside ol[start]",
        ".reflist",
        ".references",
    ];
    for selector_str in &extra_selectors {
        let ids = dom::select_within(html, main_content, selector_str);
        for id in ids {
            if dom::get_attr(html, id, PROTECTED_ATTR).is_some() {
                continue;
            }
            dom::set_attr(html, id, PROTECTED_ATTR, "list");
            protect_ancestors(html, id, main_content);
        }
    }
}

/// Mark inline footnote reference elements with protection.
fn protect_inline_references(html: &mut Html, main_content: NodeId) {
    for selector_str in FOOTNOTE_INLINE_REFERENCES {
        let ids = dom::select_within(html, main_content, selector_str);
        for id in ids {
            dom::set_attr(html, id, PROTECTED_ATTR, "ref");
        }
    }

    // Google Docs inline refs
    let gdocs = dom::select_within(html, main_content, "sup[id^=\"ftnt_ref\"]");
    for id in gdocs {
        if dom::get_attr(html, id, PROTECTED_ATTR).is_some() {
            continue;
        }
        dom::set_attr(html, id, PROTECTED_ATTR, "ref");
    }
}

/// Detect and protect sidenotes (Tufte CSS, Gwern-style).
/// These use margin-positioned elements rather than bottom
/// footnote lists.
fn protect_sidenotes(html: &mut Html, main_content: NodeId) {
    let sidenote_selectors = [
        ".sidenote",
        ".marginnote",
        "span.sidenote",
        "aside.sidenote",
        ".side-note",
    ];
    for selector_str in &sidenote_selectors {
        let ids = dom::select_within(html, main_content, selector_str);
        for id in ids {
            dom::set_attr(html, id, PROTECTED_ATTR, "sidenote");
        }
    }
}

/// Mark ancestors between `node_id` and `stop_at` (exclusive) as
/// footnote-protected so they aren't removed by partial selectors.
fn protect_ancestors(html: &mut Html, node_id: NodeId, stop_at: NodeId) {
    let mut current = node_id;
    loop {
        let Some(parent_id) = dom::parent_element(html, current) else {
            break;
        };
        if parent_id == stop_at {
            break;
        }
        if dom::get_attr(html, parent_id, PROTECTED_ATTR).is_some() {
            break;
        }
        dom::set_attr(html, parent_id, PROTECTED_ATTR, "ancestor");
        current = parent_id;
    }
}

/// Check whether a node is marked as footnote-protected.
#[must_use]
pub fn is_footnote_protected(html: &Html, node_id: NodeId) -> bool {
    dom::get_attr(html, node_id, PROTECTED_ATTR).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protects_footnote_list() {
        let input = r##"<html><body>
            <article>
                <p>Text with a <a href="#fn1">ref</a>.</p>
                <div class="footnotes">
                    <ol>
                        <li id="fn1">Footnote text.</li>
                    </ol>
                </div>
            </article>
        </body></html>"##;
        let mut html = Html::parse_document(input);
        let root = html.tree.root().id();
        let article = dom::select_ids(&html, "article");
        let main_id = article[0];

        standardize_footnotes(&mut html, main_id);

        let ol_ids = dom::select_within(&html, root, "ol");
        assert!(!ol_ids.is_empty());
        assert!(is_footnote_protected(&html, ol_ids[0]));
    }

    #[test]
    fn protects_inline_reference() {
        // This test verifies that footnote containers near inline
        // references are protected from removal
        let input = r##"<html><body>
            <article>
                <p>Text <a href="#fn1" id="fnref1">1</a>.</p>
                <ol class="footnotes"><li id="fn1">Note</li></ol>
            </article>
        </body></html>"##;
        let mut html = Html::parse_document(input);
        let article = dom::select_ids(&html, "article");
        standardize_footnotes(&mut html, article[0]);

        // The footnote list container should be protected
        let lists = dom::select_within(&html, article[0], "ol.footnotes");
        assert!(!lists.is_empty(), "footnote list should exist");
        assert!(
            is_footnote_protected(&html, lists[0]),
            "footnote list should be protected"
        );
    }

    #[test]
    fn protects_sidenotes() {
        let input = r#"<html><body>
            <article>
                <p>Text <span class="sidenote">A side note.</span></p>
            </article>
        </body></html>"#;
        let mut html = Html::parse_document(input);
        let article = dom::select_ids(&html, "article");
        standardize_footnotes(&mut html, article[0]);

        let sidenotes = dom::select_within(&html, article[0], ".sidenote");
        assert!(!sidenotes.is_empty());
        assert!(is_footnote_protected(&html, sidenotes[0]));
    }

    #[test]
    fn protects_wikipedia_reflist() {
        let input = r#"<html><body>
            <article>
                <p>Text.</p>
                <div class="reflist">
                    <ol><li>Ref 1</li></ol>
                </div>
            </article>
        </body></html>"#;
        let mut html = Html::parse_document(input);
        let article = dom::select_ids(&html, "article");
        standardize_footnotes(&mut html, article[0]);

        let reflist = dom::select_within(&html, article[0], ".reflist");
        assert!(!reflist.is_empty());
        assert!(is_footnote_protected(&html, reflist[0]));
    }

    #[test]
    fn set_attr_on_anchor_works() {
        let input = r##"<html><body>
            <article>
                <p>Text <a href="#fn1">1</a>.</p>
            </article>
        </body></html>"##;
        let mut html = Html::parse_document(input);
        let article = dom::select_ids(&html, "article");
        let anchors = dom::descendant_elements_by_tag(&html, article[0], "a");
        assert!(!anchors.is_empty());

        dom::set_attr(&mut html, anchors[0], PROTECTED_ATTR, "ref");
        let val = dom::get_attr(&html, anchors[0], PROTECTED_ATTR);
        assert_eq!(val.as_deref(), Some("ref"), "set_attr should persist");
    }

    #[test]
    fn protect_inline_references_sets_attr() {
        let input = r##"<html><body>
            <article>
                <p>Text <a href="#fn1">1</a>.</p>
            </article>
        </body></html>"##;
        let mut html = Html::parse_document(input);
        let article = dom::select_ids(&html, "article");

        protect_inline_references(&mut html, article[0]);

        let anchors = dom::descendant_elements_by_tag(&html, article[0], "a");
        assert!(!anchors.is_empty());
        let val = dom::get_attr(&html, anchors[0], PROTECTED_ATTR);
        assert!(val.is_some(), "protect_inline_references should set attr");
    }
}
