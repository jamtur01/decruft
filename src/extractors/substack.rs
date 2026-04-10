use scraper::{Html, Selector};

use crate::dom;

/// Result of Substack note extraction.
pub struct SubstackContent {
    pub html: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub site: Option<String>,
    pub image: Option<String>,
}

/// Detect whether this page is a Substack note or app page.
#[must_use]
pub fn is_substack(html: &Html, url: Option<&str>) -> bool {
    if url.is_some_and(|u| u.contains("substack.com")) {
        return true;
    }
    get_meta(html, "property", "og:site_name").is_some_and(|s| s == "Substack")
}

/// Check if this is a Substack notes page (not an article).
fn is_notes_page(html: &Html, url: Option<&str>) -> bool {
    if url.is_some_and(|u| u.contains("/note/")) {
        return true;
    }
    has_class_prefix(html, "feedCommentBody-")
}

/// Extract content from a Substack page.
///
/// Returns `None` if the page is not a Substack page or has no
/// extractable note content.
#[must_use]
pub fn extract_substack_content(html: &Html, url: Option<&str>) -> Option<SubstackContent> {
    if !is_substack(html, url) || !is_notes_page(html, url) {
        return None;
    }

    let title = get_meta(html, "property", "og:title");
    let author = extract_author_name(html);
    let og_image = get_meta(html, "property", "og:image");

    let body_html = extract_note_body(html, url, og_image.as_deref())?;
    if body_html.trim().is_empty() {
        return None;
    }

    Some(SubstackContent {
        html: body_html,
        title,
        author,
        site: Some("Substack".to_string()),
        image: og_image,
    })
}

/// Extract the note body HTML from a Substack notes page.
///
/// For permalink pages (URL contains `/note/`), find the permalink
/// unit first. For feed pages with multiple notes, match against
/// `og:description` to find the featured note.
fn extract_note_body(html: &Html, url: Option<&str>, og_image: Option<&str>) -> Option<String> {
    if url.is_some_and(|u| u.contains("/note/"))
        && let Some(content) = extract_permalink_note(html, og_image)
    {
        return Some(content);
    }

    let og_desc = get_meta(html, "property", "og:description");
    extract_matching_note_body(html, og_image, og_desc.as_deref())
}

/// Extract content from a permalink note page.
///
/// Permalink pages have a `feedPermalinkUnit-` container that holds
/// the main note, distinct from sidebar feed notes.
fn extract_permalink_note(html: &Html, og_image: Option<&str>) -> Option<String> {
    let permalink_id = find_element_with_class_prefix(html, "feedPermalinkUnit-")?;
    let body_id = find_feed_comment_body_within(html, permalink_id)?;
    let prose_html = extract_prose_mirror_html(html, body_id);
    let has_grid = has_image_grid_sibling(html, body_id);
    Some(combine_content(&prose_html, og_image, has_grid))
}

/// Find the note body matching `og:description`, falling back to
/// the first note if no match is found.
fn extract_matching_note_body(
    html: &Html,
    og_image: Option<&str>,
    og_desc: Option<&str>,
) -> Option<String> {
    let body_ids = find_all_feed_comment_bodies(html);
    if body_ids.is_empty() {
        return None;
    }

    let is_single = body_ids.len() == 1;

    if let Some(desc) = og_desc {
        let prefix = &desc[..desc.len().min(60)];
        for &body_id in &body_ids {
            let text = dom::text_content(html, body_id);
            if text.trim().starts_with(prefix) {
                let prose = extract_prose_mirror_html(html, body_id);
                let grid = is_single && has_image_grid_sibling(html, body_id);
                return Some(combine_content(&prose, og_image, grid));
            }
        }
    }

    let body_id = body_ids[0];
    let prose = extract_prose_mirror_html(html, body_id);
    let grid = is_single && has_image_grid_sibling(html, body_id);
    Some(combine_content(&prose, og_image, grid))
}

fn has_class_with_prefix(el: &scraper::node::Element, prefix: &str) -> bool {
    el.attr("class")
        .is_some_and(|c| c.split_whitespace().any(|cls| cls.starts_with(prefix)))
}

/// Find the first element with a class matching the given prefix.
fn find_element_with_class_prefix(html: &Html, prefix: &str) -> Option<ego_tree::NodeId> {
    for node_ref in html.tree.nodes() {
        if let scraper::Node::Element(el) = node_ref.value()
            && has_class_with_prefix(el, prefix)
        {
            return Some(node_ref.id());
        }
    }
    None
}

/// Check if any element on the page has a class with the given prefix.
fn has_class_prefix(html: &Html, prefix: &str) -> bool {
    find_element_with_class_prefix(html, prefix).is_some()
}

/// Find a `feedCommentBody-*` element within a subtree.
fn find_feed_comment_body_within(
    html: &Html,
    ancestor_id: ego_tree::NodeId,
) -> Option<ego_tree::NodeId> {
    let node_ref = html.tree.get(ancestor_id)?;
    for descendant in node_ref.descendants() {
        if let scraper::Node::Element(el) = descendant.value()
            && has_class_with_prefix(el, "feedCommentBody-")
        {
            return Some(descendant.id());
        }
    }
    None
}

/// Find all `feedCommentBody-*` elements on the page.
fn find_all_feed_comment_bodies(html: &Html) -> Vec<ego_tree::NodeId> {
    let mut ids = Vec::new();
    for node_ref in html.tree.nodes() {
        if let scraper::Node::Element(el) = node_ref.value()
            && has_class_with_prefix(el, "feedCommentBody-")
        {
            ids.push(node_ref.id());
        }
    }
    ids
}

/// Extract the inner HTML from a `ProseMirror FeedProseMirror` div
/// within the given node.
fn extract_prose_mirror_html(html: &Html, node_id: ego_tree::NodeId) -> String {
    let Ok(sel) = Selector::parse(".ProseMirror.FeedProseMirror") else {
        return String::new();
    };

    for elem_ref in html.select(&sel) {
        if elem_ref.id() == node_id || dom::is_ancestor(html, elem_ref.id(), node_id) {
            return dom::inner_html(html, elem_ref.id());
        }
    }
    String::new()
}

/// Check whether an image grid sibling exists near a
/// feedCommentBody element.
///
/// Only checks direct children of the feedCommentBody's parent and
/// grandparent to avoid matching unrelated image grids.
fn has_image_grid_sibling(html: &Html, body_id: ego_tree::NodeId) -> bool {
    let Some(parent_id) = dom::parent_element(html, body_id) else {
        return false;
    };
    if has_image_grid_child(html, parent_id) {
        return true;
    }
    dom::parent_element(html, parent_id).is_some_and(|gp| has_image_grid_child(html, gp))
}

/// Check if any direct child of `parent_id` has an `imageGrid-*`
/// class.
fn has_image_grid_child(html: &Html, parent_id: ego_tree::NodeId) -> bool {
    let Some(node_ref) = html.tree.get(parent_id) else {
        return false;
    };
    node_ref.children().any(|child| {
        if let scraper::Node::Element(el) = child.value() {
            has_class_with_prefix(el, "imageGrid-")
        } else {
            false
        }
    })
}

fn combine_content(prose_html: &str, og_image: Option<&str>, has_image_grid: bool) -> String {
    if let (true, Some(img_url)) = (has_image_grid, og_image) {
        let escaped = html_attr_escape(img_url);
        return format!("{prose_html}<img src=\"{escaped}\">");
    }
    prose_html.to_string()
}

fn html_attr_escape(s: &str) -> String {
    dom::html_attr_escape(s)
}

/// Extract author name from the note's author link.
fn extract_author_name(html: &Html) -> Option<String> {
    let og_title = get_meta(html, "property", "og:title")?;
    if let Some(idx) = og_title.find(" (@") {
        return Some(og_title[..idx].to_string());
    }
    Some(og_title)
}

fn get_meta(html: &Html, attr: &str, value: &str) -> Option<String> {
    let sel_str = format!("meta[{attr}=\"{value}\"]");
    let sel = Selector::parse(&sel_str).ok()?;
    let el = html.select(&sel).next()?;
    el.value().attr("content").map(String::from)
}
