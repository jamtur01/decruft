use std::time::Instant;

use scraper::Html;

use crate::cleanup;
use crate::content;
use crate::dom;
use crate::metadata;
use crate::patterns;
use crate::schema_org;
use crate::standardize;
use crate::types::{
    DebugInfo, DecruftOptions, DecruftResult, Removal,
};

/// Parse HTML and extract clean, readable content.
#[must_use]
pub fn parse(html_str: &str, options: &DecruftOptions) -> DecruftResult {
    let start = Instant::now();
    let mut html = Html::parse_document(html_str);
    let mut removals: Vec<Removal> = Vec::new();

    let schema_data = schema_org::extract_schema_org(&html);
    let meta_tags = cleanup::collect_meta_tags(&html);
    let meta = metadata::extract_metadata(
        &html,
        options.url.as_deref(),
        schema_data.as_ref(),
    );

    let main_content = resolve_content_root(&html, options);
    let content_selector_path = dom::selector_path(&html, main_content);

    run_cleanup_pipeline(
        &mut html,
        main_content,
        &mut removals,
        options,
    );

    let content_html = dom::outer_html(&html, main_content);
    let word_count = dom::count_words_html(&content_html);
    let (final_content, final_word_count) =
        if word_count < 50 && !has_relaxed_options(options) {
            retry_with_relaxed_options(html_str, options)
                .unwrap_or((content_html, word_count))
        } else {
            (content_html, word_count)
        };

    let elapsed = start.elapsed();

    #[allow(clippy::cast_possible_truncation)]
    let parse_time_ms = elapsed.as_millis() as u64;

    build_result(
        final_content,
        final_word_count,
        parse_time_ms,
        meta,
        schema_data,
        meta_tags,
        content_selector_path,
        removals,
        options.debug,
    )
}

fn resolve_content_root(
    html: &Html,
    options: &DecruftOptions,
) -> ego_tree::NodeId {
    if let Some(ref sel) = options.content_selector {
        dom::select_ids(html, sel)
            .into_iter()
            .next()
            .unwrap_or_else(|| find_main(html))
    } else {
        find_main(html)
    }
}

fn run_cleanup_pipeline(
    html: &mut Html,
    main_content: ego_tree::NodeId,
    removals: &mut Vec<Removal>,
    options: &DecruftOptions,
) {
    if options.remove_images {
        cleanup::remove_all_images(html, main_content);
    }
    if options.remove_small_images {
        cleanup::remove_small_images(html, main_content);
    }
    if options.remove_hidden_elements {
        cleanup::remove_hidden_elements(
            html, main_content, removals, options.debug,
        );
    }
    if options.remove_exact_selectors {
        cleanup::remove_exact_selectors(
            html, main_content, removals, options.debug,
        );
    }
    if options.remove_partial_selectors {
        cleanup::remove_partial_selectors(
            html, main_content, removals, options.debug,
        );
    }
    if options.remove_low_scoring {
        cleanup::score_and_remove(
            html, main_content, removals, options.debug,
        );
    }
    if options.remove_content_patterns {
        patterns::remove_content_patterns(
            html, main_content, removals, options.debug,
        );
    }
    if options.standardize {
        standardize::standardize_content(
            html, main_content, options.debug,
        );
    }
    if let Some(ref url) = options.url {
        standardize::resolve_urls(html, main_content, url);
    }
    cleanup::deduplicate_images(html, main_content);
}

#[allow(clippy::too_many_arguments)]
fn build_result(
    content: String,
    word_count: usize,
    parse_time_ms: u64,
    meta: crate::types::Metadata,
    schema_data: Option<serde_json::Value>,
    meta_tags: Vec<crate::types::MetaTag>,
    content_selector_path: String,
    removals: Vec<Removal>,
    debug: bool,
) -> DecruftResult {
    DecruftResult {
        content,
        title: meta.title,
        description: meta.description,
        domain: meta.domain,
        favicon: meta.favicon,
        image: meta.image,
        language: meta.language,
        parse_time_ms,
        published: meta.published,
        author: meta.author,
        site: meta.site_name,
        word_count,
        schema_org_data: schema_data,
        meta_tags: if debug { Some(meta_tags) } else { None },
        debug: if debug {
            Some(DebugInfo {
                content_selector: content_selector_path,
                removals,
            })
        } else {
            None
        },
    }
}

fn find_main(html: &Html) -> ego_tree::NodeId {
    content::find_main_content(html)
}

fn has_relaxed_options(options: &DecruftOptions) -> bool {
    !options.remove_partial_selectors
        && !options.remove_hidden_elements
        && !options.remove_low_scoring
}

fn retry_with_relaxed_options(
    html_str: &str,
    options: &DecruftOptions,
) -> Option<(String, usize)> {
    let mut relaxed = options.clone();
    relaxed.remove_partial_selectors = false;
    relaxed.remove_hidden_elements = false;
    relaxed.remove_low_scoring = false;
    relaxed.remove_content_patterns = false;

    let mut html = Html::parse_document(html_str);
    let main_content =
        if let Some(ref sel) = relaxed.content_selector {
            dom::select_ids(&html, sel)
                .into_iter()
                .next()
                .unwrap_or_else(|| find_main(&html))
        } else {
            find_main(&html)
        };

    if relaxed.remove_exact_selectors {
        cleanup::remove_exact_selectors(
            &mut html,
            main_content,
            &mut Vec::new(),
            false,
        );
    }

    if relaxed.standardize {
        standardize::standardize_content(&mut html, main_content, false);
    }

    let content = dom::outer_html(&html, main_content);
    let wc = dom::count_words_html(&content);
    if wc > 50 {
        Some((content, wc))
    } else {
        None
    }
}
