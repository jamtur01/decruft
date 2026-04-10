use ego_tree::NodeId;
use scraper::{Html, Selector};

use crate::dom;
use crate::scorer;

/// Entry-point selectors in priority order, from most specific
/// content containers to the generic `body` fallback.
const ENTRY_POINT_SELECTORS: &[&str] = &[
    "#post, .post-content, .post-body",
    ".article-content, #article-content",
    ".article_post, .article-wrapper",
    ".entry-content, .content-article",
    ".instapaper_body",
    ".post",
    ".markdown-body",
    ".markdown-preview-sizer",
    "article, [role=\"article\"]",
    "main, [role=\"main\"]",
    ".article-body",
    "#content",
    "body",
];

const FALLBACK_SELECTOR: &str = "div, section, article, main";
const MIN_WORDS_FOR_CHILD_PREFERENCE: usize = 50;

/// Find the body element's `NodeId`.
#[must_use]
pub fn find_body(html: &Html) -> Option<NodeId> {
    let Ok(sel) = Selector::parse("body") else {
        return None;
    };
    html.select(&sel).next().map(|el| el.id())
}

/// Find the main content element in the document.
/// Returns the `NodeId` of the best content container.
#[must_use]
pub fn find_main_content(html: &Html) -> NodeId {
    let total_selectors = ENTRY_POINT_SELECTORS.len();
    let (candidates, only_body_matched) = collect_candidates(html, total_selectors);

    let best = if only_body_matched {
        pick_fallback(html).or_else(|| extract_body(&candidates, html))
    } else {
        pick_best(html, &candidates)
    };

    best.or_else(|| find_body(html))
        .unwrap_or_else(|| html.tree.root().id())
}

/// Candidate with its originating selector index and score.
struct Candidate {
    node_id: NodeId,
    score: f64,
    selector_index: usize,
}

/// Score and collect all candidates from entry-point selectors.
/// Returns the sorted list and whether only `body` produced matches.
fn collect_candidates(html: &Html, total_selectors: usize) -> (Vec<Candidate>, bool) {
    let mut candidates = Vec::new();
    let mut non_body_matched = false;
    let body_index = total_selectors - 1;

    for (idx, selector_str) in ENTRY_POINT_SELECTORS.iter().enumerate() {
        let Ok(sel) = Selector::parse(selector_str) else {
            continue;
        };
        for el_ref in html.select(&sel) {
            let node_id = el_ref.id();
            #[allow(clippy::cast_precision_loss)]
            let priority_bonus = (total_selectors - idx) as f64 * 40.0;
            let element_score = scorer::score_element(html, node_id);
            let score = priority_bonus + element_score;

            if idx != body_index {
                non_body_matched = true;
            }
            candidates.push(Candidate {
                node_id,
                score,
                selector_index: idx,
            });
        }
    }

    candidates.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    (candidates, !non_body_matched)
}

/// Among top candidates, check if the best contains a child from a
/// higher-priority selector with enough content. If so, prefer the
/// more specific child.
fn pick_best(html: &Html, candidates: &[Candidate]) -> Option<NodeId> {
    let best = candidates.first()?;
    let best_id = best.node_id;
    let best_selector_idx = best.selector_index;

    for candidate in candidates.iter().skip(1) {
        // Only consider children matched by a higher-priority selector.
        if candidate.selector_index >= best_selector_idx {
            continue;
        }
        if !dom::is_ancestor(html, candidate.node_id, best_id) {
            continue;
        }
        let text = dom::text_content(html, candidate.node_id);
        if dom::count_words(&text) > MIN_WORDS_FOR_CHILD_PREFERENCE {
            return Some(candidate.node_id);
        }
    }

    Some(best_id)
}

/// Fallback: score all div/section/article/main elements and pick
/// the highest-scoring one.
fn pick_fallback(html: &Html) -> Option<NodeId> {
    let Ok(sel) = Selector::parse(FALLBACK_SELECTOR) else {
        return None;
    };

    let mut best_id: Option<NodeId> = None;
    let mut best_score = f64::NEG_INFINITY;

    for el_ref in html.select(&sel) {
        let node_id = el_ref.id();
        let score = scorer::score_element(html, node_id);
        if score > best_score {
            best_score = score;
            best_id = Some(node_id);
        }
    }

    best_id
}

/// Extract the body `NodeId` from the candidate list.
fn extract_body(candidates: &[Candidate], html: &Html) -> Option<NodeId> {
    for c in candidates {
        if dom::is_tag(html, c.node_id, "body") {
            return Some(c.node_id);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_body_returns_body() {
        let html = Html::parse_document("<html><body><p>hello</p></body></html>");
        let body = find_body(&html);
        assert!(body.is_some());
    }

    #[test]
    fn entry_point_selectors_not_empty() {
        assert!(!ENTRY_POINT_SELECTORS.is_empty());
    }

    #[test]
    fn last_entry_point_is_body() {
        let last = ENTRY_POINT_SELECTORS.last().copied().unwrap_or_default();
        assert_eq!(last, "body");
    }
}
