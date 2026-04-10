use ego_tree::NodeId;
use scraper::{Html, Node};

use crate::dom;

/// Resolve React streaming SSR boundaries.
///
/// React's server-side streaming inserts content via script calls
/// like `$RC("B:0","S:0")` that replace `<template id="B:0">`
/// placeholders with hidden `<div id="S:0">` content. Since we're
/// processing static HTML after streaming completes, we inline the
/// resolved content and remove the scripts and hidden containers.
pub fn resolve_streaming_ssr(html: &mut Html) {
    let replacements = collect_rc_calls(html);
    for (boundary_id, segment_id, script_node) in replacements {
        apply_replacement(html, &boundary_id, &segment_id, script_node);
    }
}

/// A pending $RC replacement: (boundary template ID, segment div ID, script `NodeId`).
type RcCall = (String, String, NodeId);

/// Find all `<script>` elements containing `$RC(` calls and parse
/// the boundary and segment IDs.
fn collect_rc_calls(html: &Html) -> Vec<RcCall> {
    let script_ids = dom::select_ids(html, "script:not([src])");
    let mut calls = Vec::new();

    for script_id in script_ids {
        let text = dom::text_content(html, script_id);
        for (boundary, segment) in parse_rc_calls(&text) {
            calls.push((boundary, segment, script_id));
        }
    }

    calls
}

/// Parse `$RC("B:X","S:X")` calls from script text.
/// Returns (`boundary_id`, `segment_id`) pairs.
fn parse_rc_calls(text: &str) -> Vec<(String, String)> {
    let mut results = Vec::new();
    let mut search_from = 0;

    while let Some(pos) = text[search_from..].find("$RC(") {
        let call_start = search_from + pos + 4; // skip "$RC("
        if let Some((boundary, segment)) = parse_single_rc_call(&text[call_start..]) {
            results.push((boundary, segment));
        }
        search_from = call_start;
    }

    results
}

/// Parse a single `$RC` call's arguments: `"B:0","S:0")`.
fn parse_single_rc_call(text: &str) -> Option<(String, String)> {
    let closing = text.find(')')?;
    let args = &text[..closing];

    let mut parts = args.split(',');
    let first = parts.next()?.trim().trim_matches('"');
    let second = parts.next()?.trim().trim_matches('"');

    if first.is_empty() || second.is_empty() {
        return None;
    }

    Some((first.to_string(), second.to_string()))
}

/// Apply a single $RC replacement: move content from the hidden
/// segment div to replace the boundary template's placeholder
/// content, then remove the segment div.
fn apply_replacement(html: &mut Html, boundary_id: &str, segment_id: &str, script_node: NodeId) {
    let template_sel = format!("template[id=\"{boundary_id}\"]");
    let segment_sel = format!("[id=\"{segment_id}\"]");

    let template_ids = dom::select_ids(html, &template_sel);
    let segment_ids = dom::select_ids(html, &segment_sel);

    let Some(&template_id) = template_ids.first() else {
        return;
    };
    let Some(&segment_id_node) = segment_ids.first() else {
        return;
    };

    // Collect children of the segment div
    let children: Vec<NodeId> = html
        .tree
        .get(segment_id_node)
        .map(|n| n.children().map(|c| c.id()).collect())
        .unwrap_or_default();

    // Find the template's parent and insert segment children before
    // the template.
    for child_id in children {
        let Some(mut template_mut) = html.tree.get_mut(template_id) else {
            break;
        };
        template_mut.insert_id_before(child_id);
    }

    // Remove the skeleton/placeholder content between the template
    // and the <!--/$--> comment. Walk siblings after the template
    // position until we hit a comment with "/$".
    remove_skeleton_siblings(html, template_id);

    // Remove template, segment div, and script
    dom::remove_node(html, template_id);
    dom::remove_node(html, segment_id_node);
    dom::remove_node(html, script_node);
}

/// Remove placeholder/skeleton siblings that follow a template
/// element, up to (and including) the `<!--/$-->` comment marker.
fn remove_skeleton_siblings(html: &mut Html, template_id: NodeId) {
    let mut to_remove = Vec::new();
    let Some(template_ref) = html.tree.get(template_id) else {
        return;
    };

    let mut sibling = template_ref.next_sibling();
    while let Some(node) = sibling {
        let next = node.next_sibling();
        let is_end_comment = matches!(node.value(), Node::Comment(c)
            if c.comment.trim() == "/$");

        to_remove.push(node.id());
        if is_end_comment {
            break;
        }
        sibling = next;
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::unwrap_used)]
    #[test]
    fn parse_rc_calls_extracts_ids() {
        let text = r#"$RC("B:0","S:0")"#;
        let calls = parse_rc_calls(text);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "B:0");
        assert_eq!(calls[0].1, "S:0");
    }

    #[allow(clippy::unwrap_used)]
    #[test]
    fn parse_rc_calls_with_surrounding_code() {
        let text = r#"$RB=[];$RC=function(b,c){};$RC("B:0","S:0")"#;
        let calls = parse_rc_calls(text);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "B:0");
        assert_eq!(calls[0].1, "S:0");
    }

    #[allow(clippy::unwrap_used)]
    #[test]
    fn resolve_streaming_ssr_inlines_content() {
        let input = r#"<html><body>
            <div class="content"><!--$?--><template id="B:0"></template>
            <div class="skeleton">Loading...</div><!--/$--></div>
            <div hidden id="S:0"><p>Real content here.</p></div>
            <script>$RC("B:0","S:0")</script>
        </body></html>"#;

        let mut doc = Html::parse_document(input);
        resolve_streaming_ssr(&mut doc);
        let output = dom::outer_html(&doc, doc.tree.root().id());
        assert!(
            output.contains("Real content here"),
            "should inline hidden content"
        );
        assert!(
            !output.contains("skeleton"),
            "should remove skeleton placeholder"
        );
        assert!(
            !output.contains("template"),
            "should remove template element"
        );
    }
}
