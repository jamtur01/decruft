#!/usr/bin/env bash
set -euo pipefail

# Compare decruft vs defuddle across diverse real pages.
# Tests all 4 output formats (json, html, markdown, text).
# Compares word counts, metadata, and format consistency.

DECRUFT="./target/release/decruft"
OUTDIR="/tmp/decruft-compare"
rm -rf "$OUTDIR"
mkdir -p "$OUTDIR"

cargo build --release 2>/dev/null

# All URLs verified to return extractable static HTML content.
# Grouped by extraction method.
URLS=(
    # ── General pipeline (no extractor) ──
    "https://www.bbc.com/news/articles/cp3l4yk5rlgo"
    "https://www.paulgraham.com/superlinear.html"
    "https://danluu.com/cocktail-ideas/"
    "https://jvns.ca/blog/2024/11/18/how-to-import-a-javascript-library/"
    "https://simonwillison.net/2024/Dec/19/one-shot-python-tools/"
    "https://without.boats/blog/a-four-year-plan/"
    "https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html"
    "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array"
    "https://en.wikipedia.org/wiki/Rust_(programming_language)"
    "https://gohugo.io/getting-started/quick-start/"
    "https://martinfowler.com/articles/microservices.html"
    "https://www.joelonsoftware.com/2000/08/09/the-joel-test-12-steps-to-better-code/"
    "https://arxiv.org/abs/2401.00001"
    # ── GitHub extractor ──
    "https://github.com/kepano/defuddle/issues/56"
    # ── Hacker News extractor ──
    "https://news.ycombinator.com/item?id=42338514"
    # ── C2 Wiki extractor (API fetch) ──
    "https://wiki.c2.com/?ExtremeProgramming"
)

PASS=0
ISSUES=0
SKIP=0
TOTAL_DC_MS=0
TOTAL_DF_MS=0
SITE_COUNT=0

printf "╔═══════════════════════════════════════════════════════╗\n"
printf "║  decruft vs defuddle · %d sites · all formats  ║\n" "${#URLS[@]}"
printf "╠═══════════════════════════════════════════════════════╣\n\n"

for url in "${URLS[@]}"; do
    name=$(echo "$url" | sed 's|https\?://||;s|/|_|g;s|[^a-zA-Z0-9_.-]||g' | cut -c1-55)
    printf "%-55s " "$name"

    # ── Fetch HTML ──
    html_file="$OUTDIR/${name}.html"
    if ! curl -sL --max-time 15 \
        -A "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)" \
        -o "$html_file" "$url" 2>/dev/null; then
        echo "SKIP (fetch failed)"
        SKIP=$((SKIP + 1)); continue
    fi
    filesize=$(wc -c < "$html_file" | tr -d ' ')
    if [ "$filesize" -lt 500 ]; then
        echo "SKIP (${filesize}B — too small)"
        SKIP=$((SKIP + 1)); continue
    fi

    # ── Decruft (4 formats) ──
    dc_start=$(python3 -c "import time; print(int(time.time()*1000))")
    $DECRUFT "$html_file" --url "$url" -f json >"$OUTDIR/${name}.dc.json" 2>/dev/null
    $DECRUFT "$html_file" --url "$url" -f html >"$OUTDIR/${name}.dc.html" 2>/dev/null
    $DECRUFT "$html_file" --url "$url" -f text >"$OUTDIR/${name}.dc.text" 2>/dev/null
    $DECRUFT "$html_file" --url "$url" -f markdown >"$OUTDIR/${name}.dc.md" 2>/dev/null
    dc_end=$(python3 -c "import time; print(int(time.time()*1000))")
    dc_ms=$((dc_end - dc_start))

    # ── Defuddle (json only — avoids timeout on large pages) ──
    df_start=$(python3 -c "import time; print(int(time.time()*1000))")
    if ! timeout 30 npx defuddle parse --json "$url" >"$OUTDIR/${name}.df.json" 2>/dev/null; then
        timeout 30 npx defuddle parse --json "$html_file" >"$OUTDIR/${name}.df.json" 2>/dev/null || {
            echo "SKIP (defuddle failed)"
            SKIP=$((SKIP + 1)); continue
        }
    fi
    df_end=$(python3 -c "import time; print(int(time.time()*1000))")
    df_ms=$((df_end - df_start))

    TOTAL_DC_MS=$((TOTAL_DC_MS + dc_ms))
    TOTAL_DF_MS=$((TOTAL_DF_MS + df_ms))
    SITE_COUNT=$((SITE_COUNT + 1))

    # ── Compare ──
    py_out=$(python3 - "$OUTDIR" "$name" "$dc_ms" "$df_ms" << 'PYEOF'
import json, sys, os, re

o, n = sys.argv[1], sys.argv[2]
dc_ms, df_ms = int(sys.argv[3]), int(sys.argv[4])

def lj(p):
    try:
        with open(p) as f: return json.load(f)
    except: return {}

def lt(p):
    try:
        with open(p) as f: return f.read()
    except: return ""

dc = lj(f"{o}/{n}.dc.json")
df = lj(f"{o}/{n}.df.json")
dc_html = lt(f"{o}/{n}.dc.html")
dc_md = lt(f"{o}/{n}.dc.md")
dc_text = lt(f"{o}/{n}.dc.text")

issues = []
dc_wc = dc.get('word_count', 0)
df_wc = df.get('wordCount', 0)

# Metadata
if dc.get('title','') != df.get('title',''):
    issues.append(f"title diff")
if df_wc > 50:
    r = dc_wc / max(df_wc, 1)
    if r < 0.5 or r > 2.0:
        issues.append(f"words {dc_wc}/{df_wc} ({r:.1f}x)")
if 'data-decruft-' in dc.get('content', ''):
    issues.append("leaked attrs")

# Format checks
if dc_text and '<p>' in dc_text:
    issues.append("text has HTML")
if dc_md and dc_md.lstrip().startswith('<') and '<p>' in dc_md[:100]:
    issues.append("md is HTML")

# Performance
spd = f"{df_ms//max(dc_ms,1)}x" if dc_ms < df_ms else f"{dc_ms/max(df_ms,1):.1f}x slow"
pct = f"{dc_wc/max(df_wc,1)*100:.0f}%" if df_wc > 0 else "n/a"

if issues:
    print(f"ISSUES [{', '.join(issues)}]  {dc_wc}/{df_wc}w ({pct})  {dc_ms}ms/{df_ms}ms ({spd})")
else:
    print(f"OK  {dc_wc}/{df_wc}w ({pct})  {dc_ms}ms/{df_ms}ms ({spd})")

print("__STATUS__:ISSUE" if issues else "__STATUS__:OK")
PYEOF
) || true

    # Print the comparison result (first line of python output)
    echo "$py_out" | head -1

    if echo "$py_out" | grep -q "__STATUS__:OK"; then
        PASS=$((PASS + 1))
    else
        ISSUES=$((ISSUES + 1))
    fi
done

echo ""
printf "╔═══════════════════════════════════════════════════════╗\n"
printf "║  %d pass  %d issues  %d skip  (of %d)\n" "$PASS" "$ISSUES" "$SKIP" "${#URLS[@]}"
if [ $SITE_COUNT -gt 0 ]; then
    dc_avg=$((TOTAL_DC_MS / SITE_COUNT))
    df_avg=$((TOTAL_DF_MS / SITE_COUNT))
    printf "║  decruft %dms avg  defuddle %dms avg  (%dx faster)\n" \
        "$dc_avg" "$df_avg" "$((df_avg / (dc_avg > 0 ? dc_avg : 1)))"
fi
printf "╚═══════════════════════════════════════════════════════╝\n"
