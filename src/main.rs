#![allow(clippy::print_stderr)]

use std::io::Read;

use clap::Parser;

/// Extract clean, readable content from web pages.
#[derive(Parser, Debug)]
#[command(name = "decruft", version, about)]
#[allow(clippy::struct_excessive_bools)]
struct Cli {
    /// URL of the page (used for resolving relative URLs and metadata).
    #[arg(short, long)]
    url: Option<String>,

    /// Path to an HTML file to process. Use - for stdin.
    #[arg(default_value = "-")]
    input: String,

    /// CSS selector to use as the content root.
    #[arg(short = 's', long)]
    selector: Option<String>,

    /// Output format: html, json, text, or markdown.
    #[arg(short, long, default_value = "json")]
    format: OutputFormat,

    /// Enable debug mode (include removal details).
    #[arg(short, long)]
    debug: bool,

    /// Remove all images from output.
    #[arg(long)]
    no_images: bool,

    /// Disable exact selector removal.
    #[arg(long)]
    no_exact_selectors: bool,

    /// Disable partial selector removal.
    #[arg(long)]
    no_partial_selectors: bool,

    /// Disable hidden element removal.
    #[arg(long)]
    no_hidden: bool,

    /// Disable content scoring removal.
    #[arg(long)]
    no_scoring: bool,

    /// Disable content pattern removal.
    #[arg(long)]
    no_patterns: bool,

    /// Disable content standardization.
    #[arg(long)]
    no_standardize: bool,

    /// Convert output to Markdown.
    #[arg(long)]
    markdown: bool,

    /// Fetch URL and process (requires url argument).
    #[arg(short = 'F', long)]
    fetch: bool,
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

    let html = if cli.fetch {
        let Some(ref url) = cli.url else {
            eprintln!("Error: --fetch requires --url");
            std::process::exit(1);
        };
        fetch_url(url)
    } else if cli.input == "-" {
        let mut buf = String::new();
        if let Err(e) = std::io::stdin().read_to_string(&mut buf) {
            eprintln!("Error reading stdin: {e}");
            std::process::exit(1);
        }
        buf
    } else {
        match std::fs::read_to_string(&cli.input) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error reading {}: {e}", cli.input);
                std::process::exit(1);
            }
        }
    };

    let options = decruft::DecruftOptions {
        url: cli.url,
        debug: cli.debug,
        remove_exact_selectors: !cli.no_exact_selectors,
        remove_partial_selectors: !cli.no_partial_selectors,
        remove_images: cli.no_images,
        remove_hidden_elements: !cli.no_hidden,
        remove_low_scoring: !cli.no_scoring,
        remove_small_images: true,
        standardize: !cli.no_standardize,
        remove_content_patterns: !cli.no_patterns,
        content_selector: cli.selector,
        markdown: cli.markdown || matches!(cli.format, OutputFormat::Markdown),
        separate_markdown: false,
    };

    let result = decruft::parse(&html, &options);

    match cli.format {
        OutputFormat::Json => match serde_json::to_string_pretty(&result) {
            Ok(json) => {
                write_stdout(&json);
            }
            Err(e) => {
                eprintln!("Error serializing result: {e}");
                std::process::exit(1);
            }
        },
        OutputFormat::Html => {
            write_stdout(&result.content);
        }
        OutputFormat::Markdown => {
            let md = result
                .content_markdown
                .as_deref()
                .unwrap_or(&result.content);
            write_stdout(md);
        }
        OutputFormat::Text => {
            let text = strip_tags_simple(&result.content);
            write_stdout(text.trim());
        }
    }
}

fn write_stdout(s: &str) {
    use std::io::Write;
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    let _ = writeln!(handle, "{s}");
}

fn strip_tags_simple(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
            result.push(' ');
        } else if !in_tag {
            result.push(ch);
        }
    }
    result
}

fn fetch_url(url: &str) -> String {
    // Minimal HTTP fetch using std - no extra dependencies
    // For production use, users would pipe curl output
    eprintln!("Tip: For fetching, pipe curl output: curl -sL '{url}' | decruft --url '{url}'");
    eprintln!("Attempting basic fetch...");

    let output = std::process::Command::new("curl")
        .args(["-sL", "--max-time", "30", url])
        .output();

    match output {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).into_owned(),
        Ok(out) => {
            eprintln!("curl failed: {}", String::from_utf8_lossy(&out.stderr));
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Failed to run curl: {e}");
            std::process::exit(1);
        }
    }
}
