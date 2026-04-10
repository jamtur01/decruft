#!/usr/bin/env bash
set -euo pipefail

# Compare decruft vs defuddle output across diverse site types.
# Tests all 4 output formats (json, html, markdown, text) and
# reports performance, content differences, and errors with reasons.

DECRUFT="./target/release/decruft"
OUTDIR="/tmp/decruft-compare"
mkdir -p "$OUTDIR"

cargo build --release 2>/dev/null

URLS=(
    # ── News ──
    "https://www.bbc.com/news/articles/cp3l4yk5rlgo"
    # ── Personal blogs ──
    "https://www.paulgraham.com/superlinear.html"
    "https://danluu.com/cocktail-ideas/"
    # ── Technical docs ──
    "https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html"
    "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array"
    # ── Wikipedia ──
    "https://en.wikipedia.org/wiki/Rust_(programming_language)"
    # ── GitHub (extractor) ──
    "https://github.com/kepano/defuddle/issues/56"
    # ── Substack (extractor) ──
    "https://www.lennysnewsletter.com/p/how-to-build-a-billion-dollar-ai"
    # ── Hacker News (extractor) ──
    "https://news.ycombinator.com/item?id=42338514"
    # ── Reddit (extractor) ──
    "https://old.reddit.com/r/rust/comments/1i6gxbr/this_week_in_rust_583/"
    # ── X/Twitter (JS-rendered) ──
    "https://x.com/elikiwen/status/1900575802102243559"
    # ── Academic ──
    "https://arxiv.org/abs/2401.00001"
    # ── Hugo static site ──
    "https://gohugo.io/getting-started/quick-start/"
    # ── Rust blog ──
    "https://blog.rust-lang.org/2025/02/20/Rust-1.85.0.html"
)

PASS=0
ISSUES=0
SKIP=0
TOTAL_DC_MS=0
TOTAL_DF_MS=0
SITE_COUNT=0

echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║       decruft vs defuddle — cross-site format comparison        ║"
echo "║       JSON + HTML + Markdown + Text · ${#URLS[@]} sites                  ║"
echo "╠══════════════════════════════════════════════════════════════════╣"
echo ""

for url in "${URLS[@]}"; do
    name=$(echo "$url" | sed 's|https\?://||;s|/|_|g;s|[^a-zA-Z0-9_.-]||g' | cut -c1-60)
    echo "--- $name ---"
    echo "    $url"

    # ── Fetch HTML ──
    html_file="$OUTDIR/${name}.html"
    if [ ! -f "$html_file" ]; then
        fetch_err=$( curl -sL --max-time 20 \
            -A "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36" \
            -o "$html_file" -w "%{http_code}" "$url" 2>&1 ) || true
        http_code="${fetch_err: -3}"
        if [ ! -s "$html_file" ]; then
            echo "    SKIP fetch: curl returned $http_code (empty response)"
            SKIP=$((SKIP + 1))
            echo ""
            continue
        fi
    fi

    filesize=$(wc -c < "$html_file" | tr -d ' ')
    if [ "$filesize" -lt 200 ]; then
        echo "    SKIP fetch: response too small (${filesize}B — likely error page or redirect)"
        SKIP=$((SKIP + 1))
        echo ""
        continue
    fi

    # ── Run decruft (all 4 formats) ──
    dc_err="$OUTDIR/${name}.dc.stderr"
    dc_start=$(python3 -c "import time; print(int(time.time()*1000))")

    if ! $DECRUFT "$html_file" --url "$url" -f json > "$OUTDIR/${name}.dc.json" 2>"$dc_err"; then
        echo "    FAIL decruft: $(head -1 "$dc_err")"
        ISSUES=$((ISSUES + 1))
        echo ""
        continue
    fi
    $DECRUFT "$html_file" --url "$url" -f html > "$OUTDIR/${name}.dc.html" 2>/dev/null
    $DECRUFT "$html_file" --url "$url" -f text > "$OUTDIR/${name}.dc.text" 2>/dev/null
    $DECRUFT "$html_file" --url "$url" -f markdown > "$OUTDIR/${name}.dc.md" 2>/dev/null

    dc_end=$(python3 -c "import time; print(int(time.time()*1000))")
    dc_ms=$((dc_end - dc_start))

    # ── Run defuddle (json + html + markdown) ──
    df_err="$OUTDIR/${name}.df.stderr"
    df_start=$(python3 -c "import time; print(int(time.time()*1000))")

    if ! npx defuddle parse --json "$url" > "$OUTDIR/${name}.df.json" 2>"$df_err"; then
        if ! npx defuddle parse --json "$html_file" > "$OUTDIR/${name}.df.json" 2>"$df_err"; then
            reason=$(head -1 "$df_err" | sed 's/^Error: //')
            [ -z "$reason" ] && reason="unknown error (empty stderr)"
            echo "    SKIP defuddle: $reason"
            SKIP=$((SKIP + 1))
            echo ""
            continue
        fi
    fi
    npx defuddle parse "$url" > "$OUTDIR/${name}.df.html" 2>/dev/null || \
        npx defuddle parse "$html_file" > "$OUTDIR/${name}.df.html" 2>/dev/null || true
    npx defuddle parse --markdown "$url" > "$OUTDIR/${name}.df.md" 2>/dev/null || \
        npx defuddle parse --markdown "$html_file" > "$OUTDIR/${name}.df.md" 2>/dev/null || true

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
        with open(path) as f: return json.load(f)
    except: return {}

def load_text(path):
    try:
        with open(path) as f: return f.read()
    except: return ""

dc = load_json(f"{outdir}/{name}.dc.json")
df = load_json(f"{outdir}/{name}.df.json")
dc_html = load_text(f"{outdir}/{name}.dc.html")
df_html = load_text(f"{outdir}/{name}.df.html")
dc_md = load_text(f"{outdir}/{name}.dc.md")
df_md = load_text(f"{outdir}/{name}.df.md")
dc_text = load_text(f"{outdir}/{name}.dc.text")

issues = []

# ── Metadata ──
dc_wc = dc.get('word_count', 0)
df_wc = df.get('wordCount', 0)
dc_title = dc.get('title', '')
df_title = df.get('title', '')

if dc_title != df_title:
    issues.append(f"title: '{dc_title[:35]}' vs '{df_title[:35]}'")

if df_wc > 50:
    ratio = dc_wc / max(df_wc, 1)
    if ratio < 0.5 or ratio > 2.0:
        issues.append(f"words: {dc_wc} vs {df_wc} ({ratio:.1f}x)")

if 'data-decruft-' in dc.get('content', ''):
    count = dc['content'].count('data-decruft-')
    issues.append(f"LEAKED {count} internal attrs")

if dc_wc == 0 and df_wc > 50:
    issues.append("EMPTY: decruft extracted nothing")

# ── HTML format ──
if dc_html and df_html:
    for tag in ['p', 'h1', 'h2', 'pre', 'blockquote']:
        dc_c = dc_html.lower().count(f'<{tag}')
        df_c = df_html.lower().count(f'<{tag}')
        if dc_c == 0 and df_c > 3:
            issues.append(f"html: missing <{tag}> ({df_c} in defuddle)")
elif not dc_html.strip() and dc_wc > 0:
    issues.append("html: empty output")

# ── Markdown format ──
if dc_md and df_md:
    if dc_md.lstrip().startswith('<') and not dc_md.lstrip().startswith('<!--'):
        issues.append("markdown: output looks like HTML")
    dc_md_w = len(dc_md.split())
    df_md_w = len(df_md.split())
    if df_md_w > 50 and dc_md_w > 0:
        r = dc_md_w / df_md_w
        if r < 0.3 or r > 3.0:
            issues.append(f"markdown: words {dc_md_w} vs {df_md_w} ({r:.1f}x)")
elif not dc_md.strip() and dc_wc > 0:
    issues.append("markdown: empty output")

# ── Text format ──
if dc_text and ('<p>' in dc_text or '<div' in dc_text):
    issues.append("text: contains HTML tags")
elif not dc_text.strip() and dc_wc > 0:
    issues.append("text: empty output")

# ── Report ──
wc_pct = f"{dc_wc/max(df_wc,1)*100:.0f}%" if df_wc > 0 else "n/a"

if dc_ms > 0 and df_ms > 0:
    if dc_ms <= df_ms:
        perf = f"{dc_ms}ms vs {df_ms}ms ({df_ms//max(dc_ms,1)}x faster)"
    else:
        perf = f"{dc_ms}ms vs {df_ms}ms ({dc_ms/max(df_ms,1):.1f}x slower)"
else:
    perf = f"{dc_ms}ms vs {df_ms}ms"

if issues:
    print(f"    ISSUES ({len(issues)}):")
    for i in issues:
        print(f"      - {i}")
else:
    print(f"    OK: words={dc_wc}/{df_wc} ({wc_pct}) '{dc_title[:40]}'")

print(f"    perf: {perf}")
sizes = (f"json={os.path.getsize(f'{outdir}/{name}.dc.json')//1024}K/"
         f"{os.path.getsize(f'{outdir}/{name}.df.json')//1024}K  "
         f"html={len(dc_html)//1024}K/{len(df_html)//1024}K  "
         f"md={len(dc_md)//1024}K/{len(df_md)//1024}K  "
         f"text={len(dc_text)//1024}K")
print(f"    sizes: {sizes}")

sys.exit(1 if 'EMPTY' in ' '.join(issues) else 0)
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
printf "║  Results: %d pass, %d issues, %d skipped (of %d sites)\n" \
    "$PASS" "$ISSUES" "$SKIP" "${#URLS[@]}"
if [ $SITE_COUNT -gt 0 ]; then
    dc_avg=$((TOTAL_DC_MS / SITE_COUNT))
    df_avg=$((TOTAL_DF_MS / SITE_COUNT))
    speedup=$((df_avg / (dc_avg > 0 ? dc_avg : 1)))
    printf "║  Avg perf: decruft %dms, defuddle %dms (%dx faster)\n" \
        "$dc_avg" "$df_avg" "$speedup"
fi
echo "╚══════════════════════════════════════════════════════════════════╝"
