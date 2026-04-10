use std::sync::LazyLock;

use fancy_regex::Regex;
use scraper::Selector;

/// CSS selectors that match elements to remove entirely.
/// Each entry is a single CSS selector compatible with the `scraper` crate.
pub const EXACT_SELECTORS: &[&str] = &[
    // Scripts, styles, metadata
    "noscript",
    "script:not([type^=\"math/\"])",
    "style",
    "meta",
    "link",
    "audio:not([src])",
    // Ads
    ".ad:not([class*=\"gradient\"])",
    "[class^=\"ad-\" i]",
    "[class$=\"-ad\" i]",
    "[data-ad-wrapper]",
    "[id^=\"ad-\" i]",
    "[id$=\"-ad\" i]",
    "[role=\"banner\" i]",
    "[alt*=\"advert\" i]",
    ".promo",
    ".Promo",
    "#barrier-page",
    ".alert:not([data-callout])",
    // Comments
    "[id=\"comments\" i]",
    "[id=\"comment\" i]",
    // Cover elements
    "div[class*=\"cover-\"]",
    "div[id*=\"cover-\"]",
    // Custom elements
    "ads-breadcrumbs",
    // Header / banner / nav
    // Note: defuddle uses `header:not(:has(p + p))` but scraper does
    // not support `:has()`. Content-wrapping headers are handled
    // separately in cleanup::remove_header_elements.
    ".header:not(.banner)",
    "#header",
    "#Header",
    "#banner",
    "#Banner",
    "nav",
    ".navigation",
    "#navigation",
    "[role=\"navigation\" i]",
    "[role=\"dialog\" i]",
    "[role*=\"complementary\" i]",
    "[class*=\"pagination\" i]",
    ".menu",
    "#siteSub",
    // Metadata / authorship
    ".previous",
    ".author",
    ".Author",
    "[class$=\"_bio\"]",
    "#categories",
    ".contributor",
    ".date",
    "#date",
    "[data-date]",
    ".entry-meta",
    ".meta",
    ".tags",
    "#tags",
    "[rel=\"tag\"]",
    // Headlines / titles
    ".headline",
    "#headline",
    "#title",
    "#Title",
    "#articleTag",
    // Author / utility links
    "[href*=\"/author/\"]",
    "[href*=\"/author?\"]",
    "[href$=\"/author\"]",
    "a[href*=\"copyright.com\"]",
    "a[href*=\"google.com/preferences\"]",
    "[href=\"#top\"]",
    "[href=\"#Top\"]",
    "[href=\"#page-header\"]",
    "[href=\"#content\"]",
    "[href=\"#site-content\"]",
    "[href=\"#main-content\"]",
    "[href^=\"#main\"]",
    "[src*=\"author\"]",
    // Table of contents
    ".toc",
    ".Toc",
    "#toc",
    "[href*=\"#toc\"]",
    // Structural
    "footer",
    ".aside",
    "aside:not([class*=\"callout\"])",
    // Interactive / form elements
    "button",
    "[class*=\"dismiss\" i]",
    "[class*=\"close-btn\" i]",
    "[class*=\"btn-close\" i]",
    "canvas",
    "date",
    "dialog",
    "fieldset",
    "form",
    "input:not([type=\"checkbox\"])",
    "label",
    "option",
    "select",
    "[role=\"listbox\"]",
    "[role=\"option\"]",
    "textarea",
    // Hidden elements
    "[hidden]",
    "[aria-hidden=\"true\"]:not([class*=\"math\"]):not(svg)",
    ".hidden",
    ".invisible",
    // Media / embed
    "instaread-player",
    "iframe:not([src])",
    // Logo
    "[class=\"logo\" i]",
    "#logo",
    "#Logo",
    // Newsletter / subscribe
    "#newsletter",
    "#Newsletter",
    ".subscribe",
    "[data-component-name=\"ButtonCreateButton\"]",
    "[data-component-name=\"DigestPostEmbed\"]",
    "[data-component-name=\"SubscribeWidgetToDOM\"]",
    "[class*=\"digestPostEmbed\"]",
    // Print
    ".noprint",
    "[data-print-layout=\"hide\" i]",
    "[data-block=\"donotprint\" i]",
    // Misc
    "[class*=\"clickable-icon\" i]",
    "li span[class*=\"ltx_tag\" i][class*=\"ltx_tag_item\" i]",
    "a[href^=\"#\"][class*=\"anchor\" i]",
    "a[href^=\"#\"][class*=\"ref\" i]:not(.ltx_ref):not(.footnote-backref)",
    "[data-container*=\"most-viewed\" i]",
    // Sidebar
    ".sidebar",
    ".Sidebar",
    "#sidebar",
    "#Sidebar",
    "#side-bar",
    "#secondary",
    "#sitesub",
    // Site / skip links
    "[href*=\"/sitemap/sitemap.xml\"]",
    "[data-link-name*=\"skip\" i]",
    "[aria-label*=\"skip\" i]",
    // Copyright / license
    ".copyright",
    "#copyright",
    ".licensebox",
    "#page-info",
    // RSS / feeds
    "#rss",
    "#feed",
    // Layout / misc
    ".gutter",
    "#primaryaudio",
    "#NYT_ABOVE_MAIN_CONTENT_REGION",
    "[data-testid=\"photoviewer-children-figure\"] > span",
    "table.infobox",
    ".infobox",
    // Wikipedia navigation / reference
    "table.navbox",
    ".navbox",
    ".navbox-container",
    // .reflist, .references, .mw-references-wrap are NOT here —
    // they're in FOOTNOTE_LIST_SELECTORS for standardization, not removal.
    // Wikipedia references are useful content (citations, DOIs, etc.).
    "[data-optimizely=\"related-articles-section\" i]",
    "[data-orientation=\"vertical\"]",
    // GitHub sticky headers
    ".gh-header-sticky",
    "[data-testid=\"issue-metadata-sticky\"]",
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

/// Patterns that contain raw regex (negative lookbehinds) and must NOT
/// be escaped before inclusion in the combined pattern.
const RAW_REGEX_PATTERNS: &[&str] = &[r"(?<!main-)access-wall", r"(?<!h[1-6]-)related"];

const PARTIAL_PATTERNS: &[&str] = &[
    "a-statement",
    // "(?<!main-)access-wall" handled as raw regex in RAW_REGEX_PATTERNS
    "activitypub",
    "actioncall",
    "addcomment",
    "addtoany",
    "advert",
    "adlayout",
    "ad-tldr",
    "ad-placement",
    "ads-container",
    "_ad_",
    "AdBlock_",
    "AdUnit",
    "after_content",
    "after_main_article",
    "afterpost",
    "allterms",
    "-alert-",
    "alert-box",
    "_archive",
    "around-the-web",
    "aroundpages",
    "article-author",
    "article-badges",
    "article-banner",
    "article-bottom-section",
    "article-bottom",
    "article-category",
    "article-card",
    "article-citation",
    "article__copy",
    "article_date",
    "article-date",
    "article-end ",
    "article_header",
    "article-header",
    "article__header",
    "article__hero",
    "article__info",
    "article-info",
    "article-meta",
    "article_meta",
    "article__meta",
    "articlename",
    "article-subject",
    "article_subject",
    "article-snippet",
    "article-separator",
    "article--share",
    "article-share",
    "article--topics",
    "article-tools",
    "articletags",
    "article-tags",
    "article_tags",
    "articletitle",
    "article-title",
    "article_title",
    "articletopics",
    "article-topics",
    "article-actions",
    "article--lede",
    "articlewell",
    "associated-people",
    "ambient-video__button",
    "audio-card",
    "author-bio",
    "author-box",
    "author-info",
    "author_info",
    "authorm",
    "author-mini-bio",
    "author-name",
    "author-publish-info",
    "authored-by",
    "avatar",
    "back-to-top",
    "backlink_container",
    "backlinks-section",
    "bio-block",
    "biobox",
    "blog-pager",
    "bookmark-",
    "-bookmark",
    "bottominfo",
    "bottomnav",
    "bottom-of-article",
    "bottom-wrapper",
    "brand-bar",
    "bcrumb",
    "breadcrumb",
    "brdcrumb",
    "button-wrapper",
    "buttons-container",
    "btn-",
    "-btn",
    "byline",
    "captcha",
    "card-text",
    "card-media",
    "card-post",
    "carouselcontainer",
    "carousel-container",
    "cat_header",
    "catlinks",
    "_categories",
    "card-author",
    "card-content",
    "chapter-list",
    "collections",
    "comments",
    "-comment",
    "commentbox",
    "comment-button",
    "commentcomp",
    "comment-content",
    "comment-count",
    "comment-form",
    "comment-number",
    "comment-respond",
    "comment-thread",
    "comment-wrap",
    "complementary",
    "consent",
    "contact-",
    "content-card",
    "copycontent",
    "content-topics",
    "contentpromo",
    "context-bar",
    "context-widget",
    "core-collateral",
    "cover-image",
    "cover-photo",
    "cover-wrap",
    "created-date",
    "creative-commons_",
    "c-subscribe",
    "_cta",
    "-cta",
    "cta-",
    "cta_",
    "current-issue",
    "custom-list-number",
    "dateline",
    "dateheader",
    "date-header",
    "date-pub",
    "disclaimer",
    "disclosure",
    "discussion",
    "discuss_",
    "-dismiss",
    "disqus",
    "donate",
    "donation",
    "dropdown",
    "editorial_contact",
    "editorial-contact",
    "element-invisible",
    "elementor-shortcode",
    "eletters",
    "emailsignup",
    "emoji-bar",
    "engagement-widget",
    "enhancement-",
    "entry-author-info",
    "entry-categories",
    "entry-date",
    "entry-title",
    "entry-utility",
    "-error",
    "error-",
    "eyebrow",
    "expand-reduce",
    "external-anchor",
    "externallinkembedwrapper",
    "extra-services",
    "extra-title",
    "facebook",
    "fancy-box",
    "favorite",
    "featured-content",
    "feature_feed",
    "feedback",
    "feed-links",
    "field-site-sections",
    "fixheader",
    "floating-vid",
    "follower",
    "footer",
    "footnote-back",
    "footnoteback",
    "form-group",
    "for-you",
    "frontmatter",
    "further-reading",
    "fullbleedheader",
    "gallery-count",
    "gated-",
    "gh-feed",
    "gist-meta",
    "goog-",
    "graph-view",
    "hamburger",
    "header_logo",
    "header-logo",
    "header-pattern",
    "hero-list",
    "hide-for-print",
    "hide-print",
    "hide-when-no-script",
    "hidden-print",
    "hidden-sidenote",
    "hidden-accessibility",
    "home-link",
    "infoline",
    "inline-topic",
    "instacartIntegration",
    "interlude",
    "interaction",
    "itemendrow",
    "intro-date",
    "invisible",
    "jp-no-solution",
    "jp-relatedposts",
    "jswarning",
    "js-warning",
    "jumplink",
    "jumpto",
    "jump-to-",
    "js-skip-to-content",
    "keepreading",
    "keep-reading",
    "keep_reading",
    "keyword_wrap",
    "kicker",
    "labstab",
    "-labels",
    "language-name",
    "lastupdated",
    "latest-content",
    "-ledes-",
    "-license",
    "license-",
    "lightbox-popup",
    "like-button",
    "link-box",
    "links-grid",
    "links-title",
    "listing-dynamic-terms",
    "list-tags",
    "listinks",
    "loading",
    "loa-info",
    "logo_container",
    "ltx_role_refnum",
    "ltx_tag_bibitem",
    "ltx_error",
    "masthead",
    "marketing",
    "media-inquiry",
    "-menu",
    "menu-",
    "metadata",
    "meta-bottom",
    "meta-date",
    "meta-row",
    "might-like",
    "minibio",
    "more-about",
    "mod-paywall",
    "_modal",
    "-modal",
    "more-",
    "morenews",
    "morestories",
    "more_wrapper",
    "most-read",
    "move-helper",
    "mw-editsection",
    "mw-cite-backlink",
    "mw-indicators",
    "mw-jump-link",
    "nav-",
    "nav_",
    "navigation-post",
    "next-",
    "next_prev",
    "no-script",
    "newsgallery",
    "news-story-title",
    "newsletter_",
    "newsletterbanner",
    "newslettercontainer",
    "newsletter-form",
    "newsletter-signup",
    "newslettersignup",
    "newsletterwidget",
    "newsletterwrapper",
    "not-found",
    "notessection",
    "nomobile",
    "noprint",
    "onward-journey",
    "open-slideshow",
    "originally-published",
    "other-blogs",
    "outline-view",
    "pagehead",
    "page-header",
    "page-title",
    "paywall_message",
    "-partners",
    "permission-",
    "plea",
    "popular",
    "popup_links",
    "pop_stories",
    "pop-up",
    "post__author",
    "post-author",
    "post-bottom",
    "post__category",
    "postcomment",
    "postdate",
    "post-date",
    "post_date",
    "post-details",
    "post-feeds",
    "postinfo",
    "post-info",
    "post_info",
    "post-inline-date",
    "post-links",
    "postlist",
    "post_list",
    "post_meta",
    "post-meta",
    "postmeta",
    "post_more",
    "postnavi",
    "post-navigation",
    "postpath",
    "post-preview",
    "postsnippet",
    "post_snippet",
    "post-snippet",
    "post-subject",
    "posttax",
    "post-tax",
    "post_tax",
    "posttag",
    "post-tag",
    "post_time",
    "posttitle",
    "post-title",
    "post_title",
    "post__title",
    "post-ufi-button",
    "prev-post",
    "prevnext",
    "prev_next",
    "prev-next",
    "previousnext",
    "press-inquiries",
    "print-none",
    "print-header",
    "print:hidden",
    "privacy-notice",
    "privacy-settings",
    "profile",
    "promo_article",
    "promo-bar",
    "promo-box",
    "pubdate",
    "pub_date",
    "pub-date",
    "publish_date",
    "publish-date",
    "publication-date",
    "publicationName",
    "qr-code",
    "qr_code",
    "quick_up",
    "_rail",
    "ratingssection",
    "read_also",
    "readmore",
    "read-next",
    "read_next",
    "read_time",
    "read-time",
    "reading_time",
    "reading-time",
    "reading-list",
    "recent-",
    "recent-articles",
    "recentpost",
    "recent_post",
    "recent-post",
    "recommend",
    "redirectedfrom",
    "recirc",
    "register",
    // "(?<!h[1-6]-)related" handled as raw regex in RAW_REGEX_PATTERNS
    "relevant",
    "reversefootnote",
    "robots-nocontent",
    "_rss",
    "rss-link",
    "screen-reader-text",
    "scroll_to",
    "scroll-to",
    "_search",
    "-search",
    "section-nav",
    "series-banner",
    "share-box",
    "sharedaddy",
    "share-icons",
    "sharelinks",
    "share-post",
    "share-print",
    "share-section",
    "sharing_",
    "shariff-",
    "show-for-print",
    "sidebartitle",
    "sidebar-content",
    "sidebar-wrapper",
    "sideitems",
    "sidebar-author",
    "sidebar-item",
    "side-box",
    "side-logo",
    "sign-in-gate",
    "similar-",
    "similar_",
    "similars-",
    "site-index",
    "site-header",
    "siteheader",
    "site-logo",
    "site-name",
    "site-wordpress",
    "skip-content",
    "skip-to-content",
    "skip-link",
    "c-skip-link",
    "_skip-link",
    "-slider",
    "slug-wrap",
    "social-author",
    "social-button",
    "social-shar",
    "social-date",
    "speechify-ignore",
    "speedbump",
    "sponsor",
    "springercitation",
    "sr-only",
    "_stats",
    "story-date",
    "story-navigation",
    "storyreadtime",
    "storysmall",
    "storypublishdate",
    "subject-label",
    "subhead",
    "submenu",
    "-subscribe-",
    "subscriber-drive",
    "subscription-",
    "_tags",
    "tags__item",
    "tag_list",
    "tag-list",
    "tag-module",
    "taxonomy",
    "table-of-contents",
    "tblc",
    "tabs-",
    "terminaltout",
    "time-rubric",
    "timestamp",
    "time-read",
    "time-to-read",
    "tip_off",
    "-ticker",
    "tiptout",
    "-tout-",
    "toc-container",
    "toggle-caption",
    "tooltip-content",
    "topbar",
    "subnavbar",
    "topic-authors",
    "topic-footer",
    "topic-list",
    "topic-subnav",
    "top-wrapper",
    "tree-item",
    "trending",
    "trust-feat",
    "trust-badge",
    "trust-project",
    "chakra-badge",
    "twitter",
    "twiblock",
    "u-hide",
    "upsell",
    "viewbottom",
    "view-language",
    "yarpp-related",
    "visually-hidden",
    "welcomebox",
    "widget_pages",
    "w-form-done",
    "w-form-fail",
];

/// CSS selectors for inline footnote references.
pub const FOOTNOTE_INLINE_REFERENCES: &[&str] = &[
    "sup.reference",
    "cite.ltx_cite",
    "sup[id^=\"fnr\"]",
    "span[id^=\"fnr\"]",
    "span[class*=\"footnote_ref\"]",
    "span[class*=\"footnote-ref\"]",
    "span.footnote-link",
    "a.citation",
    "a[id^=\"ref-link\"]",
    "a[href^=\"#fn\"]",
    "a[href^=\"#cite\"]",
    "a[href^=\"#reference\"]",
    "a[href^=\"#footnote\"]",
    "a[href^=\"#r\"]",
    "a[href^=\"#b\"]",
    "a[href*=\"cite_note\"]",
    "a[href*=\"cite_ref\"]",
    "a.footnote-anchor",
    "span.footnote-hovercard-target a",
    "a[role=\"doc-biblioref\"]",
    "a[id^=\"fnref\"]",
    "a[id^=\"ref-link\"]",
    "sup.footnoteref",
    "sup[data-fn] > a[href^=\"#\"]",
    "sup[id^=\"ftnt_ref\"] a[href^=\"#ftnt\"]",
];

/// CSS selectors for footnote list containers.
pub const FOOTNOTE_LIST_SELECTORS: &[&str] = &[
    "div.footnote ol",
    "div.footnotes ol",
    "div[role=\"doc-endnotes\"]",
    "div[role=\"doc-footnotes\"]",
    "ol.footnotes-list",
    "ol.footnotes",
    "ol.references",
    "ol[class*=\"article-references\"]",
    "section.footnotes ol",
    "section[role=\"doc-endnotes\"]",
    "section[role=\"doc-footnotes\"]",
    "section[role=\"doc-bibliography\"]",
    "ul.footnotes-list",
    "ul.ltx_biblist",
    "div.footnote[data-component-name=\"FootnoteToDOM\"]",
    "div.footnotes-footer",
    "div.footnote-definitions",
    "ol.wp-block-footnotes",
    "#footnotes",
];

/// CSS selectors for content elements that should be preserved.
pub const CONTENT_ELEMENT_SELECTOR: &[&str] = &[
    "math",
    "[data-mathml]",
    ".katex",
    ".katex-mathml",
    ".katex-display",
    ".MathJax",
    ".MathJax_Display",
    ".MathJax_SVG",
    "mjx-container",
    "pre",
    "code",
    "table",
    "img",
    "picture",
    "video",
    "blockquote",
    "figure",
];

/// Pre-compiled CSS selectors from `EXACT_SELECTORS`, paired with
/// the original string for debug output. Built once and reused
/// across all cleanup passes.
pub static COMPILED_EXACT_SELECTORS: LazyLock<Vec<(&str, Selector)>> = LazyLock::new(|| {
    EXACT_SELECTORS
        .iter()
        .filter_map(|s| Selector::parse(s).ok().map(|sel| (*s, sel)))
        .collect()
});

/// Pre-compiled CSS selectors for footnote inline references.
pub static COMPILED_FOOTNOTE_INLINE: LazyLock<Vec<Selector>> = LazyLock::new(|| {
    FOOTNOTE_INLINE_REFERENCES
        .iter()
        .filter_map(|s| Selector::parse(s).ok())
        .collect()
});

/// Pre-compiled CSS selectors for footnote list containers.
pub static COMPILED_FOOTNOTE_LISTS: LazyLock<Vec<Selector>> = LazyLock::new(|| {
    FOOTNOTE_LIST_SELECTORS
        .iter()
        .filter_map(|s| Selector::parse(s).ok())
        .collect()
});

/// Pre-compiled CSS selectors for content element preservation.
pub static COMPILED_CONTENT_ELEMENTS: LazyLock<Vec<Selector>> = LazyLock::new(|| {
    CONTENT_ELEMENT_SELECTOR
        .iter()
        .filter_map(|s| Selector::parse(s).ok())
        .collect()
});

/// Fast-path regex using the standard `regex` crate (no lookbehind
/// support). Matches all `PARTIAL_PATTERNS` plus the plain substrings
/// "access-wall" and "related" (without lookbehind guards). Used as a
/// pre-filter before the slower `fancy_regex`.
pub static FAST_PARTIAL_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    let escaped: Vec<String> = PARTIAL_PATTERNS.iter().map(|p| regex::escape(p)).collect();
    let mut all: Vec<String> = escaped;
    // Include the raw lookbehind patterns as plain substring matches
    all.push("access-wall".to_string());
    all.push("related".to_string());
    let combined = format!("(?i){}", all.join("|"));
    regex::Regex::new(&combined).unwrap_or_else(|_| {
        #[expect(clippy::unwrap_used)]
        regex::Regex::new(r"^\b$").unwrap()
    })
});

/// Compiled regex matching any partial pattern as a substring,
/// case-insensitive. Uses `fancy_regex` for lookbehind support.
pub static PARTIAL_REGEX: LazyLock<Regex> = LazyLock::new(build_partial_pattern);

/// Check if a value matches any partial pattern. Uses the fast regex
/// as a pre-filter, falling back to `fancy_regex` only when the match
/// could be a lookbehind-guarded pattern.
#[must_use]
pub fn matches_partial(val: &str) -> bool {
    // Fast path: no match at all
    if !FAST_PARTIAL_REGEX.is_match(val) {
        return false;
    }
    // If the value contains the lookbehind-guarded substrings,
    // verify with the full fancy_regex
    if val.contains("access-wall") || val.contains("related") {
        return PARTIAL_REGEX.is_match(val).unwrap_or(false);
    }
    // Fast regex matched and no lookbehind patterns involved
    true
}

/// Builds a compiled regex from all partial patterns, joined with `|`
/// and compiled with the case-insensitive flag.
///
/// Plain patterns are escaped with `fancy_regex::escape()`. The two
/// patterns using regex lookbehind syntax are included verbatim.
///
/// # Panics
///
/// Panics if the fallback regex literal is somehow invalid. This cannot
/// happen in practice because all patterns are compile-time constants.
#[must_use]
pub fn build_partial_pattern() -> Regex {
    let escaped: Vec<String> = PARTIAL_PATTERNS
        .iter()
        .map(|p| fancy_regex::escape(p).into_owned())
        .collect();

    let mut all_parts: Vec<&str> = Vec::with_capacity(escaped.len() + RAW_REGEX_PATTERNS.len());
    for part in &escaped {
        all_parts.push(part);
    }
    for raw in RAW_REGEX_PATTERNS {
        all_parts.push(raw);
    }

    let combined = format!("(?i){}", all_parts.join("|"));

    let Ok(re) = Regex::new(&combined) else {
        // All patterns are known at compile time and are valid after
        // escaping, so this branch is unreachable in practice.
        // Return a regex that never matches as a safe fallback.
        #[expect(clippy::unwrap_used)]
        return Regex::new(r"^\b$").unwrap();
    };
    re
}

#[cfg(test)]
mod tests {
    use super::*;

    #[expect(clippy::unwrap_used)]
    #[test]
    fn partial_regex_compiles() {
        let re = build_partial_pattern();
        assert!(re.is_match("advert").unwrap());
        assert!(re.is_match("ADVERT").unwrap());
        assert!(re.is_match("my-sidebar-content-widget").unwrap());
    }

    #[expect(clippy::unwrap_used)]
    #[test]
    fn partial_regex_rejects_unrelated() {
        let re = build_partial_pattern();
        assert!(!re.is_match("paragraph").unwrap());
    }

    #[expect(clippy::unwrap_used)]
    #[test]
    fn lookbehind_access_wall() {
        let re = build_partial_pattern();
        assert!(re.is_match("access-wall").unwrap());
        assert!(!re.is_match("main-access-wall").unwrap());
    }

    #[expect(clippy::unwrap_used)]
    #[test]
    fn lookbehind_related() {
        let re = build_partial_pattern();
        assert!(re.is_match("related").unwrap());
        assert!(!re.is_match("h1-related").unwrap());
        assert!(!re.is_match("h3-related").unwrap());
        assert!(re.is_match("also-related").unwrap());
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

    #[expect(clippy::unwrap_used)]
    #[test]
    fn lazy_static_works() {
        assert!(PARTIAL_REGEX.is_match("newsletter-signup").unwrap());
    }

    #[test]
    fn footnote_selectors_not_empty() {
        assert!(!FOOTNOTE_INLINE_REFERENCES.is_empty());
        assert!(!FOOTNOTE_LIST_SELECTORS.is_empty());
    }

    #[test]
    fn content_element_selector_not_empty() {
        assert!(!CONTENT_ELEMENT_SELECTOR.is_empty());
    }
}
