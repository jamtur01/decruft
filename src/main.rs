#![allow(clippy::print_stderr)]

use std::io::Read;

use clap::Parser;

/// Extract clean, readable content from web pages.
///
/// Input can be a file path, URL, or - for stdin (default).
/// When given a URL, the page is fetched automatically.
///
/// Examples:
///   decruft page.html
///   decruft <https://example.com/article>
///   cat page.html | decruft --url <https://example.com>
///   decruft page.html -f markdown
///   decruft page.html -f text
#[derive(Parser, Debug)]
#[command(
    name = "decruft",
    version,
    about,
    after_help = "Advanced options:\n  \
    --no-exact-selectors    Disable exact CSS selector removal\n  \
    --no-partial-selectors  Disable partial class/id pattern removal\n  \
    --no-hidden             Disable hidden element removal\n  \
    --no-scoring            Disable content scoring removal\n  \
    --no-patterns           Disable content pattern removal\n  \
    --no-standardize        Disable content standardization"
)]
#[expect(clippy::struct_excessive_bools)]
struct Cli {
    /// File path, URL, or - for stdin.
    /// URLs (starting with http:// or https://) are fetched automatically.
    #[arg(default_value = "-")]
    input: String,

    /// URL of the page (for resolving relative URLs and metadata).
    /// Inferred automatically when input is a URL.
    #[arg(short, long)]
    url: Option<String>,

    /// Output format.
    #[arg(short, long, default_value = "json")]
    format: OutputFormat,

    /// CSS selector to override content root detection.
    #[arg(short = 's', long)]
    selector: Option<String>,

    /// Enable debug mode (include removal details in JSON output).
    #[arg(short, long)]
    debug: bool,

    /// Remove all images from output.
    #[arg(long)]
    no_images: bool,

    /// Exclude replies/comments from extractor output.
    #[arg(long = "no-replies")]
    no_replies: bool,

    // ── Advanced: disable pipeline stages ──
    #[arg(long, hide = true)]
    no_exact_selectors: bool,
    #[arg(long, hide = true)]
    no_partial_selectors: bool,
    #[arg(long, hide = true)]
    no_hidden: bool,
    #[arg(long, hide = true)]
    no_scoring: bool,
    #[arg(long, hide = true)]
    no_patterns: bool,
    #[arg(long, hide = true)]
    no_standardize: bool,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum OutputFormat {
    Html,
    Json,
    Text,
    Markdown,
}

fn main() {
    let cli = Cli::parse();

    let is_url = cli.input.starts_with("http://") || cli.input.starts_with("https://");

    let (html, effective_url) = if is_url {
        let url = cli.input.clone();
        let content = fetch_url(&url);
        (content, Some(url))
    } else if cli.input == "-" {
        let mut buf = String::new();
        if let Err(e) = std::io::stdin().read_to_string(&mut buf) {
            eprintln!("Error reading stdin: {e}");
            std::process::exit(1);
        }
        (buf, cli.url.clone())
    } else {
        match std::fs::read_to_string(&cli.input) {
            Ok(s) => (s, cli.url.clone()),
            Err(e) => {
                eprintln!("Error reading {}: {e}", cli.input);
                std::process::exit(1);
            }
        }
    };

    let mut options = decruft::DecruftOptions::default();
    options.url = effective_url;
    options.debug = cli.debug;
    options.remove_exact_selectors = !cli.no_exact_selectors;
    options.remove_partial_selectors = !cli.no_partial_selectors;
    options.remove_images = cli.no_images;
    options.remove_hidden_elements = !cli.no_hidden;
    options.remove_low_scoring = !cli.no_scoring;
    options.standardize = !cli.no_standardize;
    options.remove_content_patterns = !cli.no_patterns;
    options.content_selector = cli.selector;
    options.markdown = matches!(cli.format, OutputFormat::Markdown);
    options.separate_markdown = matches!(cli.format, OutputFormat::Markdown);
    options.include_replies = !cli.no_replies;

    let result = decruft::parse(&html, &options);

    match cli.format {
        OutputFormat::Json => match serde_json::to_string_pretty(&result) {
            Ok(json) => write_stdout(&json),
            Err(e) => {
                eprintln!("Error serializing result: {e}");
                std::process::exit(1);
            }
        },
        OutputFormat::Html => write_stdout(&result.content),
        OutputFormat::Markdown => {
            let md = result
                .content_markdown
                .as_deref()
                .unwrap_or(&result.content);
            write_stdout(md);
        }
        OutputFormat::Text => {
            let text = decruft::strip_html_tags(&result.content);
            write_stdout(text.trim());
        }
    }
}

fn write_stdout(s: &str) {
    use std::io::Write;
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    if let Err(e) = writeln!(handle, "{s}") {
        if e.kind() == std::io::ErrorKind::BrokenPipe {
            std::process::exit(0);
        }
        eprintln!("Error writing output: {e}");
        std::process::exit(1);
    }
}

fn fetch_url(url: &str) -> String {
    if let Some(body) = decruft::http::fetch_page(url) {
        body
    } else {
        eprintln!("Error fetching {url}");
        std::process::exit(1);
    }
}
