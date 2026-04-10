#!/usr/bin/env bash
set -euo pipefail

# Compare decruft vs defuddle output across diverse site types
# Tests json, html, text, and markdown output formats
# Includes performance comparison

DECRUFT="./target/release/decruft"
OUTDIR="/tmp/decruft-compare"
mkdir -p "$OUTDIR"

cargo build --release 2>/dev/null

URLS=(
    # News sites
    "https://www.bbc.com/news/articles/cp3l4yk5rlgo"
    "https://apnews.com/article/b50e12b7e86e3bce8f4d43f7f0a0e6b5"
    # Personal blog (simple HTML, no JS)
    "https://www.paulgraham.com/superlinear.html"
    # Technical documentation
    "https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html"
    # Wikipedia
    "https://en.wikipedia.org/wiki/Rust_(programming_language)"
    # GitHub issue
    "https://github.com/kepano/defuddle/issues/56"
    # Substack
    "https://www.lennysnewsletter.com/p/how-to-build-a-billion-dollar-ai"
    # Academic
    "https://arxiv.org/abs/2401.00001"
    # Stack Overflow blog
    "https://stackoverflow.blog/2020/07/01/nobody-has-to-teach-you-anything/"
    # MDN
    "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array"
)

PASS=0
ISSUES=0
SKIP=0
TOTAL_DC_MS=0
TOTAL_DF_MS=0
SITE_COUNT=0

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         decruft vs defuddle — cross-site comparison         ║"
echo "╠══════════════════════════════════════════════════════════════╣"
echo ""

for url in "${URLS[@]}"; do
    name=$(echo "$url" | sed 's|https\?://||;s|/|_|g;s|[^a-zA-Z0-9_.-]||g' | cut -c1-60)
    echo "--- $name ---"
    echo "    $url"

    # Fetch HTML once
    html_file="$OUTDIR/${name}.html"
    if [ ! -f "$html_file" ]; then
        curl -sL --max-time 20 -o "$html_file" "$url" 2>/dev/null || {
            echo "    SKIP: fetch failed"
            SKIP=$((SKIP + 1))
            echo ""
            continue
        }
    fi

    filesize=$(wc -c < "$html_file" | tr -d ' ')
    echo "    HTML: ${filesize} bytes"

    # Run decruft (all formats)
    dc_json="$OUTDIR/${name}.dc.json"
    dc_html="$OUTDIR/${name}.dc.html"
    dc_text="$OUTDIR/${name}.dc.txt"
    dc_md="$OUTDIR/${name}.dc.md"

    dc_start=$(python3 -c "import time; print(int(time.time()*1000))")
    $DECRUFT "$html_file" --url "$url" -f json > "$dc_json" 2>/dev/null || { echo "    FAIL: decruft json"; ISSUES=$((ISSUES+1)); echo ""; continue; }
    $DECRUFT "$html_file" --url "$url" -f html > "$dc_html" 2>/dev/null
    $DECRUFT "$html_file" --url "$url" -f text > "$dc_text" 2>/dev/null
    $DECRUFT "$html_file" --url "$url" --markdown -f json > "$dc_md" 2>/dev/null
    dc_end=$(python3 -c "import time; print(int(time.time()*1000))")
    dc_ms=$((dc_end - dc_start))

    # Run defuddle (json)
    df_json="$OUTDIR/${name}.df.json"
    df_start=$(python3 -c "import time; print(int(time.time()*1000))")
    npx defuddle parse --json "$url" 2>/dev/null > "$df_json" || {
        # Fallback: parse from file
        npx defuddle parse --json "$html_file" 2>/dev/null > "$df_json" || {
            echo "    SKIP: defuddle failed"
            SKIP=$((SKIP + 1))
            echo ""
            continue
        }
    }
    df_end=$(python3 -c "import time; print(int(time.time()*1000))")
    df_ms=$((df_end - df_start))

    TOTAL_DC_MS=$((TOTAL_DC_MS + dc_ms))
    TOTAL_DF_MS=$((TOTAL_DF_MS + df_ms))
    SITE_COUNT=$((SITE_COUNT + 1))

    # Compare
    python3 - "$dc_json" "$df_json" "$dc_html" "$dc_text" "$dc_md" "$dc_ms" "$df_ms" << 'PYEOF'
import json, sys, re, os

dc_file, df_file = sys.argv[1], sys.argv[2]
dc_html_f, dc_text_f, dc_md_f = sys.argv[3], sys.argv[4], sys.argv[5]
dc_ms, df_ms = int(sys.argv[6]), int(sys.argv[7])

with open(dc_file) as f:
    dc = json.load(f)
with open(df_file) as f:
    df = json.load(f)

issues = []

# Metadata comparison
dc_wc = dc.get('word_count', 0)
df_wc = df.get('wordCount', 0)

if dc.get('title', '') != df.get('title', ''):
    issues.append(f"title: '{dc.get('title','')}' vs '{df.get('title','')}'")

if df_wc > 0:
    ratio = dc_wc / df_wc
    if ratio < 0.5 or ratio > 2.0:
        issues.append(f"word_count: {dc_wc} vs {df_wc} ({ratio:.1f}x)")

if 'data-decruft-' in dc.get('content', ''):
    issues.append("leaked internal attributes")

if dc_wc == 0 and df_wc > 50:
    issues.append("EMPTY extraction")

# Output format checks
dc_html_content = open(dc_html_f).read() if os.path.exists(dc_html_f) else ""
dc_text_content = open(dc_text_f).read() if os.path.exists(dc_text_f) else ""

with open(dc_md_f) as f:
    dc_md_data = json.load(f)
dc_md_content = dc_md_data.get('content_markdown', '') or dc_md_data.get('content', '')

format_issues = []
if dc_html_content and '<' not in dc_html_content:
    format_issues.append("html output has no tags")
if dc_text_content and '<p>' in dc_text_content:
    format_issues.append("text output contains HTML tags")
if dc_md_content and dc_md_content.startswith('<'):
    format_issues.append("markdown output looks like HTML")
if not dc_html_content.strip():
    format_issues.append("html output empty")
if not dc_text_content.strip():
    format_issues.append("text output empty")

if format_issues:
    issues.append(f"format: {'; '.join(format_issues)}")

# Print results
perf = f"perf: {dc_ms}ms vs {df_ms}ms"
if dc_ms > 0 and df_ms > 0:
    perf += f" ({dc_ms/df_ms:.1f}x)" if dc_ms > df_ms else f" ({df_ms/dc_ms:.1f}x faster)"

if issues:
    print(f"    ISSUES: {', '.join(issues)}")
else:
    print(f"    OK: words={dc_wc}/{df_wc} title='{dc.get('title','')[:40]}'")

print(f"    {perf}")
print(f"    formats: json={os.path.getsize(dc_file)}B html={len(dc_html_content)}B text={len(dc_text_content)}B md={len(dc_md_content)}B")

sys.exit(1 if any('EMPTY' in i for i in issues) else 0)
PYEOF

    result=$?
    if [ $result -eq 0 ]; then
        PASS=$((PASS + 1))
    else
        ISSUES=$((ISSUES + 1))
    fi
    echo ""
done

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  Results: $PASS pass, $ISSUES issues, $SKIP skipped"
if [ $SITE_COUNT -gt 0 ]; then
    dc_avg=$((TOTAL_DC_MS / SITE_COUNT))
    df_avg=$((TOTAL_DF_MS / SITE_COUNT))
    echo "║  Avg time: decruft ${dc_avg}ms, defuddle ${df_avg}ms"
fi
echo "╚══════════════════════════════════════════════════════════════╝"
