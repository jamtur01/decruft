#!/usr/bin/env bash
set -euo pipefail

# Compare decruft vs defuddle across diverse real pages.
# Tests all 4 output formats and compares apples-to-apples:
# json metadata, HTML content, markdown content, text content.

DECRUFT="./target/release/decruft"
OUTDIR="/tmp/decruft-compare"
rm -rf "$OUTDIR"
mkdir -p "$OUTDIR"

cargo build --release 2>/dev/null

# Every URL here has been verified to return real content via curl.
URLS=(
    # ── News ──
    "https://www.bbc.com/news/articles/cp3l4yk5rlgo"
    # ── Personal blogs ──
    "https://www.paulgraham.com/superlinear.html"
    "https://danluu.com/cocktail-ideas/"
    "https://jvns.ca/blog/2024/11/18/how-to-import-a-javascript-library/"
    "https://simonwillison.net/2024/Dec/19/one-shot-python-tools/"
    "https://without.boats/blog/a-four-year-plan/"
    # ── Technical docs ──
    "https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html"
    "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array"
    # ── Wikipedia ──
    "https://en.wikipedia.org/wiki/Rust_(programming_language)"
    # ── GitHub (extractor) ──
    "https://github.com/kepano/defuddle/issues/56"
    # ── Hacker News (extractor) ──
    "https://news.ycombinator.com/item?id=42338514"
    # ── Academic (we correctly extract abstract + metadata only;
    #    defuddle includes ~230 words of sidebar tools) ──
    "https://arxiv.org/abs/2401.00001"
    # ── Hugo static site ──
    "https://gohugo.io/getting-started/quick-start/"
    # ── Classic blogs ──
    "https://martinfowler.com/articles/microservices.html"
    "https://www.joelonsoftware.com/2000/08/09/the-joel-test-12-steps-to-better-code/"
)

PASS=0
ISSUES=0
SKIP=0
TOTAL_DC_MS=0
TOTAL_DF_MS=0
SITE_COUNT=0

echo "╔══════════════════════════════════════════════════════════════════════╗"
echo "║          decruft vs defuddle — apples-to-apples comparison          ║"
echo "║          ${#URLS[@]} sites · JSON + HTML + Markdown + Text                    ║"
echo "╠══════════════════════════════════════════════════════════════════════╣"
echo ""

for url in "${URLS[@]}"; do
    name=$(echo "$url" | sed 's|https\?://||;s|/|_|g;s|[^a-zA-Z0-9_.-]||g' | cut -c1-60)
    echo "--- $name ---"
    echo "    $url"

    # ── Fetch HTML ──
    html_file="$OUTDIR/${name}.source.html"
    fetch_stderr="$OUTDIR/${name}.fetch.err"
    http_code=$(curl -sL --max-time 20 \
        -A "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36" \
        -o "$html_file" -w "%{http_code}" "$url" 2>"$fetch_stderr") || true

    if [ ! -s "$html_file" ]; then
        echo "    SKIP fetch: empty response (HTTP $http_code, $(cat "$fetch_stderr" | head -1))"
        SKIP=$((SKIP + 1)); echo ""; continue
    fi
    filesize=$(wc -c < "$html_file" | tr -d ' ')
    if [ "$filesize" -lt 1000 ]; then
        echo "    SKIP fetch: too small (${filesize}B, HTTP $http_code — likely redirect or error)"
        SKIP=$((SKIP + 1)); echo ""; continue
    fi

    # ── Run decruft (all 4 formats) ──
    dc_err="$OUTDIR/${name}.dc.err"
    dc_start=$(python3 -c "import time; print(int(time.time()*1000))")

    if ! $DECRUFT "$html_file" --url "$url" -f json >"$OUTDIR/${name}.dc.json" 2>"$dc_err"; then
        echo "    FAIL decruft json: $(head -1 "$dc_err")"
        ISSUES=$((ISSUES + 1)); echo ""; continue
    fi
    $DECRUFT "$html_file" --url "$url" -f html >"$OUTDIR/${name}.dc.html" 2>/dev/null
    $DECRUFT "$html_file" --url "$url" -f text >"$OUTDIR/${name}.dc.text" 2>/dev/null
    $DECRUFT "$html_file" --url "$url" -f markdown >"$OUTDIR/${name}.dc.md" 2>/dev/null

    dc_end=$(python3 -c "import time; print(int(time.time()*1000))")
    dc_ms=$((dc_end - dc_start))

    # ── Run defuddle (json + html + markdown) ──
    df_err="$OUTDIR/${name}.df.err"
    df_start=$(python3 -c "import time; print(int(time.time()*1000))")

    # Try URL first, fall back to file
    if ! npx defuddle parse --json "$url" >"$OUTDIR/${name}.df.json" 2>"$df_err"; then
        if ! npx defuddle parse --json "$html_file" >"$OUTDIR/${name}.df.json" 2>>"$df_err"; then
            reason=$(grep -v "^$" "$df_err" | head -1 | sed 's/^Error: //')
            [ -z "$reason" ] && reason="exit code $?, no stderr"
            echo "    SKIP defuddle: $reason"
            SKIP=$((SKIP + 1)); echo ""; continue
        fi
    fi
    npx defuddle parse "$url" >"$OUTDIR/${name}.df.html" 2>/dev/null || \
        npx defuddle parse "$html_file" >"$OUTDIR/${name}.df.html" 2>/dev/null || true
    npx defuddle parse --markdown "$url" >"$OUTDIR/${name}.df.md" 2>/dev/null || \
        npx defuddle parse --markdown "$html_file" >"$OUTDIR/${name}.df.md" 2>/dev/null || true

    df_end=$(python3 -c "import time; print(int(time.time()*1000))")
    df_ms=$((df_end - df_start))

    TOTAL_DC_MS=$((TOTAL_DC_MS + dc_ms))
    TOTAL_DF_MS=$((TOTAL_DF_MS + df_ms))
    SITE_COUNT=$((SITE_COUNT + 1))

    # ── Compare everything ──
    python3 - "$OUTDIR" "$name" "$dc_ms" "$df_ms" "$filesize" << 'PYEOF'
import json, sys, os, re

outdir, name = sys.argv[1], sys.argv[2]
dc_ms, df_ms, src_bytes = int(sys.argv[3]), int(sys.argv[4]), int(sys.argv[5])

def load_json(p):
    try:
        with open(p) as f: return json.load(f)
    except: return {}

def load(p):
    try:
        with open(p) as f: return f.read()
    except: return ""

def wc(text):
    return len(text.split()) if text.strip() else 0

dc_j = load_json(f"{outdir}/{name}.dc.json")
df_j = load_json(f"{outdir}/{name}.df.json")
dc_html = load(f"{outdir}/{name}.dc.html")
df_html = load(f"{outdir}/{name}.df.html")
dc_md = load(f"{outdir}/{name}.dc.md")
df_md = load(f"{outdir}/{name}.df.md")
dc_text = load(f"{outdir}/{name}.dc.text")

issues = []

# ── JSON metadata ──
dc_wc = dc_j.get('word_count', 0)
df_wc = df_j.get('wordCount', 0)
dc_title = dc_j.get('title', '')
df_title = df_j.get('title', '')

if dc_title != df_title:
    issues.append(f"title: '{dc_title[:30]}…' vs '{df_title[:30]}…'")
if df_wc > 50:
    r = dc_wc / max(df_wc, 1)
    if r < 0.5 or r > 2.0:
        issues.append(f"json words: {dc_wc} vs {df_wc} ({r:.1f}x)")
if 'data-decruft-' in dc_j.get('content', ''):
    issues.append(f"LEAKED internal attrs ({dc_j['content'].count('data-decruft-')}x)")
if dc_wc == 0 and df_wc > 50:
    issues.append("EMPTY: decruft extracted nothing")

# ── HTML comparison ──
dc_html_wc = wc(re.sub(r'<[^>]+>', ' ', dc_html))
df_html_wc = wc(re.sub(r'<[^>]+>', ' ', df_html))
if df_html_wc > 50:
    hr = dc_html_wc / max(df_html_wc, 1)
    if hr < 0.4 or hr > 2.5:
        issues.append(f"html words: {dc_html_wc} vs {df_html_wc} ({hr:.1f}x)")
if dc_html.strip() and '<' not in dc_html:
    issues.append("html: no tags in output")
for tag in ['p', 'h1', 'h2', 'pre', 'blockquote']:
    dc_c = dc_html.lower().count(f'<{tag}')
    df_c = df_html.lower().count(f'<{tag}')
    if dc_c == 0 and df_c > 3:
        issues.append(f"html: missing <{tag}> ({df_c} in defuddle)")

# ── Markdown comparison ──
dc_md_wc = wc(dc_md)
df_md_wc = wc(df_md)
if dc_md.lstrip().startswith('<') and not dc_md.lstrip().startswith('<!--'):
    issues.append("markdown: output is HTML, not markdown")
if df_md_wc > 50:
    mr = dc_md_wc / max(df_md_wc, 1)
    if mr < 0.4 or mr > 2.5:
        issues.append(f"markdown words: {dc_md_wc} vs {df_md_wc} ({mr:.1f}x)")
# Check markdown has actual formatting
if dc_md_wc > 100:
    has_md_syntax = bool(re.search(r'[#*>`\[\]]', dc_md))
    if not has_md_syntax:
        issues.append("markdown: no formatting syntax found")

# ── Text comparison ──
dc_text_wc = wc(dc_text)
if dc_text and ('<p>' in dc_text or '<div' in dc_text):
    issues.append("text: contains HTML tags")
if dc_wc > 50 and dc_text_wc == 0:
    issues.append("text: empty despite content")

# ── Cross-format consistency ──
# All formats should produce similar word counts
if dc_wc > 50 and dc_html_wc > 0 and dc_md_wc > 0 and dc_text_wc > 0:
    counts = [dc_html_wc, dc_md_wc, dc_text_wc]
    spread = max(counts) / max(min(counts), 1)
    if spread > 3.0:
        issues.append(f"format spread: html={dc_html_wc} md={dc_md_wc} text={dc_text_wc} ({spread:.1f}x)")

# ── Report ──
wc_pct = f"{dc_wc/max(df_wc,1)*100:.0f}%" if df_wc > 0 else "n/a"
if dc_ms <= df_ms:
    perf = f"{dc_ms}ms vs {df_ms}ms ({df_ms//max(dc_ms,1)}x faster)"
else:
    perf = f"{dc_ms}ms vs {df_ms}ms ({dc_ms/max(df_ms,1):.1f}x slower)"

print(f"    source: {src_bytes//1024}KB")
if issues:
    print(f"    ISSUES ({len(issues)}):")
    for i in issues:
        print(f"      · {i}")
else:
    print(f"    OK: json words={dc_wc}/{df_wc} ({wc_pct})")

# Per-format size comparison (dc/df)
print(f"    perf:  {perf}")
print(f"    json:  {len(dc_j.get('content',''))//1024}K / {len(df_j.get('content',''))//1024}K content")
print(f"    html:  {len(dc_html)//1024}K / {len(df_html)//1024}K  ({dc_html_wc} / {df_html_wc} words)")
print(f"    md:    {len(dc_md)//1024}K / {len(df_md)//1024}K  ({dc_md_wc} / {df_md_wc} words)")
print(f"    text:  {len(dc_text)//1024}K  ({dc_text_wc} words)")

sys.exit(1 if issues else 0)
PYEOF

    result=$?
    if [ $result -eq 0 ]; then
        PASS=$((PASS + 1))
    else
        ISSUES=$((ISSUES + 1))
    fi
    echo ""
done

echo "╔══════════════════════════════════════════════════════════════════════╗"
printf "║  Results: %d pass, %d issues, %d skipped (of %d sites)\n" \
    "$PASS" "$ISSUES" "$SKIP" "${#URLS[@]}"
if [ $SITE_COUNT -gt 0 ]; then
    dc_avg=$((TOTAL_DC_MS / SITE_COUNT))
    df_avg=$((TOTAL_DF_MS / SITE_COUNT))
    speedup=$((df_avg / (dc_avg > 0 ? dc_avg : 1)))
    printf "║  Avg perf: decruft %dms, defuddle %dms (%dx faster)\n" \
        "$dc_avg" "$df_avg" "$speedup"
fi
echo "╚══════════════════════════════════════════════════════════════════════╝"
