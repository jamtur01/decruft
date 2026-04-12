use serde::{Deserialize, Serialize};

/// Options for configuring the decruft extraction pipeline.
#[derive(Debug, Clone)]
#[non_exhaustive]
#[expect(clippy::struct_excessive_bools)]
pub struct DecruftOptions {
    /// URL of the page being parsed (for resolving relative URLs).
    pub url: Option<String>,
    /// Enable debug logging and include removal details in output.
    pub debug: bool,
    /// Remove elements matching exact CSS selectors.
    pub remove_exact_selectors: bool,
    /// Remove elements matching partial class/id patterns.
    pub remove_partial_selectors: bool,
    /// Remove all images from output.
    pub remove_images: bool,
    /// Remove elements hidden via CSS.
    pub remove_hidden_elements: bool,
    /// Remove low-scoring non-content blocks.
    pub remove_low_scoring: bool,
    /// Remove small images (< 33px).
    pub remove_small_images: bool,
    /// Standardize heading levels, code blocks, etc.
    pub standardize: bool,
    /// Remove content patterns (bylines, read time, etc.).
    pub remove_content_patterns: bool,
    /// CSS selector override for content root.
    pub content_selector: Option<String>,
    /// Convert output to Markdown.
    pub markdown: bool,
    /// Include Markdown alongside HTML content.
    pub separate_markdown: bool,
    /// Include replies/comments in extracted content.
    pub include_replies: bool,
}

impl Default for DecruftOptions {
    fn default() -> Self {
        Self {
            url: None,
            debug: false,
            remove_exact_selectors: true,
            remove_partial_selectors: true,
            remove_images: false,
            remove_hidden_elements: true,
            remove_low_scoring: true,
            remove_small_images: true,
            standardize: true,
            remove_content_patterns: true,
            content_selector: None,
            markdown: false,
            separate_markdown: false,
            include_replies: true,
        }
    }
}

/// Result of the decruft extraction pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DecruftResult {
    /// Cleaned HTML content.
    pub content: String,
    /// Page title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Page description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Domain name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    /// Favicon URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,
    /// Primary image URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    /// Content language.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Parse time in milliseconds.
    pub parse_time_ms: u64,
    /// Publication date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<String>,
    /// Last modified date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>,
    /// Author name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Site name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site: Option<String>,
    /// Canonical URL of the page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_url: Option<String>,
    /// Keywords/tags associated with the content.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
    /// Content type (e.g., "article", "website", "video").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    /// Markdown version of content (when `markdown` or `separate_markdown` is enabled).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_markdown: Option<String>,
    /// Word count of extracted content.
    pub word_count: usize,
    /// Schema.org data if found.
    pub schema_org_data: Option<serde_json::Value>,
    /// All meta tags found on the page. Only populated when debug mode is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_tags: Option<Vec<MetaTag>>,
    /// Which site-specific extractor produced this result (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extractor_type: Option<String>,
    /// Debug information (only present when debug mode is enabled).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug: Option<DebugInfo>,
}

/// A meta tag from the page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaTag {
    pub name: Option<String>,
    pub property: Option<String>,
    pub content: Option<String>,
}

/// Debug information about the extraction process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugInfo {
    /// CSS selector path of the chosen content element.
    pub content_selector: String,
    /// List of elements that were removed.
    pub removals: Vec<Removal>,
}

/// A record of a removed element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Removal {
    pub step: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub text: String,
}

/// Metadata extracted from the page (internal representation).
#[derive(Debug, Clone, Default, Serialize)]
pub(crate) struct Metadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub domain: Option<String>,
    pub favicon: Option<String>,
    pub image: Option<String>,
    pub language: Option<String>,
    pub published: Option<String>,
    pub modified: Option<String>,
    pub author: Option<String>,
    pub site_name: Option<String>,
    pub canonical_url: Option<String>,
    pub keywords: Vec<String>,
    pub content_type: Option<String>,
}
