#!/usr/bin/env bash
set -euo pipefail

# Compare decruft vs defuddle across diverse sites and ALL output formats
# Usage: ./tests/compare_sites.sh

DECRUFT="./target/release/decruft"
OUTDIR="/tmp/decruft-compare"
mkdir -p "$OUTDIR"

cargo build --release 2>/dev/null

URLS=(
    # News
    "https://www.bbc.com/news/articles/cp3l4yk5rlgo"
    # Personal blog
    "https://www.paulgraham.com/superlinear.html"
    # Technical docs
    "https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html"
    # Wikipedia
    "https://en.wikipedia.org/wiki/Rust_(programming_language)"
    # GitHub issue
    "https://github.com/kepano/defuddle/issues/56"
    # Substack
    "https://www.lennysnewsletter.com/p/how-to-build-a-billion-dollar-ai"
    # Academic
    "https://arxiv.org/abs/2401.00001"
    # MDN docs
    "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array"
    # Hacker News
    "https://news.ycombinator.com/item?id=42338514"
)

PASS=0
ISSUES=0
SKIP=0
TOTAL_DC_MS=0
TOTAL_DF_MS=0
SITE_COUNT=0

echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║          decruft vs defuddle — cross-site comparison            ║"
echo "║          JSON + HTML + Markdown + Text                          ║"
echo "╠══════════════════════════════════════════════════════════════════╣"
echo ""

for url in "${URLS[@]}"; do
    name=$(echo "$url" | sed 's|https\?://||;s|/|_|g;s|[^a-zA-Z0-9_.-]||g' | cut -c1-60)
    echo "--- $name ---"
    echo "    $url"

    # Fetch HTML once
    html_file="$OUTDIR/${name}.html"
    if [ ! -f "$html_file" ]; then
        curl -sL --max-time 20 -A "Mozilla/5.0" -o "$html_file" "$url" 2>/dev/null || {
            echo "    SKIP: fetch failed"
            SKIP=$((SKIP + 1))
            echo ""
            continue
        }
    fi

    filesize=$(wc -c < "$html_file" | tr -d ' ')
    if [ "$filesize" -lt 200 ]; then
        echo "    SKIP: page too small (${filesize}B, likely error)"
        SKIP=$((SKIP + 1))
        echo ""
        continue
    fi
    echo "    HTML: ${filesize} bytes"

    # ── Run decruft (all formats) ──
    dc_start=$(python3 -c "import time; print(int(time.time()*1000))")
    $DECRUFT "$html_file" --url "$url" -f json > "$OUTDIR/${name}.dc.json" 2>/dev/null || { echo "    FAIL: decruft"; ISSUES=$((ISSUES+1)); echo ""; continue; }
    $DECRUFT "$html_file" --url "$url" -f html > "$OUTDIR/${name}.dc.html" 2>/dev/null
    $DECRUFT "$html_file" --url "$url" -f text > "$OUTDIR/${name}.dc.text" 2>/dev/null
    $DECRUFT "$html_file" --url "$url" -f markdown > "$OUTDIR/${name}.dc.md" 2>/dev/null
    dc_end=$(python3 -c "import time; print(int(time.time()*1000))")
    dc_ms=$((dc_end - dc_start))

    # ── Run defuddle (all formats) ──
    df_start=$(python3 -c "import time; print(int(time.time()*1000))")
    npx defuddle parse --json "$url" 2>/dev/null > "$OUTDIR/${name}.df.json" || \
        npx defuddle parse --json "$html_file" 2>/dev/null > "$OUTDIR/${name}.df.json" || {
            echo "    SKIP: defuddle failed"
            SKIP=$((SKIP + 1))
            echo ""
            continue
        }
    npx defuddle parse "$url" 2>/dev/null > "$OUTDIR/${name}.df.html" || \
        npx defuddle parse "$html_file" 2>/dev/null > "$OUTDIR/${name}.df.html" || true
    npx defuddle parse --markdown "$url" 2>/dev/null > "$OUTDIR/${name}.df.md" || \
        npx defuddle parse --markdown "$html_file" 2>/dev/null > "$OUTDIR/${name}.df.md" || true
    df_end=$(python3 -c "import time; print(int(time.time()*1000))")
    df_ms=$((df_end - df_start))

    TOTAL_DC_MS=$((TOTAL_DC_MS + dc_ms))
    TOTAL_DF_MS=$((TOTAL_DF_MS + df_ms))
    SITE_COUNT=$((SITE_COUNT + 1))

    # ── Compare all formats ──
    python3 - "$OUTDIR" "$name" "$dc_ms" "$df_ms" << 'PYEOF'
import json, sys, os, re

outdir, name = sys.argv[1], sys.argv[2]
dc_ms, df_ms = int(sys.argv[3]), int(sys.argv[4])

def load_json(path):
    try:
        with open(path) as f:
            return json.load(f)
    except:
        return {}

def load_text(path):
    try:
        with open(path) as f:
            return f.read()
    except:
        return ""

dc = load_json(f"{outdir}/{name}.dc.json")
df = load_json(f"{outdir}/{name}.df.json")
dc_html = load_text(f"{outdir}/{name}.dc.html")
df_html = load_text(f"{outdir}/{name}.df.html")
dc_md = load_text(f"{outdir}/{name}.dc.md")
df_md = load_text(f"{outdir}/{name}.df.md")
dc_text = load_text(f"{outdir}/{name}.dc.text")

issues = []

# ── Metadata comparison ──
dc_wc = dc.get('word_count', 0)
df_wc = df.get('wordCount', 0)

dc_title = dc.get('title', '')
df_title = df.get('title', '')
if dc_title != df_title:
    issues.append(f"title: '{dc_title[:40]}' vs '{df_title[:40]}'")

if df_wc > 50:
    ratio = dc_wc / max(df_wc, 1)
    if ratio < 0.5 or ratio > 2.0:
        issues.append(f"word_count: {dc_wc} vs {df_wc} ({ratio:.1f}x)")

if 'data-decruft-' in dc.get('content', ''):
    issues.append("leaked internal attributes")

# ── HTML format comparison ──
html_issues = []
if dc_html and df_html:
    dc_html_tags = set(re.findall(r'<(\w+)', dc_html))
    df_html_tags = set(re.findall(r'<(\w+)', df_html))
    # Check for major structural differences
    for tag in ['p', 'h1', 'h2', 'blockquote', 'pre', 'code']:
        dc_count = dc_html.lower().count(f'<{tag}')
        df_count = df_html.lower().count(f'<{tag}')
        if dc_count == 0 and df_count > 2:
            html_issues.append(f"missing <{tag}> ({df_count} in defuddle)")
elif not dc_html.strip():
    html_issues.append("empty html output")

if html_issues:
    issues.append(f"html: {'; '.join(html_issues)}")

# ── Markdown format comparison ──
md_issues = []
if dc_md and df_md:
    # Both should have markdown-like content
    dc_has_md = bool(re.search(r'[#*`\[\]>]', dc_md))
    df_has_md = bool(re.search(r'[#*`\[\]>]', df_md))
    if df_has_md and not dc_has_md:
        md_issues.append("no markdown syntax in decruft output")
    if dc_md.startswith('<') and not dc_md.startswith('<!--'):
        md_issues.append("markdown output looks like HTML")
    # Compare word counts
    dc_md_words = len(dc_md.split())
    df_md_words = len(df_md.split())
    if df_md_words > 50 and dc_md_words > 0:
        md_ratio = dc_md_words / df_md_words
        if md_ratio < 0.3 or md_ratio > 3.0:
            md_issues.append(f"md words: {dc_md_words} vs {df_md_words} ({md_ratio:.1f}x)")
elif not dc_md.strip():
    md_issues.append("empty markdown output")

if md_issues:
    issues.append(f"markdown: {'; '.join(md_issues)}")

# ── Text format check ──
if dc_text:
    if '<p>' in dc_text or '<div' in dc_text:
        issues.append("text output contains HTML tags")
elif dc_wc > 0:
    issues.append("empty text output despite content")

# ── Performance ──
if dc_ms > 0 and df_ms > 0:
    if dc_ms < df_ms:
        perf_str = f"decruft {dc_ms}ms, defuddle {df_ms}ms ({df_ms/dc_ms:.1f}x faster)"
    else:
        perf_str = f"decruft {dc_ms}ms, defuddle {df_ms}ms ({dc_ms/df_ms:.1f}x slower)"
else:
    perf_str = f"decruft {dc_ms}ms, defuddle {df_ms}ms"

# ── Report ──
if issues:
    print(f"    ISSUES ({len(issues)}):")
    for i in issues:
        print(f"      - {i}")
else:
    wc_str = f"words={dc_wc}/{df_wc}"
    print(f"    OK: {wc_str} title='{dc_title[:40]}'")

print(f"    perf: {perf_str}")
print(f"    json: {os.path.getsize(f'{outdir}/{name}.dc.json')}B / {os.path.getsize(f'{outdir}/{name}.df.json')}B")
print(f"    html: {len(dc_html)}B / {len(df_html)}B")
print(f"    md:   {len(dc_md)}B / {len(df_md)}B")
print(f"    text: {len(dc_text)}B")

sys.exit(1 if any('EMPTY' in i.upper() for i in issues) else 0)
PYEOF

    result=$?
    if [ $result -eq 0 ]; then
        PASS=$((PASS + 1))
    else
        ISSUES=$((ISSUES + 1))
    fi
    echo ""
done

echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║  Results: $PASS pass, $ISSUES issues, $SKIP skipped"
if [ $SITE_COUNT -gt 0 ]; then
    dc_avg=$((TOTAL_DC_MS / SITE_COUNT))
    df_avg=$((TOTAL_DF_MS / SITE_COUNT))
    echo "║  Avg perf: decruft ${dc_avg}ms, defuddle ${df_avg}ms"
fi
echo "╚══════════════════════════════════════════════════════════════════╝"
