use scraper::{Html, Selector};

use crate::dom;
use crate::types::Metadata;

/// Extract metadata from an HTML document using priority-based
/// fallback chains. Schema.org data, when present, participates
/// in several chains.
#[must_use]
pub fn extract_metadata(
    html: &Html,
    url: Option<&str>,
    schema: Option<&serde_json::Value>,
) -> Metadata {
    let author = extract_author(html, schema);
    let site_name = extract_site_name(html, schema, &author);
    let domain = extract_domain(url);
    let title = extract_title(html, schema, &site_name, &domain);
    let published = extract_published(html, schema);
    let modified = extract_modified(html, schema);
    let description = extract_description(html, schema);
    let image = extract_image(html, schema);
    let language = extract_language(html, schema);
    let favicon = extract_favicon(html, url);
    let canonical_url = extract_canonical_url(html);
    let keywords = extract_keywords(html);
    let content_type = extract_content_type(html, schema);

    Metadata {
        title,
        description,
        domain,
        favicon,
        image,
        language,
        published,
        modified,
        author,
        site_name,
        canonical_url,
        keywords,
        content_type,
    }
}

/// Query `meta[{attr}="{value}"]` and return the `content` attribute.
fn get_meta_content(html: &Html, attr: &str, value: &str) -> Option<String> {
    let selector_str = format!("meta[{attr}=\"{value}\"]");
    let Ok(sel) = Selector::parse(&selector_str) else {
        return None;
    };
    let element = html.select(&sel).next()?;
    let content = element.value().attr("content")?;
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

/// Walk a dot-separated path on a single JSON object, returning a
/// trimmed, non-empty string.
/// Query Dublin Core meta tags. Checks all common naming variants:
/// `DC.{field}`, `dc.{field}`, `dc:{field}`, `dcterm:{field}`,
/// `DCTERMS.{field}`, `dcterms.{field}`.
fn get_dc_content(html: &Html, field: &str) -> Option<String> {
    let variants = [
        format!("DC.{field}"),
        format!("dc.{field}"),
        format!("dc:{field}"),
        format!("dcterm:{field}"),
        format!("DCTERMS.{field}"),
        format!("dcterms.{field}"),
    ];
    for name in &variants {
        if let Some(v) = get_meta_content(html, "name", name) {
            return Some(v);
        }
    }
    None
}

fn walk_schema_path(value: &serde_json::Value, path: &str) -> Option<String> {
    let mut current = value;
    for key in path.split('.') {
        current = current.get(key)?;
    }
    let text = current.as_str()?.trim();
    if text.is_empty() {
        return None;
    }
    Some(text.to_string())
}

/// Dig into a `serde_json::Value` by a dot-separated path.
/// Returns the leaf value as a trimmed, non-empty string.
/// Handles both single objects and arrays (tries each item).
fn schema_str(schema: Option<&serde_json::Value>, path: &str) -> Option<String> {
    let data = schema?;
    // Direct path on object
    if let Some(result) = walk_schema_path(data, path) {
        return Some(result);
    }
    // If array, try each item
    if let serde_json::Value::Array(items) = data {
        for item in items {
            if let Some(result) = walk_schema_path(item, path) {
                return Some(result);
            }
        }
    }
    None
}

// ------------------------------------------------------------------
// Title
// ------------------------------------------------------------------

fn extract_title(
    html: &Html,
    schema: Option<&serde_json::Value>,
    site_name: &str,
    domain: &str,
) -> String {
    let meta_title = get_meta_content(html, "property", "og:title")
        .or_else(|| get_meta_content(html, "name", "twitter:title"))
        .or_else(|| get_meta_content(html, "property", "twitter:title"))
        .or_else(|| schema_str(schema, "headline"))
        .or_else(|| get_dc_content(html, "title"))
        .or_else(|| get_meta_content(html, "name", "title"))
        .or_else(|| get_meta_content(html, "name", "sailthru.title"))
        .or_else(|| get_meta_content(html, "name", "parsely-title"));

    let html_title = title_element_text(html);

    let raw = meta_title.clone().or(html_title.clone());
    let Some(title) = raw else {
        return String::new();
    };
    // Use site_name or derive one from the domain
    let effective_site_name = if site_name.is_empty() {
        domain_to_site_name(domain)
    } else {
        site_name.to_string()
    };
    clean_title(
        &title,
        &effective_site_name,
        meta_title.as_deref(),
        html_title.as_deref(),
    )
}

fn title_element_text(html: &Html) -> Option<String> {
    let Ok(sel) = Selector::parse("title") else {
        return None;
    };
    let el = html.select(&sel).next()?;
    let text = dom::text_content(html, el.id());
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

const TITLE_SEPARATORS: &[&str] = &[" | ", " - ", " -- ", " / ", " · "];

/// Remove site-name suffixes/prefixes from a title.
///
/// When a site name is known, strips trailing/leading segments that
/// match it (exact or fuzzy). Uses `<title>` vs og:title comparison
/// to detect additional breadcrumb segments to strip.
///
/// When no site name is known, returns the title unchanged.
#[must_use]
pub fn clean_title(
    title: &str,
    site_name: &str,
    meta_title: Option<&str>,
    html_title: Option<&str>,
) -> String {
    // When we have both meta_title and html_title, use their
    // difference to detect breadcrumb segments to strip.
    if let Some(meta) = meta_title
        && let Some(html_t) = html_title
        && let Some(cleaned) = strip_via_html_comparison(meta, html_t, site_name)
    {
        return cleaned;
    }

    // Direct site_name stripping
    if !site_name.is_empty() {
        return strip_site_name(title, site_name);
    }

    title.to_string()
}

/// Compare `meta_title` (e.g. og:title) with `html_title` (from
/// `<title>` element) to detect site-name segments.
///
/// If `html_title` ends with a segment matching `site_name`, strip it,
/// then see if og:title also needs the next-to-last segment stripped
/// (common for breadcrumb paths like "owner/repo" on GitHub).
fn strip_via_html_comparison(
    meta_title: &str,
    html_title: &str,
    site_name: &str,
) -> Option<String> {
    if !site_name.is_empty() {
        // Strip site_name from html_title
        let html_cleaned = strip_site_name(html_title, site_name);

        // If meta_title matches html_cleaned exactly, both had the
        // same content minus site_name — strip one more breadcrumb
        // segment from meta_title.
        if meta_title == html_cleaned {
            let further = strip_last_breadcrumb_segment(meta_title);
            if further != meta_title {
                return Some(further);
            }
        }

        // meta_title is shorter/different from html — it's already
        // cleaner, just strip site_name from it too.
        let cleaned = strip_site_name(meta_title, site_name);
        return Some(cleaned);
    }

    // No site name: if html_title is longer than meta_title and
    // meta_title is a prefix of html_title (after trimming a
    // separator), the extra part is likely a site suffix.
    if html_title.len() > meta_title.len() && html_title.starts_with(meta_title) {
        return Some(meta_title.to_string());
    }

    None
}

/// Strip a segment matching `site_name` from the end (or start) of
/// the title. Matching is fuzzy: exact, case-insensitive, or
/// containment (the segment contains the site name or vice versa).
fn strip_site_name(title: &str, site_name: &str) -> String {
    let site_lower = site_name.to_lowercase();
    for sep in TITLE_SEPARATORS {
        let Some(idx) = title.rfind(sep) else {
            continue;
        };
        let before = title[..idx].trim();
        let after = title[idx + sep.len()..].trim();
        if before.is_empty() || after.is_empty() {
            continue;
        }
        let after_lower = after.to_lowercase();
        let before_lower = before.to_lowercase();

        let after_matches = is_site_name_match(&after_lower, &site_lower);
        let before_matches = is_site_name_match(&before_lower, &site_lower);

        // If both match, keep the longer one (it's the real title)
        if after_matches && before_matches {
            return if before.len() >= after.len() {
                before.to_string()
            } else {
                after.to_string()
            };
        }
        // Check trailing segment (most common: "Title - SiteName")
        if after_matches {
            return before.to_string();
        }
        // Check leading segment ("SiteName | Title")
        if before_matches {
            return after.to_string();
        }
    }
    title.to_string()
}

/// Check whether a title segment matches the site name.
/// Supports exact match, and containment (for cases like
/// "Wikipedia" matching "Wikimedia Foundation, Inc.").
fn is_site_name_match(segment: &str, site_name_lower: &str) -> bool {
    if segment == site_name_lower {
        return true;
    }
    // Check containment
    if segment.contains(site_name_lower) || site_name_lower.contains(segment) {
        return true;
    }
    // Segment is a single word matching the site name's first word
    // e.g., "wikipedia" matching "wikimedia" (close but different)
    // Only match if the segment is a SINGLE word (no spaces)
    let seg_first_word = segment.split_whitespace().next().unwrap_or("");
    let site_first_word = site_name_lower.split_whitespace().next().unwrap_or("");
    if segment.contains(' ') {
        // Multi-word segments should only match via containment (above)
        return false;
    }
    if !seg_first_word.is_empty() && !site_first_word.is_empty() && seg_first_word.len() >= 4 {
        let common_prefix_len = seg_first_word
            .chars()
            .zip(site_first_word.chars())
            .take_while(|(a, b)| a == b)
            .count();
        if common_prefix_len >= 5 {
            return true;
        }
    }
    false
}

/// Strip the last separator-delimited segment if it looks like a
/// breadcrumb (contains `/`, is a path, or is <=3 words).
fn strip_last_breadcrumb_segment(title: &str) -> String {
    for sep in TITLE_SEPARATORS {
        let Some(idx) = title.rfind(sep) else {
            continue;
        };
        let before = title[..idx].trim();
        let after = title[idx + sep.len()..].trim();
        if before.is_empty() || after.is_empty() {
            continue;
        }
        // Strip if it looks like a path or short breadcrumb
        let is_path = after.contains('/');
        let is_short = after.split_whitespace().count() <= 3;
        if is_path || is_short {
            return before.to_string();
        }
    }
    title.to_string()
}

// ------------------------------------------------------------------
// Author
// ------------------------------------------------------------------

fn extract_author(html: &Html, schema: Option<&serde_json::Value>) -> String {
    let raw = extract_author_raw(html, schema);
    clean_author(&raw)
}

/// Strip URLs, separator-delimited URLs, and "By " prefixes from author.
fn clean_author(author: &str) -> String {
    let mut result = author.to_string();
    // Strip trailing " - https://..." or " | https://..."
    for sep in [" - ", " | ", " · "] {
        if let Some(idx) = result.find(sep) {
            let after = result[idx + sep.len()..].trim();
            if after.starts_with("http://") || after.starts_with("https://") {
                result = result[..idx].trim().to_string();
            }
        }
    }
    // Strip if the entire value is a URL
    if result.starts_with("http://") || result.starts_with("https://") {
        return String::new();
    }
    result
}

fn extract_author_raw(html: &Html, schema: Option<&serde_json::Value>) -> String {
    if let Some(v) = get_meta_content(html, "property", "author")
        .or_else(|| get_meta_content(html, "name", "author"))
        .or_else(|| get_meta_content(html, "property", "article:author"))
        .or_else(|| get_meta_content(html, "property", "article:author_name"))
    {
        return v;
    }

    if let Some(v) = get_meta_content(html, "name", "sailthru.author") {
        return v;
    }

    if let Some(v) = schema_author(schema) {
        return v;
    }

    if let Some(v) = get_meta_content(html, "name", "byl") {
        return v;
    }

    if let Some(v) = get_meta_content(html, "name", "authorList") {
        return v;
    }

    if let Some(v) =
        get_meta_content(html, "name", "citation_author").map(|s| reverse_citation_author(&s))
    {
        return v;
    }

    if let Some(v) = get_dc_content(html, "creator") {
        return v;
    }

    if let Some(v) = get_meta_content(html, "name", "parsely-author") {
        return v;
    }

    if let Some(v) = itemprop_author(html) {
        return v;
    }

    if let Some(v) = class_author(html) {
        return v;
    }

    if let Some(v) = author_href_elements(html) {
        return v;
    }

    if let Some(v) = authors_link_elements(html) {
        return v;
    }

    String::new()
}

/// Reverse "Last, First" to "First Last" for `citation_author` meta.
fn reverse_citation_author(name: &str) -> String {
    if let Some((last, first)) = name.split_once(',') {
        let first = first.trim();
        let last = last.trim();
        if !first.is_empty() && !last.is_empty() {
            return format!("{first} {last}");
        }
    }
    name.to_string()
}

/// Extract author names from `[href*="/author/"]` elements (max 3).
fn author_href_elements(html: &Html) -> Option<String> {
    let Ok(sel) = Selector::parse("[href*=\"/author/\"]") else {
        return None;
    };
    let mut names = Vec::new();
    for el in html.select(&sel) {
        let text = dom::text_content(html, el.id());
        let trimmed = text.trim().to_string();
        if !trimmed.is_empty() {
            names.push(trimmed);
        }
        if names.len() >= 3 {
            break;
        }
    }
    if names.is_empty() {
        return None;
    }
    Some(names.join(", "))
}

/// Extract author names from `.authors a` elements (max 3).
fn authors_link_elements(html: &Html) -> Option<String> {
    let Ok(sel) = Selector::parse(".authors a") else {
        return None;
    };
    let mut names = Vec::new();
    for el in html.select(&sel) {
        let text = dom::text_content(html, el.id());
        let trimmed = text.trim().to_string();
        if !trimmed.is_empty() {
            names.push(trimmed);
        }
        if names.len() >= 3 {
            break;
        }
    }
    if names.is_empty() {
        return None;
    }
    Some(names.join(", "))
}

/// Extract author from a single schema object's "author" field.
fn extract_author_from_value(author: &serde_json::Value) -> Option<String> {
    // Single author object: {"name": "Alice"}
    if let Some(name) = author.get("name").and_then(|v| v.as_str()) {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    // Author as plain string
    if let Some(s) = author.as_str() {
        let trimmed = s.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    // Author as array: [{"name": "Alice"}, {"name": "Bob"}]
    if let serde_json::Value::Array(authors) = author {
        let names: Vec<&str> = authors
            .iter()
            .filter_map(|a| {
                a.get("name")
                    .and_then(|v| v.as_str())
                    .or_else(|| a.as_str())
            })
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        if !names.is_empty() {
            return Some(names.join(", "));
        }
    }
    None
}

fn schema_author(schema: Option<&serde_json::Value>) -> Option<String> {
    let data = schema?;
    // Direct object with "author" key
    if let Some(author) = data.get("author")
        && let Some(result) = extract_author_from_value(author)
    {
        return Some(result);
    }
    // If array, try each item
    if let serde_json::Value::Array(items) = data {
        for item in items {
            if let Some(author) = item.get("author")
                && let Some(result) = extract_author_from_value(author)
            {
                return Some(result);
            }
        }
    }
    None
}

fn itemprop_author(html: &Html) -> Option<String> {
    let Ok(sel) = Selector::parse("[itemprop=\"author\"]") else {
        return None;
    };
    let el = html.select(&sel).next()?;
    let text = dom::text_content(html, el.id());
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

fn class_author(html: &Html) -> Option<String> {
    // Prefer [rel="author"] or [itemprop="author"] over bare .author
    let selectors = ["[rel=\"author\"]", "[itemprop=\"author\"]", ".author"];
    for sel_str in selectors {
        let Ok(sel) = Selector::parse(sel_str) else {
            continue;
        };
        for el in html.select(&sel) {
            // Skip .author inside comment/mention/reply containers
            if is_inside_comment_section(html, el.id()) {
                continue;
            }
            let text = dom::text_content(html, el.id());
            let trimmed = text.trim().to_string();
            if !trimmed.is_empty() && trimmed.split_whitespace().count() <= 6 {
                return Some(trimmed);
            }
        }
    }
    None
}

fn is_inside_comment_section(html: &Html, node_id: ego_tree::NodeId) -> bool {
    let mut current = node_id;
    loop {
        if let Some(class) = dom::get_attr(html, current, "class") {
            let lower = class.to_lowercase();
            if lower.contains("comment")
                || lower.contains("mention")
                || lower.contains("repli")
                || lower.contains("backlink")
            {
                return true;
            }
        }
        let Some(parent) = dom::parent_element(html, current) else {
            return false;
        };
        current = parent;
    }
}

// ------------------------------------------------------------------
// Published date
// ------------------------------------------------------------------

fn extract_published(html: &Html, schema: Option<&serde_json::Value>) -> String {
    schema_str(schema, "datePublished")
        .or_else(|| get_meta_content(html, "name", "publishDate"))
        .or_else(|| get_meta_content(html, "property", "article:published_time"))
        .or_else(|| get_dc_content(html, "date"))
        .or_else(|| get_meta_content(html, "name", "DCTERMS.created"))
        .or_else(|| get_meta_content(html, "name", "DCTERMS.issued"))
        .or_else(|| get_meta_content(html, "name", "parsely-pub-date"))
        .or_else(|| abbr_date_published(html))
        .or_else(|| get_meta_content(html, "name", "sailthru.date"))
        .or_else(|| first_time_element(html))
        .unwrap_or_default()
}

// ------------------------------------------------------------------
// Modified date
// ------------------------------------------------------------------

fn extract_modified(html: &Html, schema: Option<&serde_json::Value>) -> String {
    schema_str(schema, "dateModified")
        .or_else(|| get_meta_content(html, "property", "article:modified_time"))
        .or_else(|| get_meta_content(html, "property", "og:updated_time"))
        .or_else(|| get_meta_content(html, "name", "DCTERMS.modified"))
        .or_else(|| get_meta_content(html, "name", "dcterms.modified"))
        .or_else(|| get_meta_content(html, "http-equiv", "last-modified"))
        .unwrap_or_default()
}

/// Extract date from `abbr[itemprop="datePublished"]` -- check
/// `datetime` attr first, then text content.
fn abbr_date_published(html: &Html) -> Option<String> {
    let Ok(sel) = Selector::parse("abbr[itemprop=\"datePublished\"]") else {
        return None;
    };
    let el = html.select(&sel).next()?;
    if let Some(dt) = el.value().attr("datetime") {
        let trimmed = dt.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    let text = dom::text_content(html, el.id());
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

/// Find a publication-related `<time>` element near the start of the
/// document. Prefers elements with a `datetime` attribute (more
/// likely article dates vs. comment timestamps) and only inspects
/// the first 5 `<time>` elements to avoid picking up comment dates.
fn first_time_element(html: &Html) -> Option<String> {
    let Ok(sel) = Selector::parse("time") else {
        return None;
    };

    let candidates: Vec<_> = html.select(&sel).take(5).collect();

    // First pass: prefer <time datetime="...">
    for el in &candidates {
        if let Some(dt) = el.value().attr("datetime") {
            let trimmed = dt.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    // Second pass: fall back to text content
    for el in &candidates {
        let text = dom::text_content(html, el.id());
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    None
}

// ------------------------------------------------------------------
// Site name
// ------------------------------------------------------------------

fn extract_site_name(html: &Html, schema: Option<&serde_json::Value>, author: &str) -> String {
    schema_str(schema, "publisher.name")
        .or_else(|| get_meta_content(html, "property", "og:site_name"))
        .or_else(|| schema_graph_website_name(schema))
        .or_else(|| schema_str(schema, "sourceOrganization.name"))
        .or_else(|| get_meta_content(html, "name", "copyright"))
        .or_else(|| schema_str(schema, "copyrightHolder.name"))
        .or_else(|| schema_str(schema, "isPartOf.name"))
        .or_else(|| get_dc_content(html, "publisher"))
        .or_else(|| get_meta_content(html, "name", "application-name"))
        // Fallback: use author name as site name (matches defuddle behavior)
        .or_else(|| {
            if !author.is_empty()
                && author.split_whitespace().count() <= 4
                && !author.contains(',')
                && !author.contains("http")
            {
                Some(author.to_string())
            } else {
                None
            }
        })
        // Fallback: infer from <title> element's last segment after separator
        .or_else(|| site_name_from_title(html))
        .and_then(|name| {
            if name.split_whitespace().count() > 6 {
                None
            } else {
                Some(name)
            }
        })
        .unwrap_or_default()
}

/// Extract site name from `<title>` element's last separator-delimited
/// segment. e.g., "Article Title - Wikipedia" → "Wikipedia".
/// Only returns if the last segment is short (≤4 words) and the title
/// segment before it is longer (indicating it's the article title).
fn site_name_from_title(html: &Html) -> Option<String> {
    let title = title_element_text(html)?;
    // Only use unambiguous separators for site name inference
    // (skip " · ", " / ", " -- " which are used within titles)
    for sep in &[" | ", " - "] {
        let Some(idx) = title.rfind(sep) else {
            continue;
        };
        let before = title[..idx].trim();
        let after = title[idx + sep.len()..].trim();
        if after.is_empty() || before.is_empty() {
            continue;
        }
        // Last segment ≤4 words, before must be ≥2 words (a real article title)
        // and at least as long as the suffix
        if after.split_whitespace().count() <= 4
            && before.split_whitespace().count() >= 2
            && before.len() >= after.len()
        {
            return Some(after.to_string());
        }
    }
    None
}

/// Search a single schema object's `@graph` for `@type: "WebSite"`
/// and return its `name`.
fn website_name_from_graph(obj: &serde_json::Value) -> Option<String> {
    let graph = obj.get("@graph")?.as_array()?;
    for item in graph {
        let Some(type_val) = item.get("@type") else {
            continue;
        };
        let is_website = type_val.as_str() == Some("WebSite")
            || type_val
                .as_array()
                .is_some_and(|a| a.iter().any(|v| v.as_str() == Some("WebSite")));
        if !is_website {
            continue;
        }
        let Some(name) = item.get("name").and_then(|v| v.as_str()) else {
            continue;
        };
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

/// Search Schema.org `@graph` for `@type: "WebSite"` and return its
/// `name`. Handles both single objects and arrays.
fn schema_graph_website_name(schema: Option<&serde_json::Value>) -> Option<String> {
    let data = schema?;
    if let Some(result) = website_name_from_graph(data) {
        return Some(result);
    }
    if let serde_json::Value::Array(items) = data {
        for item in items {
            if let Some(result) = website_name_from_graph(item) {
                return Some(result);
            }
        }
    }
    None
}

// ------------------------------------------------------------------
// Description
// ------------------------------------------------------------------

fn extract_description(html: &Html, schema: Option<&serde_json::Value>) -> String {
    get_meta_content(html, "name", "description")
        .or_else(|| get_meta_content(html, "property", "og:description"))
        .or_else(|| get_dc_content(html, "description"))
        .or_else(|| get_meta_content(html, "property", "twitter:description"))
        .or_else(|| get_meta_content(html, "name", "twitter:description"))
        .or_else(|| get_meta_content(html, "name", "sailthru.description"))
        .or_else(|| schema_str(schema, "description"))
        .unwrap_or_default()
}

// ------------------------------------------------------------------
// Image
// ------------------------------------------------------------------

fn extract_image(html: &Html, schema: Option<&serde_json::Value>) -> String {
    get_meta_content(html, "property", "og:image")
        .or_else(|| get_meta_content(html, "property", "twitter:image"))
        .or_else(|| get_meta_content(html, "name", "twitter:image"))
        .or_else(|| get_meta_content(html, "name", "twitter:image:src"))
        .or_else(|| get_meta_content(html, "name", "parsely-image-url"))
        .or_else(|| get_meta_content(html, "name", "thumbnail"))
        .or_else(|| schema_image(schema))
        .unwrap_or_default()
}

fn schema_image(schema: Option<&serde_json::Value>) -> Option<String> {
    let image = schema?.get("image")?;
    if let Some(url) = image.get("url").and_then(|v| v.as_str()) {
        let trimmed = url.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    if let Some(s) = image.as_str() {
        let trimmed = s.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

// ------------------------------------------------------------------
// Language
// ------------------------------------------------------------------

fn extract_language(html: &Html, schema: Option<&serde_json::Value>) -> String {
    html_lang(html)
        .or_else(|| get_meta_content(html, "name", "content-language"))
        .or_else(|| get_meta_content(html, "http-equiv", "content-language"))
        .or_else(|| get_dc_content(html, "language"))
        .or_else(|| get_meta_content(html, "property", "og:locale"))
        .or_else(|| schema_str(schema, "inLanguage"))
        .map(|s| normalize_bcp47(&s))
        .unwrap_or_default()
}

/// Normalize BCP 47 language tags: replace `_` with `-`.
fn normalize_bcp47(lang: &str) -> String {
    lang.replace('_', "-")
}

fn html_lang(html: &Html) -> Option<String> {
    let Ok(sel) = Selector::parse("html") else {
        return None;
    };
    let el = html.select(&sel).next()?;
    let lang = el.value().attr("lang")?.trim();
    if lang.is_empty() {
        return None;
    }
    Some(lang.to_string())
}

// ------------------------------------------------------------------
// Favicon
// ------------------------------------------------------------------

fn extract_favicon(html: &Html, url: Option<&str>) -> String {
    get_meta_content(html, "property", "og:image:favicon")
        .or_else(|| link_icon(html, "icon", url))
        .or_else(|| link_icon(html, "shortcut icon", url))
        .or_else(|| favicon_fallback(url))
        .unwrap_or_default()
}

fn link_icon(html: &Html, rel: &str, base_url: Option<&str>) -> Option<String> {
    let selector_str = format!("link[rel=\"{rel}\"]");
    let Ok(sel) = Selector::parse(&selector_str) else {
        return None;
    };
    let el = html.select(&sel).next()?;
    let href = el.value().attr("href")?.trim();
    if href.is_empty() {
        return None;
    }
    Some(resolve_favicon_url(href, base_url))
}

/// Resolve a favicon href against a base URL. Returns the href
/// unchanged when it is already absolute or resolution fails.
fn resolve_favicon_url(href: &str, base_url: Option<&str>) -> String {
    if href.starts_with("http://") || href.starts_with("https://") {
        return href.to_string();
    }
    if let Some(base) = base_url
        && let Ok(base_parsed) = url::Url::parse(base)
        && let Ok(resolved) = base_parsed.join(href)
    {
        return resolved.to_string();
    }
    href.to_string()
}

fn favicon_fallback(url: Option<&str>) -> Option<String> {
    let base = base_url(url?)?;
    Some(format!("{base}/favicon.ico"))
}

// ------------------------------------------------------------------
// Domain
// ------------------------------------------------------------------

/// Derive a human-readable site name from a domain.
/// "en.wikipedia.org" -> "Wikipedia", "github.com" -> "GitHub".
fn domain_to_site_name(domain: &str) -> String {
    if domain.is_empty() {
        return String::new();
    }
    // Strip common prefixes (www. or 2-letter language subdomains)
    let stripped = domain
        .strip_prefix("www.")
        .or_else(|| {
            let parts: Vec<&str> = domain.splitn(2, '.').collect();
            // Only strip 2-letter subdomains that look like language
            // codes (ISO 639-1), not short domain labels like "bbc"
            if parts.len() == 2
                && parts[0].len() == 2
                && parts[0].chars().all(|c| c.is_ascii_lowercase())
            {
                Some(parts[1])
            } else {
                None
            }
        })
        .unwrap_or(domain);
    // Take the second-level domain (before .com/.org/etc.)
    let name_part = stripped.split('.').next().unwrap_or(stripped);
    if name_part.is_empty() {
        return String::new();
    }
    // Capitalize first letter
    let mut chars = name_part.chars();
    match chars.next() {
        Some(first) => {
            let mut result = first.to_uppercase().to_string();
            result.extend(chars);
            result
        }
        None => String::new(),
    }
}

fn extract_domain(url: Option<&str>) -> String {
    let Some(raw) = url else {
        return String::new();
    };
    let Ok(parsed) = url::Url::parse(raw) else {
        return String::new();
    };
    parsed.host_str().unwrap_or_default().to_string()
}

/// Return scheme + host (+ port if non-default).
fn base_url(raw: &str) -> Option<String> {
    let parsed = url::Url::parse(raw).ok()?;
    let host = parsed.host_str()?;
    let scheme = parsed.scheme();
    match parsed.port() {
        Some(port) => Some(format!("{scheme}://{host}:{port}")),
        None => Some(format!("{scheme}://{host}")),
    }
}

// ------------------------------------------------------------------
// Canonical URL
// ------------------------------------------------------------------

fn extract_canonical_url(html: &Html) -> String {
    // link[rel=canonical] is the strongest signal
    if let Ok(sel) = Selector::parse("link[rel=\"canonical\"]") {
        if let Some(el) = html.select(&sel).next() {
            if let Some(href) = el.value().attr("href") {
                let trimmed = href.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
    }
    get_meta_content(html, "property", "og:url").unwrap_or_default()
}

// ------------------------------------------------------------------
// Keywords / Tags
// ------------------------------------------------------------------

fn extract_keywords(html: &Html) -> Vec<String> {
    // Try multiple sources, take the first non-empty one
    let raw = get_meta_content(html, "name", "keywords")
        .or_else(|| get_dc_content(html, "subject"))
        .or_else(|| get_meta_content(html, "name", "news_keywords"))
        .or_else(|| get_meta_content(html, "name", "parsely-tags"));

    if let Some(raw) = raw {
        return raw
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    // article:tag can appear multiple times
    if let Ok(sel) = Selector::parse("meta[property=\"article:tag\"]") {
        let tags: Vec<String> = html
            .select(&sel)
            .filter_map(|el| el.value().attr("content"))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !tags.is_empty() {
            return tags;
        }
    }

    Vec::new()
}

// ------------------------------------------------------------------
// Content Type
// ------------------------------------------------------------------

fn extract_content_type(html: &Html, schema: Option<&serde_json::Value>) -> String {
    get_meta_content(html, "property", "og:type")
        .or_else(|| get_dc_content(html, "type"))
        .or_else(|| schema_type(schema))
        .unwrap_or_default()
}

/// Extract @type from schema.org data (e.g., "Article", "`NewsArticle`").
fn schema_type(schema: Option<&serde_json::Value>) -> Option<String> {
    let data = schema?;
    if let Some(t) = data.get("@type").and_then(|v| v.as_str()) {
        return Some(t.to_string());
    }
    if let serde_json::Value::Array(items) = data {
        for item in items {
            if let Some(t) = item.get("@type").and_then(|v| v.as_str()) {
                // Skip generic types, prefer specific content types
                if t != "WebSite" && t != "WebPage" && t != "BreadcrumbList" {
                    return Some(t.to_string());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

    #[test]
    fn title_from_og() {
        let doc = Html::parse_document(
            r#"<html><head>
            <meta property="og:title" content="OG Title">
            <title>Fallback</title>
            </head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.title, "OG Title");
    }

    #[test]
    fn title_suffix_inferred_as_site_name() {
        let doc = Html::parse_document(
            r"<html><head>
            <title>Article Name | Site Name</title>
            </head><body></body></html>",
        );
        let m = extract_metadata(&doc, None, None);
        // Site name inferred from title suffix, title stripped
        assert_eq!(m.title, "Article Name");
        assert_eq!(m.site_name, "Site Name");
    }

    #[test]
    fn title_stripped_with_og_site_name() {
        let doc = Html::parse_document(
            r#"<html><head>
            <meta property="og:site_name" content="Site Name">
            <title>Article Name | Site Name</title>
            </head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.title, "Article Name");
    }

    #[test]
    fn title_stripped_when_site_name_matches() {
        let doc = Html::parse_document(
            r#"<html><head>
            <meta property="og:site_name" content="Wikipedia">
            <title>Bengaluru - Wikipedia</title>
            </head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.title, "Bengaluru");
    }

    #[test]
    fn title_not_stripped_when_site_name_mismatches() {
        let doc = Html::parse_document(
            r#"<html><head>
            <meta property="og:site_name" content="MyBlog">
            <title>Part A - Part B</title>
            </head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.title, "Part A - Part B");
    }

    #[test]
    fn author_from_meta() {
        let doc = Html::parse_document(
            r#"<html><head>
            <meta name="author" content="Jane Doe">
            </head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.author, "Jane Doe");
    }

    #[test]
    fn author_from_schema_object() {
        let schema: serde_json::Value = serde_json::json!({
            "author": { "name": "Schema Author" }
        });
        let doc = Html::parse_document("<html><body></body></html>");
        let m = extract_metadata(&doc, None, Some(&schema));
        assert_eq!(m.author, "Schema Author");
    }

    #[test]
    fn domain_extracted_from_url() {
        let doc = Html::parse_document("<html><body></body></html>");
        let m = extract_metadata(&doc, Some("https://example.com/page"), None);
        assert_eq!(m.domain, "example.com");
    }

    #[test]
    fn favicon_fallback_to_root() {
        let doc = Html::parse_document("<html><body></body></html>");
        let m = extract_metadata(&doc, Some("https://example.com/a/b"), None);
        assert_eq!(m.favicon, "https://example.com/favicon.ico");
    }

    #[test]
    fn language_from_html_attr() {
        let doc = Html::parse_document(r#"<html lang="en-US"><body></body></html>"#);
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.language, "en-US");
    }

    #[test]
    fn description_from_meta() {
        let doc = Html::parse_document(
            r#"<html><head>
            <meta name="description" content="A page about things">
            </head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.description, "A page about things");
    }

    #[test]
    fn published_from_schema() {
        let schema: serde_json::Value = serde_json::json!({
            "datePublished": "2025-01-15"
        });
        let doc = Html::parse_document("<html><body></body></html>");
        let m = extract_metadata(&doc, None, Some(&schema));
        assert_eq!(m.published, "2025-01-15");
    }

    #[test]
    fn site_name_from_og() {
        let doc = Html::parse_document(
            r#"<html><head>
            <meta property="og:site_name" content="My Site">
            </head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.site_name, "My Site");
    }

    #[test]
    fn image_from_schema_string() {
        let schema: serde_json::Value = serde_json::json!({
            "image": "https://img.example.com/photo.jpg"
        });
        let doc = Html::parse_document("<html><body></body></html>");
        let m = extract_metadata(&doc, None, Some(&schema));
        assert_eq!(m.image, "https://img.example.com/photo.jpg");
    }

    #[test]
    fn empty_metadata_for_blank_doc() {
        let doc = Html::parse_document("<html><body></body></html>");
        let m = extract_metadata(&doc, None, None);
        assert!(m.title.is_empty());
        assert!(m.author.is_empty());
        assert!(m.published.is_empty());
        assert!(m.domain.is_empty());
    }

    // ── clean_title ─────────────────────────────────────────────────

    #[test]
    fn clean_title_strips_trailing_site_name_pipe() {
        let result = clean_title("Article Title | Site Name", "Site Name", None, None);
        assert_eq!(result, "Article Title");
    }

    #[test]
    fn clean_title_strips_trailing_site_name_dash() {
        let result = clean_title("Article Title - Site Name", "Site Name", None, None);
        assert_eq!(result, "Article Title");
    }

    #[test]
    fn clean_title_strips_leading_site_name() {
        let result = clean_title("Site Name | Article Title", "Site Name", None, None);
        assert_eq!(result, "Article Title");
    }

    #[test]
    fn clean_title_unchanged_without_separator() {
        let result = clean_title("Title With No Separator", "Site Name", None, None);
        assert_eq!(result, "Title With No Separator");
    }

    #[test]
    fn clean_title_keeps_longer_part() {
        let result = clean_title("Short | Very Long Article Title Here", "Short", None, None);
        assert_eq!(result, "Very Long Article Title Here");
    }

    #[test]
    fn schema_str_from_array() {
        let schema: serde_json::Value = serde_json::json!([
            {"@type": "WebPage"},
            {"@type": "Article", "headline": "Array Title", "datePublished": "2025-06-01"}
        ]);
        let doc = Html::parse_document("<html><body></body></html>");
        let m = extract_metadata(&doc, None, Some(&schema));
        assert_eq!(m.title, "Array Title");
        assert_eq!(m.published, "2025-06-01");
    }

    #[test]
    fn author_from_schema_array() {
        let schema: serde_json::Value = serde_json::json!({
            "author": [{"name": "Alice"}, {"name": "Bob"}]
        });
        let doc = Html::parse_document("<html><body></body></html>");
        let m = extract_metadata(&doc, None, Some(&schema));
        assert_eq!(m.author, "Alice, Bob");
    }

    #[test]
    fn author_from_array_schema_item() {
        let schema: serde_json::Value = serde_json::json!([
            {"@type": "WebPage"},
            {"@type": "Article", "author": {"name": "Charlie"}}
        ]);
        let doc = Html::parse_document("<html><body></body></html>");
        let m = extract_metadata(&doc, None, Some(&schema));
        assert_eq!(m.author, "Charlie");
    }

    #[test]
    fn graph_website_name_from_array_schema() {
        let schema: serde_json::Value = serde_json::json!([
            {
                "@graph": [
                    {"@type": "WebSite", "name": "My Blog"}
                ]
            }
        ]);
        let doc = Html::parse_document("<html><body></body></html>");
        let m = extract_metadata(&doc, None, Some(&schema));
        assert_eq!(m.site_name, "My Blog");
    }
}

#[cfg(test)]
mod dc_tests {
    use super::*;
    use scraper::Html;

    #[test]
    fn dc_title() {
        let doc = Html::parse_document(
            r#"<html><head><meta name="DC.title" content="Dublin Core Title"></head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.title, "Dublin Core Title");
    }

    #[test]
    fn dcterm_title() {
        let doc = Html::parse_document(
            r#"<html><head><meta name="dcterm:title" content="DCTerm Title"></head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.title, "DCTerm Title");
    }

    #[test]
    fn dc_creator_as_author() {
        let doc = Html::parse_document(
            r#"<html><head><meta name="DC.creator" content="Jane Doe"></head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.author, "Jane Doe");
    }

    #[test]
    fn dc_date_as_published() {
        let doc = Html::parse_document(
            r#"<html><head><meta name="DC.date" content="2025-06-15"></head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.published, "2025-06-15");
    }

    #[test]
    fn dcterms_created_as_published() {
        let doc = Html::parse_document(
            r#"<html><head><meta name="DCTERMS.created" content="2025-01-01T00:00:00Z"></head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.published, "2025-01-01T00:00:00Z");
    }

    #[test]
    fn dc_description() {
        let doc = Html::parse_document(
            r#"<html><head><meta name="DC.description" content="A Dublin Core description"></head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.description, "A Dublin Core description");
    }

    #[test]
    fn dc_language() {
        let doc = Html::parse_document(
            r#"<html><head><meta name="DC.language" content="en-US"></head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.language, "en-US");
    }

    #[test]
    fn og_takes_precedence_over_dc() {
        let doc = Html::parse_document(
            r#"<html><head>
            <meta property="og:title" content="OG Title">
            <meta name="DC.title" content="DC Title">
            </head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.title, "OG Title");
    }

    #[test]
    fn parsely_author() {
        let doc = Html::parse_document(
            r#"<html><head><meta name="parsely-author" content="Parsely Author"></head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.author, "Parsely Author");
    }

    #[test]
    fn parsely_pub_date() {
        let doc = Html::parse_document(
            r#"<html><head><meta name="parsely-pub-date" content="2025-03-20"></head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.published, "2025-03-20");
    }
}

#[cfg(test)]
mod modified_tests {
    use super::*;
    use scraper::Html;

    #[test]
    fn modified_from_article_modified_time() {
        let doc = Html::parse_document(
            r#"<html><head><meta property="article:modified_time" content="2025-09-01T12:00:00Z"></head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.modified, "2025-09-01T12:00:00Z");
    }

    #[test]
    fn modified_from_og_updated_time() {
        let doc = Html::parse_document(
            r#"<html><head><meta property="og:updated_time" content="2025-08-15"></head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.modified, "2025-08-15");
    }

    #[test]
    fn modified_from_dcterms() {
        let doc = Html::parse_document(
            r#"<html><head><meta name="DCTERMS.modified" content="2025-07-20"></head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.modified, "2025-07-20");
    }

    #[test]
    fn modified_from_schema_date_modified() {
        let schema: serde_json::Value = serde_json::json!({
            "@type": "Article",
            "dateModified": "2025-06-10T08:00:00Z"
        });
        let doc = Html::parse_document("<html><body></body></html>");
        let m = extract_metadata(&doc, None, Some(&schema));
        assert_eq!(m.modified, "2025-06-10T08:00:00Z");
    }

    #[test]
    fn modified_empty_when_absent() {
        let doc = Html::parse_document("<html><body></body></html>");
        let m = extract_metadata(&doc, None, None);
        assert!(m.modified.is_empty());
    }

    #[test]
    fn dc_publisher_as_site_name() {
        let doc = Html::parse_document(
            r#"<html><head><meta name="DC.publisher" content="Example Press"></head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.site_name, "Example Press");
    }
}

#[cfg(test)]
mod new_fields_tests {
    use super::*;
    use scraper::Html;

    #[test]
    fn canonical_url_from_link_rel() {
        let doc = Html::parse_document(
            r#"<html><head><link rel="canonical" href="https://example.com/article"></head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.canonical_url, "https://example.com/article");
    }

    #[test]
    fn canonical_url_from_og_url() {
        let doc = Html::parse_document(
            r#"<html><head><meta property="og:url" content="https://example.com/page"></head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.canonical_url, "https://example.com/page");
    }

    #[test]
    fn keywords_from_meta() {
        let doc = Html::parse_document(
            r#"<html><head><meta name="keywords" content="rust, programming, web"></head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.keywords, vec!["rust", "programming", "web"]);
    }

    #[test]
    fn keywords_from_dc_subject() {
        let doc = Html::parse_document(
            r#"<html><head><meta name="DC.subject" content="science, biology"></head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.keywords, vec!["science", "biology"]);
    }

    #[test]
    fn keywords_from_article_tag() {
        let doc = Html::parse_document(
            r#"<html><head>
            <meta property="article:tag" content="rust">
            <meta property="article:tag" content="web">
            </head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.keywords, vec!["rust", "web"]);
    }

    #[test]
    fn keywords_empty_when_absent() {
        let doc = Html::parse_document("<html><body></body></html>");
        let m = extract_metadata(&doc, None, None);
        assert!(m.keywords.is_empty());
    }

    #[test]
    fn content_type_from_og_type() {
        let doc = Html::parse_document(
            r#"<html><head><meta property="og:type" content="article"></head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.content_type, "article");
    }

    #[test]
    fn content_type_from_schema() {
        let schema: serde_json::Value = serde_json::json!({"@type": "NewsArticle"});
        let doc = Html::parse_document("<html><body></body></html>");
        let m = extract_metadata(&doc, None, Some(&schema));
        assert_eq!(m.content_type, "NewsArticle");
    }

    #[test]
    fn content_type_skips_generic_schema_types() {
        let schema: serde_json::Value = serde_json::json!([
            {"@type": "WebSite", "name": "Example"},
            {"@type": "Article", "headline": "Test"}
        ]);
        let doc = Html::parse_document("<html><body></body></html>");
        let m = extract_metadata(&doc, None, Some(&schema));
        assert_eq!(m.content_type, "Article");
    }

    #[test]
    fn image_from_twitter_image_src() {
        let doc = Html::parse_document(
            r#"<html><head><meta name="twitter:image:src" content="https://img.example.com/photo.jpg"></head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.image, "https://img.example.com/photo.jpg");
    }

    #[test]
    fn author_from_article_author() {
        let doc = Html::parse_document(
            r#"<html><head><meta property="article:author" content="Jane Smith"></head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.author, "Jane Smith");
    }
}
