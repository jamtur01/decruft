use std::collections::{BTreeMap, HashMap, HashSet};

use ego_tree::NodeId;
use markup5ever::{Attribute, QualName, ns};
use regex::Regex;
use scraper::{Html, Node};

use crate::dom;
use crate::selectors::{FOOTNOTE_INLINE_REFERENCES, FOOTNOTE_LIST_SELECTORS};

/// Collected footnote data before building the canonical HTML.
struct FootnoteData {
    /// Inner HTML content of the footnote.
    content_html: String,
    /// The original ID used to link to this footnote.
    original_id: String,
}

/// Regex matching heading text that delimits a footnote section.
fn footnote_section_re() -> &'static Regex {
    use std::sync::LazyLock;
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)^(foot\s*notes?|end\s*notes?|notes?|references?)$")
            .expect("footnote section regex is valid")
    });
    &RE
}

/// Standardize footnotes: collect footnote definitions and inline
/// references from various formats, then emit canonical HTML.
///
/// Canonical inline ref:  `<sup id="fnref:N"><a href="#fn:N">N</a></sup>`
/// Canonical list:        `<div id="footnotes"><ol><li id="fn:N">...</li></ol></div>`
///
/// Runs BEFORE selector-based removal.
pub fn standardize_footnotes(html: &mut Html, main_content: NodeId) {
    let mut footnotes: BTreeMap<usize, FootnoteData> = BTreeMap::new();
    let mut containers_to_remove: Vec<NodeId> = Vec::new();

    collect_inline_sidenotes(
        html,
        main_content,
        &mut footnotes,
        &mut containers_to_remove,
    );
    collect_selector_footnotes(
        html,
        main_content,
        &mut footnotes,
        &mut containers_to_remove,
    );
    collect_aside_footnotes(
        html,
        main_content,
        &mut footnotes,
        &mut containers_to_remove,
    );

    if footnotes.is_empty() {
        collect_word_footnotes(
            html,
            main_content,
            &mut footnotes,
            &mut containers_to_remove,
        );
    }
    if footnotes.is_empty() {
        collect_google_docs_footnotes(
            html,
            main_content,
            &mut footnotes,
            &mut containers_to_remove,
        );
    }
    if footnotes.is_empty() {
        collect_generic_footnotes(
            html,
            main_content,
            &mut footnotes,
            &mut containers_to_remove,
        );
    }
    if footnotes.is_empty() {
        collect_loose_footnotes(
            html,
            main_content,
            &mut footnotes,
            &mut containers_to_remove,
        );
    }
    if footnotes.is_empty() {
        collect_class_footnotes(
            html,
            main_content,
            &mut footnotes,
            &mut containers_to_remove,
        );
    }

    if footnotes.is_empty() {
        return;
    }

    standardize_inline_refs(html, main_content, &footnotes);
    remove_containers(html, &containers_to_remove);
    remove_orphaned_hrs(html, main_content);
    build_canonical_list(html, main_content, &footnotes);
}

// ── Inline sidenotes (Tufte/Maggie Appleton style) ──────────────

/// Collect inline footnotes from `span.footnote-container` or
/// `span.inline-footnote` and replace the container with a canonical
/// inline `<sup>` reference.
fn collect_inline_sidenotes(
    html: &mut Html,
    main_content: NodeId,
    footnotes: &mut BTreeMap<usize, FootnoteData>,
    _containers: &mut Vec<NodeId>,
) {
    let sel = "span.footnote-container, span.sidenote-container, \
               span.inline-footnote";
    let containers = dom::select_within(html, main_content, sel);
    if containers.is_empty() {
        // Remove standalone sidenotes that duplicate formal lists
        let sidenotes = dom::select_within(html, main_content, "span.sidenote");
        for id in sidenotes {
            dom::remove_node(html, id);
        }
        return;
    }

    let mut count = footnotes.len() + 1;
    for container_id in containers {
        let content_sel = "span.footnote, span.sidenote, span.footnoteContent";
        let content_ids = dom::select_within(html, container_id, content_sel);
        let Some(&content_id) = content_ids.first() else {
            continue;
        };

        let content_html = dom::inner_html(html, content_id);
        if content_html.trim().is_empty() {
            continue;
        }

        let num = count;
        let ref_id = format!("fnref:{num}");
        footnotes.insert(
            num,
            FootnoteData {
                content_html,
                original_id: num.to_string(),
            },
        );

        let sup_id = create_footnote_ref(html, num, &ref_id);
        replace_node(html, container_id, sup_id);

        count += 1;
    }
}

// ── Selector-based footnote lists ───────────────────────────────

/// Collect footnotes from elements matching `FOOTNOTE_LIST_SELECTORS`.
fn collect_selector_footnotes(
    html: &Html,
    main_content: NodeId,
    footnotes: &mut BTreeMap<usize, FootnoteData>,
    containers: &mut Vec<NodeId>,
) {
    let mut processed_ids: HashSet<String> = HashSet::new();
    let mut count = footnotes.len() + 1;

    for selector_str in FOOTNOTE_LIST_SELECTORS {
        let lists = dom::select_within(html, main_content, selector_str);
        for list_id in &lists {
            collect_list_items(html, *list_id, &mut count, &mut processed_ids, footnotes);
        }
        containers.extend(lists);
    }

    // Walk up from each container to find wrapping parent
    // (e.g. section.footnotes wrapping ol)
    let all_containers: Vec<NodeId> = containers.clone();
    for &cid in &all_containers {
        if let Some(pid) = dom::parent_element(html, cid)
            && pid != main_content
            && is_footnote_wrapper(html, pid)
        {
            containers.push(pid);
        }
    }
}

/// Check if a node is a footnote wrapper (section/div with footnote
/// class or role).
fn is_footnote_wrapper(html: &Html, node_id: NodeId) -> bool {
    let tag = dom::tag_name(html, node_id).unwrap_or_default();
    if tag != "section" && tag != "div" {
        return false;
    }
    let cls = dom::get_attr(html, node_id, "class").unwrap_or_default();
    let role = dom::get_attr(html, node_id, "role").unwrap_or_default();
    cls.contains("footnote") || role.contains("doc-endnotes") || role.contains("doc-footnotes")
}

/// Extract list items from a footnote list container.
fn collect_list_items(
    html: &Html,
    list_id: NodeId,
    count: &mut usize,
    processed_ids: &mut HashSet<String>,
    footnotes: &mut BTreeMap<usize, FootnoteData>,
) {
    // Hugo/org-mode: div.footnote-definitions > div.footnote-definition
    let def_items = dom::select_within(html, list_id, "div.footnote-definition");
    if !def_items.is_empty() {
        collect_definition_items(html, &def_items, count, processed_ids, footnotes);
        return;
    }

    let item_sel = "li, div[role=\"listitem\"]";
    let items = dom::select_within(html, list_id, item_sel);

    for item_id in items {
        let id = extract_item_id(html, item_id);
        if id.is_empty() || !processed_ids.insert(id.clone()) {
            continue;
        }

        let content = strip_backrefs_from_html(&dom::inner_html(html, item_id));

        footnotes.insert(
            *count,
            FootnoteData {
                content_html: content,
                original_id: id,
            },
        );
        *count += 1;
    }
}

/// Collect from div.footnote-definition elements containing
/// sup[id] + .footnote-body.
fn collect_definition_items(
    html: &Html,
    items: &[NodeId],
    count: &mut usize,
    processed_ids: &mut HashSet<String>,
    footnotes: &mut BTreeMap<usize, FootnoteData>,
) {
    for &def_id in items {
        let sup_ids = dom::select_within(html, def_id, "sup[id]");
        let body_ids = dom::select_within(html, def_id, ".footnote-body");
        let Some(&sup_id) = sup_ids.first() else {
            continue;
        };
        let Some(&body_id) = body_ids.first() else {
            continue;
        };

        let id = dom::get_attr(html, sup_id, "id")
            .unwrap_or_default()
            .to_lowercase();
        if id.is_empty() || !processed_ids.insert(id.clone()) {
            continue;
        }

        let content = strip_backrefs_from_html(&dom::inner_html(html, body_id));

        footnotes.insert(
            *count,
            FootnoteData {
                content_html: content,
                original_id: id,
            },
        );
        *count += 1;
    }
}

/// Extract a footnote ID from a list item element.
fn extract_item_id(html: &Html, item_id: NodeId) -> String {
    let raw_id = dom::get_attr(html, item_id, "id").unwrap_or_default();
    let lower = raw_id.to_lowercase();

    if lower.starts_with("fn:") {
        return lower.strip_prefix("fn:").unwrap_or(&lower).to_string();
    }
    if lower.starts_with("fn") {
        return lower.strip_prefix("fn").unwrap_or(&lower).to_string();
    }

    // Wikipedia cite_note pattern
    if let Some(suffix) = lower.split("cite_note-").nth(1) {
        return suffix.to_string();
    }

    // data-counter attribute (Nature.com)
    if let Some(counter) = dom::get_attr(html, item_id, "data-counter") {
        return counter.trim_end_matches('.').to_lowercase();
    }

    lower
}

// ── Aside footnotes (ol[start] inside <aside>) ─────────────────

fn collect_aside_footnotes(
    html: &Html,
    main_content: NodeId,
    footnotes: &mut BTreeMap<usize, FootnoteData>,
    containers: &mut Vec<NodeId>,
) {
    let ols = dom::select_within(html, main_content, "aside > ol[start]");
    for ol_id in ols {
        let start_attr = dom::get_attr(html, ol_id, "start").unwrap_or_default();
        let Ok(start_num) = start_attr.parse::<usize>() else {
            continue;
        };
        if start_num < 1 {
            continue;
        }

        let items = dom::select_within(html, ol_id, "li");
        if items.is_empty() {
            continue;
        }

        let content = if items.len() == 1 {
            dom::inner_html(html, items[0])
        } else {
            let mut combined = String::new();
            for item in &items {
                combined.push_str("<p>");
                combined.push_str(&dom::inner_html(html, *item));
                combined.push_str("</p>");
            }
            combined
        };

        footnotes.insert(
            start_num,
            FootnoteData {
                content_html: strip_backrefs_from_html(&content),
                original_id: start_num.to_string(),
            },
        );

        // Remove the <aside> parent
        if let Some(aside_id) = dom::parent_element(html, ol_id) {
            containers.push(aside_id);
        }
    }
}

// ── Microsoft Word HTML footnotes ───────────────────────────────

fn collect_word_footnotes(
    html: &Html,
    main_content: NodeId,
    footnotes: &mut BTreeMap<usize, FootnoteData>,
    containers: &mut Vec<NodeId>,
) {
    let backrefs = dom::select_within(html, main_content, "a[href*=\"#_ftnref\"]");
    if backrefs.len() < 2 {
        return;
    }

    let num_re = Regex::new(r"^_ftnref(\d+)$").expect("word ftnref regex is valid");

    for backref_id in &backrefs {
        let href = dom::get_attr(html, *backref_id, "href").unwrap_or_default();
        let fragment = href.split('#').next_back().unwrap_or("");
        let Some(caps) = num_re.captures(fragment) else {
            continue;
        };
        let Ok(num) = caps[1].parse::<usize>() else {
            continue;
        };

        // Walk up to find the container (p, div, or li)
        let container_id = find_block_ancestor(html, *backref_id, main_content);
        let Some(cid) = container_id else { continue };

        let content = strip_backrefs_from_html(&dom::inner_html(html, cid));

        footnotes.insert(
            num,
            FootnoteData {
                content_html: content,
                original_id: format!("_ftn{num}"),
            },
        );
        containers.push(cid);
    }
}

/// Walk up from `node_id` to find the nearest block-level ancestor
/// (p, div, li) before `stop_at`.
fn find_block_ancestor(html: &Html, node_id: NodeId, stop_at: NodeId) -> Option<NodeId> {
    let mut current = node_id;
    loop {
        let parent = dom::parent_element(html, current)?;
        if parent == stop_at {
            return None;
        }
        let tag = dom::tag_name(html, parent).unwrap_or_default();
        if tag == "p" || tag == "div" || tag == "li" {
            return Some(parent);
        }
        current = parent;
    }
}

// ── Google Docs footnotes ───────────────────────────────────────

fn collect_google_docs_footnotes(
    html: &Html,
    main_content: NodeId,
    footnotes: &mut BTreeMap<usize, FootnoteData>,
    containers: &mut Vec<NodeId>,
) {
    let paras = dom::select_within(html, main_content, "p[id^=\"ftnt\"]");
    let num_re = Regex::new(r"^ftnt(\d+)$").expect("gdocs ftnt regex is valid");

    let mut pairs: Vec<(usize, NodeId)> = Vec::new();
    for p_id in &paras {
        let id = dom::get_attr(html, *p_id, "id").unwrap_or_default();
        let Some(caps) = num_re.captures(&id) else {
            continue;
        };
        let Ok(num) = caps[1].parse::<usize>() else {
            continue;
        };
        pairs.push((num, *p_id));
    }

    if pairs.len() < 2 {
        return;
    }
    pairs.sort_by_key(|(n, _)| *n);

    let mut count = footnotes.len() + 1;
    for (num, p_id) in &pairs {
        let original_id = format!("ftnt{num}");
        let content = strip_backrefs_from_html(&dom::inner_html(html, *p_id));

        footnotes.insert(
            count,
            FootnoteData {
                content_html: content,
                original_id,
            },
        );
        count += 1;

        containers.push(*p_id);
        // Also remove wrapper div if it has only this child
        if let Some(parent) = dom::parent_element(html, *p_id)
            && dom::tag_name(html, parent).as_deref() == Some("div")
            && dom::child_elements(html, parent).len() == 1
        {
            containers.push(parent);
        }
    }

    // Remove "Footnotes" heading preceding the first footnote
    if let Some((_, first_p)) = pairs.first() {
        remove_preceding_footnote_heading(html, *first_p, containers);
    }
}

// ── Generic ID-based footnote detection ─────────────────────────

fn collect_generic_footnotes(
    html: &Html,
    main_content: NodeId,
    footnotes: &mut BTreeMap<usize, FootnoteData>,
    containers: &mut Vec<NodeId>,
) {
    let num_re = Regex::new(r"^\[?\(?\d{1,4}\)?\]?$").expect("generic num regex is valid");

    // Find all anchors with fragment links and numeric text
    let all_anchors = dom::select_within(html, main_content, "a[href*=\"#\"]");
    let mut candidate_refs: HashMap<String, Vec<NodeId>> = HashMap::new();

    for a_id in &all_anchors {
        let href = dom::get_attr(html, *a_id, "href").unwrap_or_default();
        let fragment = href.split('#').next_back().unwrap_or("").to_lowercase();
        if fragment.is_empty() {
            continue;
        }
        let text = dom::text_content(html, *a_id).trim().to_string();
        if !num_re.is_match(&text) {
            continue;
        }
        candidate_refs.entry(fragment).or_default().push(*a_id);
    }

    if candidate_refs.len() < 2 {
        return;
    }

    let fragment_set: HashSet<String> = candidate_refs.keys().cloned().collect();

    // Find the best container holding footnote targets
    let container_sel = "div, section, aside, footer, ol, ul";
    let all_containers = dom::select_within(html, main_content, container_sel);

    let mut best_container: Option<NodeId> = None;
    let mut best_count = 0_usize;

    for cid in &all_containers {
        if *cid == main_content {
            continue;
        }
        let matches = find_matching_footnote_elements(html, *cid, &fragment_set);
        if matches.len() >= 2 && matches.len() >= best_count {
            best_count = matches.len();
            best_container = Some(*cid);
        }
    }

    let Some(best_cid) = best_container else {
        return;
    };

    // Validate: require >= 75% of external refs point into container
    let ordered = find_matching_footnote_elements(html, best_cid, &fragment_set);
    let fn_fragments: HashSet<String> = ordered.iter().map(|(_, id)| id.clone()).collect();

    let mut external_total = 0_usize;
    let mut external_match = 0_usize;
    for (frag, anchors) in &candidate_refs {
        let any_inside = anchors.iter().any(|a| dom::is_ancestor(html, *a, best_cid));
        if any_inside {
            continue; // back-link
        }
        external_total += 1;
        if fn_fragments.contains(frag) {
            external_match += 1;
        }
    }
    let threshold = 2.max((external_total * 3).div_ceil(4));
    if external_match < threshold {
        return;
    }

    let mut count = footnotes.len() + 1;
    let mut processed: HashSet<String> = HashSet::new();
    for (el_id, id) in &ordered {
        if !processed.insert(id.clone()) {
            continue;
        }

        let content = strip_numeric_prefix_and_backrefs(html, *el_id, id);

        footnotes.insert(
            count,
            FootnoteData {
                content_html: content,
                original_id: id.clone(),
            },
        );
        count += 1;
    }

    containers.push(best_cid);
}

/// Find elements inside a container whose id (or child anchor id)
/// matches the fragment set.
fn find_matching_footnote_elements(
    html: &Html,
    container: NodeId,
    fragment_set: &HashSet<String>,
) -> Vec<(NodeId, String)> {
    let mut results = Vec::new();
    let mut seen = HashSet::new();

    let els = dom::select_within(html, container, "li, p, div");
    for el_id in els {
        let el_raw_id = dom::get_attr(html, el_id, "id").unwrap_or_default();
        let el_lower = el_raw_id.to_lowercase();

        let id = if !el_lower.is_empty() && fragment_set.contains(&el_lower) {
            el_lower
        } else if el_raw_id.is_empty() {
            let anchor_id = get_child_anchor_id(html, el_id);
            if !anchor_id.is_empty() && fragment_set.contains(&anchor_id) {
                anchor_id
            } else {
                continue;
            }
        } else {
            continue;
        };

        if seen.insert(id.clone()) {
            results.push((el_id, id));
        }
    }
    results
}

/// Get the id or name of the first child anchor element.
fn get_child_anchor_id(html: &Html, el_id: NodeId) -> String {
    let anchors = dom::select_within(html, el_id, "a[id], a[name]");
    for a_id in anchors {
        if let Some(id) = dom::get_attr(html, a_id, "id")
            && !id.is_empty()
        {
            return id.to_lowercase();
        }
        if let Some(name) = dom::get_attr(html, a_id, "name")
            && !name.is_empty()
        {
            return name.to_lowercase();
        }
    }
    String::new()
}

/// Strip numeric prefix and backrefs from a footnote element's
/// content, returning clean inner HTML.
fn strip_numeric_prefix_and_backrefs(html: &Html, el_id: NodeId, id: &str) -> String {
    let mut content = dom::inner_html(html, el_id);

    // Remove id-anchor with empty/numeric text (e.g. <a id="r1"></a>)
    let id_anchor_re = Regex::new(&format!(
        r#"<a\s[^>]*id="{}"[^>]*>(\d+[.)\s]*)?</a>"#,
        regex::escape(id)
    ))
    .ok();
    if let Some(re) = id_anchor_re {
        content = re.replace(&content, "").to_string();
    }

    // Remove name-anchor (Gutenberg style)
    let name_re = Regex::new(&format!(
        r#"<a\s[^>]*name="{}"[^>]*>[^<]*</a>"#,
        regex::escape(id)
    ))
    .ok();
    if let Some(re) = name_re {
        content = re.replace(&content, "").to_string();
    }

    // Strip leading digit prefix
    let digit_re = Regex::new(r"^\s*\d+\.\s*").expect("digit prefix regex is valid");
    content = digit_re.replace(&content, "").to_string();

    strip_backrefs_from_html(&content)
}

// ── Loose footnotes (hr/heading delimited paragraphs) ───────────

fn collect_loose_footnotes(
    html: &Html,
    main_content: NodeId,
    footnotes: &mut BTreeMap<usize, FootnoteData>,
    containers: &mut Vec<NodeId>,
) {
    let result = find_loose_footnote_paragraphs(html, main_content);
    let Some((paragraphs, to_remove)) = result else {
        return;
    };

    let mut count = footnotes.len() + 1;
    for (i, (num, def_id)) in paragraphs.iter().enumerate() {
        let content_html = strip_marker_and_wrap(html, *def_id);

        // Collect continuation elements (between this def and next)
        let next_def = paragraphs.get(i + 1).map(|(_, id)| *id);
        let mut continuation = String::new();
        let siblings = collect_next_siblings_until(html, *def_id, next_def);
        for sib_id in &siblings {
            continuation.push_str(&dom::outer_html(html, *sib_id));
        }

        let full_content = format!("{content_html}{continuation}");

        footnotes.insert(
            count,
            FootnoteData {
                content_html: strip_backrefs_from_html(&full_content),
                original_id: num.to_string(),
            },
        );
        count += 1;
    }

    containers.extend(to_remove);
}

/// Numbered footnote paragraphs and the nodes to remove.
type LooseFootnotes = (Vec<(usize, NodeId)>, Vec<NodeId>);

/// Find loose footnote paragraphs: numbered `<p>` elements after
/// an `<hr>` or at the trailing end of the content.
fn find_loose_footnote_paragraphs(html: &Html, main_content: NodeId) -> Option<LooseFootnotes> {
    // Use parent of last <p> as scan container for nested layouts
    let all_ps = dom::descendant_elements_by_tag(html, main_content, "p");
    let container = if let Some(&last_p) = all_ps.last() {
        dom::parent_element(html, last_p).unwrap_or(main_content)
    } else {
        main_content
    };
    let children = dom::child_elements(html, container);

    // Strategy 1: forward-scan after the last <hr>
    for i in (0..children.len()).rev() {
        if dom::tag_name(html, children[i]).as_deref() != Some("hr") {
            continue;
        }

        let mut paragraphs = Vec::new();
        for &child in &children[(i + 1)..] {
            if let Some(num) = parse_footnote_num(html, child) {
                paragraphs.push((num, child));
            }
        }

        if paragraphs.len() >= 2 && cross_validate(html, main_content, &paragraphs) {
            let to_remove: Vec<NodeId> = children[i..].to_vec();
            return Some((paragraphs, to_remove));
        }
        break;
    }

    // Strategy 2: backward-scan for trailing numbered paragraphs
    let mut trailing: Vec<(usize, NodeId)> = Vec::new();
    let mut first_idx = children.len();
    for i in (0..children.len()).rev() {
        let tag = dom::tag_name(html, children[i]).unwrap_or_default();
        if tag == "p" {
            if let Some(num) = parse_footnote_num(html, children[i]) {
                trailing.push((num, children[i]));
                first_idx = i;
                continue;
            }
            break;
        }
        if tag == "ul" || tag == "ol" || tag == "blockquote" {
            continue;
        }
        break;
    }
    trailing.reverse();

    if trailing.len() >= 2 && cross_validate(html, main_content, &trailing) {
        let mut to_remove: Vec<NodeId> = children[first_idx..].to_vec();

        // Remove preceding footnote heading
        if let Some(&(_, first_el)) = trailing.first() {
            remove_preceding_footnote_heading(html, first_el, &mut to_remove);
        }

        return Some((trailing, to_remove));
    }

    None
}

/// Parse the footnote number from the first child element of a
/// paragraph (either `<sup>N</sup>` or `<strong>N</strong>`).
fn parse_footnote_num(html: &Html, el_id: NodeId) -> Option<usize> {
    let children = dom::child_elements(html, el_id);
    let first = children.first()?;
    let tag = dom::tag_name(html, *first)?;
    if tag != "sup" && tag != "strong" {
        return None;
    }
    let text = dom::text_content(html, *first).trim().to_string();
    let num: usize = text.parse().ok()?;
    if num < 1 || text != num.to_string() {
        return None;
    }
    Some(num)
}

/// Cross-validate: do inline `<sup>` elements reference the
/// numbered definitions?
fn cross_validate(html: &Html, main_content: NodeId, paragraphs: &[(usize, NodeId)]) -> bool {
    let numbered: HashSet<usize> = paragraphs.iter().map(|(n, _)| *n).collect();
    let mut matched = HashSet::new();

    let sups = dom::descendant_elements_by_tag(html, main_content, "sup");
    for sup_id in sups {
        // Skip sups that are inside our footnote paragraphs
        if paragraphs
            .iter()
            .any(|(_, pid)| dom::is_ancestor(html, sup_id, *pid))
        {
            continue;
        }
        // Skip sups containing links (already structured footnotes)
        if !dom::descendant_elements_by_tag(html, sup_id, "a").is_empty() {
            continue;
        }
        let text = dom::text_content(html, sup_id).trim().to_string();
        if let Ok(n) = text.parse::<usize>()
            && n >= 1
            && text == n.to_string()
            && numbered.contains(&n)
        {
            matched.insert(n);
        }
    }

    matched.len() >= 2
}

/// Strip the marker (first child element) from a footnote paragraph
/// and return the remaining content as HTML.
fn strip_marker_and_wrap(html: &Html, el_id: NodeId) -> String {
    let content = dom::inner_html(html, el_id);
    // Remove the first element (sup or strong marker)
    let marker_re =
        Regex::new(r"^\s*<(sup|strong)>[^<]*</(sup|strong)>\s*").expect("marker regex is valid");
    marker_re.replace(&content, "").to_string()
}

/// Collect next sibling elements until reaching `stop_at` or the
/// end of children.
fn collect_next_siblings_until(
    html: &Html,
    node_id: NodeId,
    stop_at: Option<NodeId>,
) -> Vec<NodeId> {
    let mut result = Vec::new();
    let Some(node_ref) = html.tree.get(node_id) else {
        return result;
    };
    let mut current = node_ref.next_sibling();
    while let Some(sib) = current {
        if matches!(sib.value(), Node::Element(_)) {
            if stop_at.is_some_and(|s| sib.id() == s) {
                break;
            }
            // Also stop if this sibling has an id in the fragment set
            result.push(sib.id());
        }
        current = sib.next_sibling();
    }
    result
}

// ── Class-based footnote paragraphs (p.footnote) ───────────────

fn collect_class_footnotes(
    html: &Html,
    main_content: NodeId,
    footnotes: &mut BTreeMap<usize, FootnoteData>,
    containers: &mut Vec<NodeId>,
) {
    let paras = dom::select_within(html, main_content, "p.footnote");

    let mut count = footnotes.len() + 1;
    for p_id in paras {
        let Some(num) = parse_footnote_num(html, p_id) else {
            continue;
        };

        let content_html = strip_marker_and_wrap(html, p_id);

        footnotes.insert(
            count,
            FootnoteData {
                content_html: strip_backrefs_from_html(&content_html),
                original_id: num.to_string(),
            },
        );
        count += 1;
        containers.push(p_id);
    }
}

// ── Inline reference standardization ────────────────────────────

/// Replace inline references with canonical `<sup>` elements.
fn standardize_inline_refs(
    html: &mut Html,
    main_content: NodeId,
    footnotes: &BTreeMap<usize, FootnoteData>,
) {
    // Build lookup from original_id -> (footnote_number, data)
    let by_original: HashMap<String, usize> = footnotes
        .iter()
        .map(|(&num, data)| (data.original_id.to_lowercase(), num))
        .collect();

    // Also build by sequential number for loose footnotes
    let by_num: HashMap<String, usize> = footnotes.keys().map(|&n| (n.to_string(), n)).collect();

    // Track which footnotes get refs assigned
    let mut ref_counts: HashMap<usize, usize> = HashMap::new();

    // Process selector-matched inline references
    for selector_str in FOOTNOTE_INLINE_REFERENCES {
        let refs = dom::select_within(html, main_content, selector_str);
        for ref_id in refs {
            // Skip already-canonicalized elements
            if is_already_canonical(html, ref_id) {
                continue;
            }
            let text = dom::text_content(html, ref_id).trim().to_string();
            if text.is_empty() {
                continue;
            }

            let fn_num = resolve_inline_ref(html, ref_id, &by_original, &by_num);
            let Some(num) = fn_num else { continue };

            let count = ref_counts.entry(num).or_insert(0);
            *count += 1;
            let ref_name = make_ref_id(num, *count - 1);

            let sup = create_footnote_ref(html, num, &ref_name);
            let outer = find_outer_footnote_container(html, ref_id);
            replace_node(html, outer, sup);
        }
    }

    // Google Docs inline refs
    let gdoc_refs = dom::select_within(html, main_content, "sup[id^=\"ftnt_ref\"]");
    for ref_id in gdoc_refs {
        let id_attr = dom::get_attr(html, ref_id, "id").unwrap_or_default();
        let fragment = id_attr
            .strip_prefix("ftnt_ref")
            .unwrap_or("")
            .to_lowercase();
        let original = format!("ftnt{fragment}");
        let Some(&num) = by_original.get(&original) else {
            continue;
        };

        let count = ref_counts.entry(num).or_insert(0);
        *count += 1;
        let ref_name = make_ref_id(num, *count - 1);
        let sup = create_footnote_ref(html, num, &ref_name);
        replace_node(html, ref_id, sup);
    }

    // Fallback: match remaining unmatched footnotes by fragment links
    assign_unmatched_refs_by_link(html, main_content, footnotes, &mut ref_counts);

    // Fallback: match by numeric sup/span text
    assign_unmatched_refs_by_text(html, main_content, footnotes, &mut ref_counts);
}

/// Resolve which footnote number an inline reference corresponds to.
fn resolve_inline_ref(
    html: &Html,
    ref_id: NodeId,
    by_original: &HashMap<String, usize>,
    by_num: &HashMap<String, usize>,
) -> Option<usize> {
    // Try href fragment
    let href = dom::get_attr(html, ref_id, "href").or_else(|| {
        // Look for child anchor
        let anchors = dom::descendant_elements_by_tag(html, ref_id, "a");
        anchors
            .first()
            .and_then(|a| dom::get_attr(html, *a, "href"))
    });

    if let Some(href) = &href {
        let fragment = href.split('#').next_back().unwrap_or("").to_lowercase();
        if !fragment.is_empty() {
            // Try direct match
            if let Some(&num) = by_original.get(&fragment) {
                return Some(num);
            }
            // Try cite_note pattern
            if let Some(suffix) = fragment.split("cite_note-").nth(1)
                && let Some(&num) = by_original.get(suffix)
            {
                return Some(num);
            }
            // Try fn: prefix
            if let Some(suffix) = fragment.strip_prefix("fn:")
                && let Some(&num) = by_original.get(suffix)
            {
                return Some(num);
            }
            if let Some(suffix) = fragment.strip_prefix("fn")
                && let Some(&num) = by_original.get(suffix)
            {
                return Some(num);
            }
            // Try _ftn prefix (Word)
            if let Some(suffix) = fragment.strip_prefix("_ftn") {
                let original = format!("_ftn{suffix}");
                if let Some(&num) = by_original.get(&original) {
                    return Some(num);
                }
            }
        }
    }

    // Try element ID
    let el_id_attr = dom::get_attr(html, ref_id, "id").unwrap_or_default();
    if el_id_attr.starts_with("fnref:") {
        let suffix = el_id_attr
            .strip_prefix("fnref:")
            .unwrap_or("")
            .to_lowercase();
        if let Some(&num) = by_original.get(&suffix) {
            return Some(num);
        }
    }
    if el_id_attr.starts_with("fnref") {
        let suffix = el_id_attr
            .strip_prefix("fnref")
            .unwrap_or("")
            .to_lowercase();
        if let Some(&num) = by_original.get(&suffix) {
            return Some(num);
        }
    }

    // Try text content as number
    let text = dom::text_content(html, ref_id)
        .trim()
        .trim_matches(|c| c == '[' || c == ']' || c == '(' || c == ')')
        .to_string();
    if let Some(&num) = by_num.get(&text) {
        return Some(num);
    }

    None
}

/// Assign refs to unmatched footnotes by looking for fragment links.
fn assign_unmatched_refs_by_link(
    html: &mut Html,
    main_content: NodeId,
    footnotes: &BTreeMap<usize, FootnoteData>,
    ref_counts: &mut HashMap<usize, usize>,
) {
    let unmatched: Vec<(usize, String)> = footnotes
        .iter()
        .filter(|(num, _)| !ref_counts.contains_key(num))
        .map(|(&num, data)| (num, data.original_id.clone()))
        .collect();

    if unmatched.is_empty() {
        return;
    }

    let id_map: HashMap<String, usize> = unmatched
        .iter()
        .map(|(num, id)| (id.to_lowercase(), *num))
        .collect();

    let all_links = dom::select_within(html, main_content, "a[href*=\"#\"]");
    let num_re = Regex::new(r"^[\[\(]?\d{1,4}[\]\)]?$").expect("num regex is valid");

    for link_id in all_links {
        let href = dom::get_attr(html, link_id, "href").unwrap_or_default();
        let fragment = href.split('#').next_back().unwrap_or("").to_lowercase();
        let Some(&num) = id_map.get(&fragment) else {
            continue;
        };
        let text = dom::text_content(html, link_id).trim().to_string();
        if !num_re.is_match(&text) {
            continue;
        }

        // Skip if inside footnotes section
        if is_inside_footnotes(html, link_id) {
            continue;
        }

        let count = ref_counts.entry(num).or_insert(0);
        *count += 1;
        let ref_name = make_ref_id(num, *count - 1);
        let sup = create_footnote_ref(html, num, &ref_name);
        let outer = find_outer_footnote_container(html, link_id);
        replace_node(html, outer, sup);
    }
}

/// Assign refs to unmatched footnotes by matching numeric sup/span
/// text content.
fn assign_unmatched_refs_by_text(
    html: &mut Html,
    main_content: NodeId,
    footnotes: &BTreeMap<usize, FootnoteData>,
    ref_counts: &mut HashMap<usize, usize>,
) {
    let unmatched: Vec<usize> = footnotes
        .keys()
        .filter(|num| !ref_counts.contains_key(num))
        .copied()
        .collect();

    if unmatched.is_empty() {
        return;
    }

    let num_set: HashSet<usize> = unmatched.into_iter().collect();
    let num_re = Regex::new(r"^[\[\(]?(\d{1,4})[\]\)]?$").expect("num regex is valid");

    let sups = dom::select_within(html, main_content, "sup, span.footnote-ref");
    for sup_id in sups {
        // Skip already-standardized elements
        if dom::get_attr(html, sup_id, "id").is_some_and(|id| id.starts_with("fnref:")) {
            continue;
        }
        if is_inside_footnotes(html, sup_id) {
            continue;
        }

        let text = dom::text_content(html, sup_id).trim().to_string();
        let Some(caps) = num_re.captures(&text) else {
            continue;
        };
        let Ok(n) = caps[1].parse::<usize>() else {
            continue;
        };
        if !num_set.contains(&n) {
            continue;
        }

        // Check not already assigned
        if ref_counts.contains_key(&n) {
            // Allow multiple refs
        }

        let count = ref_counts.entry(n).or_insert(0);
        *count += 1;
        let ref_name = make_ref_id(n, *count - 1);
        let sup = create_footnote_ref(html, n, &ref_name);
        let outer = find_outer_footnote_container(html, sup_id);
        replace_node(html, outer, sup);
    }
}

/// Check if a node or its parent is already a canonical footnote ref.
fn is_already_canonical(html: &Html, node_id: NodeId) -> bool {
    // Check the node itself
    if dom::get_attr(html, node_id, "id").is_some_and(|id| id.starts_with("fnref:")) {
        return true;
    }
    // Check parent (canonical sup wraps an a)
    if let Some(parent) = dom::parent_element(html, node_id)
        && dom::get_attr(html, parent, "id").is_some_and(|id| id.starts_with("fnref:"))
    {
        return true;
    }
    false
}

/// Check if a node is inside a footnotes section.
fn is_inside_footnotes(html: &Html, node_id: NodeId) -> bool {
    let mut current = node_id;
    loop {
        let Some(parent) = dom::parent_element(html, current) else {
            return false;
        };
        let id = dom::get_attr(html, parent, "id").unwrap_or_default();
        if id.starts_with("fnref:") || id == "footnotes" {
            return true;
        }
        let cls = dom::get_attr(html, parent, "class").unwrap_or_default();
        if cls.contains("footnote") || cls.contains("reflist") || cls.contains("references") {
            return true;
        }
        current = parent;
    }
}

// ── DOM manipulation helpers ────────────────────────────────────

/// Create a canonical footnote reference element:
/// `<sup id="fnref:N"><a href="#fn:N">N</a></sup>`
fn create_footnote_ref(html: &mut Html, footnote_num: usize, ref_id: &str) -> NodeId {
    let sup_el = create_element("sup", &[("id", ref_id)]);
    let sup_id = html.tree.orphan(Node::Element(sup_el)).id();

    let link_el = create_element("a", &[("href", &format!("#fn:{footnote_num}"))]);
    let link_id = html.tree.orphan(Node::Element(link_el)).id();

    let text = Node::Text(scraper::node::Text {
        text: footnote_num.to_string().into(),
    });
    let text_id = html.tree.orphan(text).id();

    if let Some(mut link_mut) = html.tree.get_mut(link_id) {
        link_mut.append_id(text_id);
    }
    if let Some(mut sup_mut) = html.tree.get_mut(sup_id) {
        sup_mut.append_id(link_id);
    }

    sup_id
}

/// Replace a node with another node (insert before, then detach).
fn replace_node(html: &mut Html, old_id: NodeId, new_id: NodeId) {
    let Some(mut old_mut) = html.tree.get_mut(old_id) else {
        return;
    };
    old_mut.insert_id_before(new_id);
    old_mut.detach();
}

/// Create an element node with given tag and attributes.
fn create_element(tag: &str, attrs: &[(&str, &str)]) -> scraper::node::Element {
    let name = QualName::new(None, ns!(html), markup5ever::LocalName::from(tag));
    let attributes: Vec<Attribute> = attrs
        .iter()
        .map(|(k, v)| Attribute {
            name: QualName::new(None, ns!(), markup5ever::LocalName::from(*k)),
            value: (*v).into(),
        })
        .collect();

    scraper::node::Element::new(name, attributes)
}

/// Build the `make_ref_id` value: `fnref:N` for first ref,
/// `fnref:N-M` for subsequent refs.
fn make_ref_id(footnote_num: usize, ref_index: usize) -> String {
    if ref_index == 0 {
        format!("fnref:{footnote_num}")
    } else {
        format!("fnref:{footnote_num}-{}", ref_index + 1)
    }
}

/// Find the outermost footnote container (walk up through sup/span
/// wrappers).
fn find_outer_footnote_container(html: &Html, el_id: NodeId) -> NodeId {
    let mut current = el_id;
    loop {
        let Some(parent) = dom::parent_element(html, current) else {
            return current;
        };
        let tag = dom::tag_name(html, parent).unwrap_or_default();
        if tag != "span" && tag != "sup" {
            return current;
        }

        // Don't walk into spans with substantial non-footnote content
        if tag == "span" && has_non_footnote_content(html, parent, current) {
            return current;
        }

        current = parent;
    }
}

/// Check if a parent has non-footnote children besides the current
/// element.
fn has_non_footnote_content(html: &Html, parent: NodeId, current: NodeId) -> bool {
    let Some(parent_ref) = html.tree.get(parent) else {
        return false;
    };
    for child in parent_ref.children() {
        if child.id() == current {
            continue;
        }
        match child.value() {
            Node::Text(t) if !t.text.trim().is_empty() => return true,
            Node::Element(el) => {
                let tag = el.name.local.as_ref();
                if tag != "sup" {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

// ── Canonical list building ─────────────────────────────────────

/// Build the canonical `<div id="footnotes"><ol>...</ol></div>` and
/// append it to `main_content`.
fn build_canonical_list(
    html: &mut Html,
    main_content: NodeId,
    footnotes: &BTreeMap<usize, FootnoteData>,
) {
    if footnotes.is_empty() {
        return;
    }

    // Build div#footnotes containing div.footnote items
    let container_el = create_element("div", &[("id", "footnotes")]);
    let container_id = html.tree.orphan(Node::Element(container_el)).id();

    for (&num, data) in footnotes {
        let item_id = build_footnote_item(html, num, data);
        if let Some(mut container_mut) = html.tree.get_mut(container_id) {
            container_mut.append_id(item_id);
        }
    }

    if let Some(mut main_mut) = html.tree.get_mut(main_content) {
        main_mut.append_id(container_id);
    }
}

/// Build a single `<div id="fn:N" class="footnote">` element with
/// content and backref link.
fn build_footnote_item(html: &mut Html, num: usize, data: &FootnoteData) -> NodeId {
    let item_el = create_element(
        "div",
        &[("id", &format!("fn:{num}")), ("class", "footnote")],
    );
    let li_id = html.tree.orphan(Node::Element(item_el)).id();

    // Parse the content HTML into a tree fragment and transfer nodes
    let content = &data.content_html;
    if !content.trim().is_empty() {
        // Wrap in a temporary container for parsing
        let wrapper = format!("<div>{content}</div>");
        let parsed = Html::parse_fragment(&wrapper);
        transfer_parsed_content(html, li_id, &parsed);
    }

    li_id
}

/// Transfer parsed content from a fragment into a target node.
fn transfer_parsed_content(target_html: &mut Html, target_id: NodeId, source: &Html) {
    // Html::parse_fragment wraps in html > body > div (our wrapper).
    // We want the children of the wrapper div.
    let body_ids = dom::select_ids(source, "div");
    // The first div in the fragment is our wrapper
    let wrapper_id = body_ids.into_iter().next();
    let Some(wrapper) = wrapper_id else { return };

    let Some(wrapper_ref) = source.tree.get(wrapper) else {
        return;
    };

    for child in wrapper_ref.children() {
        clone_node_into(target_html, target_id, source, child.id());
    }
}

/// Recursively clone a node from `source` tree into `target` tree
/// as a child of `parent_id`.
fn clone_node_into(target: &mut Html, parent_id: NodeId, source: &Html, source_id: NodeId) {
    let Some(node_ref) = source.tree.get(source_id) else {
        return;
    };

    match node_ref.value() {
        Node::Text(t) => {
            let new_text = Node::Text(scraper::node::Text {
                text: t.text.clone(),
            });
            let new_id = target.tree.orphan(new_text).id();
            if let Some(mut parent) = target.tree.get_mut(parent_id) {
                parent.append_id(new_id);
            }
        }
        Node::Element(el) => {
            let new_el = scraper::node::Element::new(
                el.name.clone(),
                el.attrs()
                    .map(|(k, v)| Attribute {
                        name: QualName::new(None, ns!(), markup5ever::LocalName::from(k)),
                        value: v.into(),
                    })
                    .collect(),
            );
            let new_id = target.tree.orphan(Node::Element(new_el)).id();
            if let Some(mut parent) = target.tree.get_mut(parent_id) {
                parent.append_id(new_id);
            }
            // Recurse for children
            for child in node_ref.children() {
                clone_node_into(target, new_id, source, child.id());
            }
        }
        Node::Comment(c) => {
            let new_comment = Node::Comment(scraper::node::Comment {
                comment: c.comment.clone(),
            });
            let new_id = target.tree.orphan(new_comment).id();
            if let Some(mut parent) = target.tree.get_mut(parent_id) {
                parent.append_id(new_id);
            }
        }
        _ => {}
    }
}

// ── Cleanup helpers ─────────────────────────────────────────────

/// Remove containers from the DOM.
fn remove_containers(html: &mut Html, containers: &[NodeId]) {
    for &id in containers {
        dom::remove_node(html, id);
    }
}

/// Remove orphaned `<hr>` elements at the end of main content
/// (left behind when footnote sections after a divider are removed).
fn remove_orphaned_hrs(html: &mut Html, main_content: NodeId) {
    // Find all <p> descendants to locate the nested container
    let all_ps = dom::descendant_elements_by_tag(html, main_content, "p");
    let container = if let Some(&last_p) = all_ps.last() {
        dom::parent_element(html, last_p).unwrap_or(main_content)
    } else {
        main_content
    };

    let children = dom::child_elements(html, container);
    // Remove trailing <hr> elements
    for &child in children.iter().rev() {
        if dom::tag_name(html, child).as_deref() == Some("hr") {
            dom::remove_node(html, child);
        } else {
            break;
        }
    }
}

/// Remove a "Footnotes"/"Notes"/etc. heading preceding a footnote
/// element.
fn remove_preceding_footnote_heading(html: &Html, first_el: NodeId, containers: &mut Vec<NodeId>) {
    // Check the previous sibling, or the parent's previous sibling
    let scan_from = if let Some(parent) = dom::parent_element(html, first_el) {
        if dom::tag_name(html, parent).as_deref() == Some("div")
            && dom::child_elements(html, parent).len() == 1
        {
            parent
        } else {
            first_el
        }
    } else {
        first_el
    };

    let Some(node_ref) = html.tree.get(scan_from) else {
        return;
    };
    let Some(prev) = node_ref.prev_sibling() else {
        return;
    };

    // Skip text nodes to find previous element
    let prev_el = if matches!(prev.value(), Node::Element(_)) {
        Some(prev.id())
    } else {
        // Walk backwards through siblings
        let mut current = prev;
        loop {
            let Some(p) = current.prev_sibling() else {
                break None;
            };
            if matches!(p.value(), Node::Element(_)) {
                break Some(p.id());
            }
            current = p;
        }
    };

    let Some(prev_el_id) = prev_el else { return };
    let tag = dom::tag_name(html, prev_el_id).unwrap_or_default();
    if !tag.starts_with('h') || tag.len() != 2 {
        return;
    }

    let text = dom::text_content(html, prev_el_id).trim().to_string();
    if footnote_section_re().is_match(&text) {
        containers.push(prev_el_id);
    }
}

/// Strip backref links (↩, ↥, ↑, etc.) from HTML content.
fn strip_backrefs_from_html(html_content: &str) -> String {
    use std::sync::LazyLock;

    // Remove backref anchors
    static BACKREF_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"<a\s[^>]*>[\s]*[\u{21A9}\u{21A5}\u{2191}\u{21B5}\u{2934}\u{2935}\u{23CE}\u{FE0E}\u{FE0F}↩︎]+[\s]*</a>",
        )
        .expect("backref regex is valid")
    });

    // Also remove backref anchors with class="footnote-backref" or
    // class="data-footnote-backref"
    static CLASS_BACKREF_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r#"<a\s[^>]*class="[^"]*(?:footnote-backref|data-footnote-backref)[^"]*"[^>]*>[^<]*</a>"#,
        )
        .expect("class backref regex is valid")
    });

    // Remove leading [N] text from gdocs footnotes
    static LEADING_NUM_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^\s*(<a\s[^>]*>)?\s*\[?\d+\]?\s*(</a>)?\s*")
            .expect("leading num regex is valid")
    });

    // Word-style backrefs: <sup><a href="...#_ftnref...">...</a></sup>
    // or <sup><a href="...#_ftnref...">...</a> </sup>
    static WORD_BACKREF_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"<sup>\s*<a\s[^>]*href="[^"]*#_ftnref\d+"[^>]*>[^<]*</a>\s*</sup>"#)
            .expect("word backref regex is valid")
    });

    let result = BACKREF_RE.replace_all(html_content, "");
    let result = CLASS_BACKREF_RE.replace_all(&result, "");
    let result = WORD_BACKREF_RE.replace_all(&result, "");

    // Trim trailing whitespace/commas
    let result = result.trim_end_matches(|c: char| c.is_whitespace() || c == ',' || c == ';');

    // Strip leading numeric prefix if it's from a gdocs backref
    let result = if result.starts_with("<a") && result.contains("ftnt_ref") {
        LEADING_NUM_RE.replace(result, "").to_string()
    } else {
        result.to_string()
    };

    result.trim().to_string()
}

// ── Markdown conversion support ─────────────────────────────────

/// Check if this is a canonical footnote inline reference
/// (`<sup id="fnref:N"><a href="#fn:N">N</a></sup>`) and return the
/// footnote number if so.
#[must_use]
pub fn is_canonical_footnote_ref(attrs: &[(String, String)]) -> Option<String> {
    let id_attr = attrs.iter().find(|(k, _)| k == "id")?;
    let id = &id_attr.1;
    id.strip_prefix("fnref:").map(String::from)
}

/// Check if this is the canonical footnotes container
/// (`<div id="footnotes">`).
#[must_use]
pub fn is_canonical_footnotes_div(attrs: &[(String, String)]) -> bool {
    attrs.iter().any(|(k, v)| k == "id" && v == "footnotes")
}

/// Check if this is a canonical footnote list item
/// (`<li id="fn:N" class="footnote">`).
#[must_use]
pub fn is_canonical_footnote_item(attrs: &[(String, String)]) -> Option<String> {
    let id = attrs.iter().find(|(k, _)| k == "id")?;
    id.1.strip_prefix("fn:").map(String::from)
}

/// Check if this is a footnote backref link.
#[must_use]
pub fn is_footnote_backref(attrs: &[(String, String)]) -> bool {
    attrs
        .iter()
        .any(|(k, v)| k == "class" && v.contains("footnote-backref"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_standardize(input: &str) -> Html {
        let mut html = Html::parse_document(input);
        let article = dom::select_ids(&html, "article");
        let main_id = if article.is_empty() {
            dom::select_ids(&html, "body")[0]
        } else {
            article[0]
        };
        standardize_footnotes(&mut html, main_id);
        html
    }

    fn has_canonical_ref(html: &Html, num: usize) -> bool {
        let sel = format!("sup[id=\"fnref:{num}\"]");
        !dom::select_ids(html, &sel).is_empty()
    }

    fn has_canonical_item(html: &Html, num: usize) -> bool {
        let sel = format!("div[id=\"fn:{num}\"]");
        !dom::select_ids(html, &sel).is_empty()
    }

    fn has_footnotes_div(html: &Html) -> bool {
        !dom::select_ids(html, "div#footnotes").is_empty()
    }

    #[test]
    fn standardizes_basic_footnote_list() {
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

        let html = run_standardize(input);
        assert!(has_footnotes_div(&html), "should have #footnotes div");
        assert!(has_canonical_item(&html, 1), "should have fn:1 item");
    }

    #[test]
    fn standardizes_inline_reference() {
        let input = r##"<html><body>
            <article>
                <p>Text <sup class="reference"><a href="#cite_note-1">[1]</a></sup>.</p>
                <ol class="references">
                    <li id="cite_note-1">A citation.</li>
                </ol>
            </article>
        </body></html>"##;

        let html = run_standardize(input);
        assert!(has_canonical_ref(&html, 1), "should have fnref:1");
        assert!(has_canonical_item(&html, 1), "should have fn:1");
    }

    #[test]
    fn standardizes_wp_block_footnotes() {
        let input = r##"<html><body><article>
            <p>Text<sup data-fn="uuid1" class="fn">
                <a href="#uuid1" id="uuid1-link">1</a></sup></p>
            <ol class="wp-block-footnotes">
                <li id="uuid1">Note text.
                    <a href="#uuid1-link">↩︎</a></li>
            </ol>
        </article></body></html>"##;

        let html = run_standardize(input);
        assert!(has_footnotes_div(&html));
        assert!(has_canonical_item(&html, 1));
        // Backref should be stripped from content
        let li_ids = dom::select_ids(&html, "div#fn\\:1");
        assert!(!li_ids.is_empty());
        let content = dom::inner_html(&html, li_ids[0]);
        assert!(
            !content.contains('\u{21A9}') || content.contains("footnote-backref"),
            "original backref should be replaced by canonical one"
        );
    }

    #[test]
    fn preserves_equation_refs() {
        let input = r##"<html><body><article>
            <p>See <a href="#thm-1">1</a> and <a href="#thm-2">2</a>.</p>
            <h3><a name="thm-1"></a>Theorem 1</h3>
            <p>Content.</p>
            <h3><a name="thm-2"></a>Theorem 2</h3>
            <p>Content.</p>
        </article></body></html>"##;

        let html = run_standardize(input);
        // No footnotes div should be created
        assert!(
            !has_footnotes_div(&html),
            "equation refs should not become footnotes"
        );
    }
}
