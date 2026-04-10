use std::sync::LazyLock;

use regex::Regex;

/// CSS selectors that match elements to remove entirely.
pub const EXACT_SELECTORS: &[&str] = &[
    "noscript",
    "script:not([type^=\"math/\"])",
    "style",
    "meta",
    "link",
    ".ad, [class^=\"ad-\"], [class$=\"-ad\"], [data-ad-wrapper]",
    "[role=\"banner\"]:not(header)",
    ".promo, [class*=\"promo-\"]",
    "[id=\"comments\"], [id=\"comment\"], .comments-section",
    "header:not(:has(p + p))",
    "nav, [role=\"navigation\"], [role=\"dialog\"]",
    ".menu, .nav, .navbar, .navigation",
    ".author, .date, .meta, .tags",
    ".headline, #title",
    "[href*=\"/author/\"]",
    ".toc, #toc, .table-of-contents",
    "footer",
    "button, form, input:not([type=\"checkbox\"]), select, textarea",
    "[hidden], [aria-hidden=\"true\"]:not([class*=\"math\"]):not(svg)",
    concat!(
        ".hidden:not([class*=\"sm:\"])",
        ":not([class*=\"md:\"])",
        ":not([class*=\"lg:\"])",
        ":not([class*=\"xl:\"])",
    ),
    ".invisible",
    ".sidebar, #sidebar, #secondary, [role=\"complementary\"]",
    "aside:not([role=\"note\"])",
    "table.infobox",
    ".widget, .related, .recommended",
    ".social, .share, .sharing",
    ".newsletter, .subscribe",
    ".breadcrumb, .breadcrumbs",
    ".pagination",
    ".alert, .banner, .notice:not([role=\"note\"])",
    "[role=\"search\"]",
    ".skip-link, .sr-only, .screen-reader-text",
];

/// Attributes to test partial patterns against.
pub const PARTIAL_ATTRIBUTES: &[&str] = &[
    "class",
    "id",
    "data-component",
    "data-test",
    "data-testid",
    "data-test-id",
    "data-qa",
    "data-cy",
];

const PARTIAL_PATTERNS: &[&str] = &[
    "ad-slot",
    "advert",
    "advertisement",
    "adsense",
    "ad-container",
    "ad-wrapper",
    "ad-banner",
    "ad-unit",
    "ad-zone",
    "article-meta",
    "article-tag",
    "article-share",
    "article-footer",
    "article-header",
    "author",
    "byline",
    "byl",
    "breadcrumb",
    "comment",
    "comments",
    "cookie",
    "consent",
    "date",
    "datetime",
    "timestamp",
    "published",
    "footer",
    "foot",
    "header-nav",
    "header-menu",
    "menu",
    "nav",
    "navbar",
    "navigation",
    "newsletter",
    "subscribe",
    "subscription",
    "signup",
    "popular",
    "trending",
    "recommended",
    "related",
    "suggested",
    "more-stories",
    "share",
    "sharing",
    "social",
    "sidebar",
    "side-bar",
    "skip-nav",
    "skip-link",
    "sponsor",
    "sponsored",
    "sticky-header",
    "sticky-nav",
    "tag",
    "tags",
    "topic",
    "category",
    "toolbar",
    "tool-bar",
    "tooltip",
    "promo",
    "promotion",
    "promotional",
    "widget",
    "alert",
    "banner",
    "popup",
    "modal",
    "overlay",
    "lightbox",
    "card-grid",
    "article-list",
    "story-list",
    "masthead",
    "paywall",
    "access-wall",
    "regwall",
    "print-only",
    "table-of-contents",
    "toc",
];

/// Compiled regex matching any partial pattern as a substring,
/// case-insensitive.
pub static PARTIAL_REGEX: LazyLock<Regex> =
    LazyLock::new(build_partial_pattern);

/// Get a reference to the compiled partial pattern regex.
#[must_use] 
pub fn partial_pattern() -> &'static Regex {
    &PARTIAL_REGEX
}

/// Builds a compiled regex from all partial patterns, joined with `|`
/// and compiled with the case-insensitive flag.
///
/// # Panics
///
/// Panics if the fallback regex literal is somehow invalid. This cannot
/// happen in practice because all patterns are compile-time constants.
#[must_use]
pub fn build_partial_pattern() -> Regex {
    let mut combined = String::new();
    for (i, pattern) in PARTIAL_PATTERNS.iter().enumerate() {
        if i > 0 {
            combined.push('|');
        }
        combined.push_str(&regex::escape(pattern));
    }

    let Ok(re) = regex::RegexBuilder::new(&combined)
        .case_insensitive(true)
        .build()
    else {
        // All patterns are known at compile time and are valid after
        // escaping, so this branch is unreachable in practice.
        // Return a regex that never matches as a safe fallback.
        #[allow(clippy::unwrap_used)]
        return Regex::new(r"^\b$").unwrap();
    };
    re
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_regex_compiles() {
        let re = build_partial_pattern();
        assert!(re.is_match("ad-slot"));
        assert!(re.is_match("AD-SLOT"));
        assert!(re.is_match("my-sidebar-widget"));
    }

    #[test]
    fn partial_regex_rejects_unrelated() {
        let re = build_partial_pattern();
        assert!(!re.is_match("main-content"));
        assert!(!re.is_match("paragraph"));
    }

    #[test]
    fn exact_selectors_not_empty() {
        assert!(!EXACT_SELECTORS.is_empty());
    }

    #[test]
    fn partial_attributes_has_class_and_id() {
        assert!(PARTIAL_ATTRIBUTES.contains(&"class"));
        assert!(PARTIAL_ATTRIBUTES.contains(&"id"));
    }

    #[test]
    fn lazy_static_works() {
        assert!(PARTIAL_REGEX.is_match("newsletter"));
    }
}
